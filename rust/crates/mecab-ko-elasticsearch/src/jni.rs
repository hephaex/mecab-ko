//! JNI 바인딩
//!
//! Java/Elasticsearch와의 통합을 위한 JNI 인터페이스를 제공합니다.
//!
//! # 아키텍처
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │  Java/Elasticsearch                 │
//! │  └─ com.mecab.ko.search.jni         │
//! │     └─ NativeAnalyzer.java          │
//! └─────────────────────────────────────┘
//!            ↕ JNI
//! ┌─────────────────────────────────────┐
//! │  Rust Native Library (libmecab_ko)  │
//! │  ├─ create_analyzer()               │
//! │  ├─ analyze_text()                  │
//! │  ├─ destroy_analyzer()              │
//! │  ├─ get_dictionary_path()           │
//! │  ├─ set_dictionary_path()           │
//! │  └─ Token serialization             │
//! └─────────────────────────────────────┘
//! ```
//!
//! # Java 사용 예제
//!
//! ```java
//! package com.mecab.ko.search.jni;
//!
//! public class NativeAnalyzer {
//!     static {
//!         System.loadLibrary("mecab_ko_elasticsearch");
//!     }
//!
//!     public static native long createAnalyzer(String configJson);
//!     public static native String analyzeText(long handle, String text);
//!     public static native void destroyAnalyzer(long handle);
//!     public static native String getVersion();
//!     public static native boolean validateConfig(String configJson);
//!     public static native String getDictionaryPath();
//!     public static native boolean setDictionaryPath(String path);
//! }
//! ```

use crate::analyzer::NoriAnalyzer;
use crate::config::AnalyzerConfig;
use crate::error::Error;
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jlong, jstring};
use jni::JNIEnv;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

/// 핸들 ID 생성기 (monotonically increasing, raw pointer 노출 없음)
static NEXT_HANDLE: AtomicI64 = AtomicI64::new(1);

/// Analyzer 핸들 레지스트리
///
/// `RwLock<HashMap>` 기반으로 동시 읽기를 허용하면서 삽입/삭제 시에만 write lock.
/// 각 analyzer는 `Arc`로 감싸져 있어 lookup 후 clone하면 map lock을 즉시 해제 가능.
static ANALYZERS: once_cell::sync::Lazy<RwLock<HashMap<i64, Arc<Mutex<NoriAnalyzer>>>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

/// 사전 경로 저장소
static DICTIONARY_PATH: once_cell::sync::Lazy<RwLock<String>> =
    once_cell::sync::Lazy::new(|| RwLock::new(String::new()));

/// Analyzer 생성
///
/// # Java Signature
///
/// ```java
/// public static native long createAnalyzer(String configJson);
/// ```
///
/// # Safety
///
/// JNI 호출에서만 사용되어야 하며, Java에서 올바른 인자 전달 필요
#[no_mangle]
#[allow(unsafe_code)]
pub extern "system" fn Java_com_mecab_ko_search_jni_NativeAnalyzer_createAnalyzer(
    mut env: JNIEnv,
    _class: JClass,
    config_json: JString,
) -> jlong {
    catch_unwind(AssertUnwindSafe(|| match create_analyzer_impl(&mut env, &config_json) {
        Ok(handle) => handle,
        Err(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", format!("{e}"));
            0
        }
    }))
    .unwrap_or_else(|_| {
        eprintln!("[mecab-ko] panic in createAnalyzer JNI");
        0
    })
}

fn create_analyzer_impl(env: &mut JNIEnv, config_json: &JString) -> Result<jlong, Error> {
    let config_str: String = env
        .get_string(config_json)
        .map_err(|e| Error::jni(format!("Failed to get config string: {e}")))?
        .into();

    let config: AnalyzerConfig = serde_json::from_str(&config_str)?;
    let analyzer = NoriAnalyzer::new(config)?;

    let handle_id = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
    let handle = Arc::new(Mutex::new(analyzer));

    ANALYZERS.write().insert(handle_id, handle);

    Ok(handle_id)
}

/// 텍스트 분석
///
/// # Java Signature
///
/// ```java
/// public static native String analyzeText(long handle, String text);
/// ```
///
/// # Safety
///
/// 유효한 analyzer 핸들이 전달되어야 함
#[no_mangle]
#[allow(unsafe_code)]
pub extern "system" fn Java_com_mecab_ko_search_jni_NativeAnalyzer_analyzeText(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    text: JString,
) -> jstring {
    catch_unwind(AssertUnwindSafe(|| match analyze_text_impl(&mut env, handle, &text) {
        Ok(result) => result,
        Err(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", format!("{e}"));
            JObject::null().into_raw() as jstring
        }
    }))
    .unwrap_or_else(|_| {
        eprintln!("[mecab-ko] panic in analyzeText JNI");
        JObject::null().into_raw() as jstring
    })
}

fn analyze_text_impl(env: &mut JNIEnv, handle: jlong, text: &JString) -> Result<jstring, Error> {
    let text_str: String = env
        .get_string(text)
        .map_err(|e| Error::jni(format!("Failed to get text string: {e}")))?
        .into();

    // Read lock으로 Arc clone 후 즉시 해제 -- 최소 contention
    let analyzer_arc = ANALYZERS.read().get(&handle).cloned().ok_or_else(|| {
        Error::jni(format!(
            "Invalid or already destroyed analyzer handle: {handle}"
        ))
    })?;

    let tokens = {
        let guard = analyzer_arc
            .lock()
            .map_err(|e| Error::jni(format!("Failed to lock analyzer: {e}")))?;
        guard.analyze(&text_str)?
    };
    let result_json = serde_json::to_string(&tokens)?;

    env.new_string(result_json)
        .map(JString::into_raw)
        .map_err(|e| Error::jni(format!("Failed to create result string: {e}")))
}

/// Analyzer 해제
///
/// # Java Signature
///
/// ```java
/// public static native void destroyAnalyzer(long handle);
/// ```
///
/// # Safety
///
/// 유효한 analyzer 핸들이 전달되어야 하며, 해제 후 재사용 금지
#[no_mangle]
#[allow(unsafe_code)]
pub extern "system" fn Java_com_mecab_ko_search_jni_NativeAnalyzer_destroyAnalyzer(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if catch_unwind(AssertUnwindSafe(|| {
        if let Err(e) = destroy_analyzer_impl(handle) {
            let _ = env.throw_new("java/lang/RuntimeException", format!("{e}"));
        }
    }))
    .is_err()
    {
        eprintln!("[mecab-ko] panic in destroyAnalyzer JNI");
    }
}

fn destroy_analyzer_impl(handle: jlong) -> Result<(), Error> {
    let removed = ANALYZERS.write().remove(&handle);

    if removed.is_none() {
        return Err(Error::jni(format!(
            "Invalid or already destroyed analyzer handle: {handle}"
        )));
    }

    Ok(())
}

/// 버전 정보 반환
///
/// # Java Signature
///
/// ```java
/// public static native String getVersion();
/// ```
#[no_mangle]
#[allow(unsafe_code)]
pub extern "system" fn Java_com_mecab_ko_search_jni_NativeAnalyzer_getVersion(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    catch_unwind(AssertUnwindSafe(|| {
        env.new_string(crate::VERSION)
            .map_or_else(|_| JObject::null().into_raw() as jstring, JString::into_raw)
    }))
    .unwrap_or_else(|_| {
        eprintln!("[mecab-ko] panic in getVersion JNI");
        JObject::null().into_raw() as jstring
    })
}

/// 설정 유효성 검증
///
/// # Java Signature
///
/// ```java
/// public static native boolean validateConfig(String configJson);
/// ```
#[no_mangle]
#[allow(unsafe_code)]
pub extern "system" fn Java_com_mecab_ko_search_jni_NativeAnalyzer_validateConfig(
    mut env: JNIEnv,
    _class: JClass,
    config_json: JString,
) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        validate_config_impl(&mut env, &config_json).is_ok()
    }))
    .unwrap_or_else(|_| {
        eprintln!("[mecab-ko] panic in validateConfig JNI");
        false
    })
}

fn validate_config_impl(env: &mut JNIEnv, config_json: &JString) -> Result<(), Error> {
    let config_str: String = env
        .get_string(config_json)
        .map_err(|e| Error::jni(format!("Failed to get config string: {e}")))?
        .into();

    let config: AnalyzerConfig = serde_json::from_str(&config_str)?;
    config.validate()?;

    Ok(())
}

/// 사전 경로 반환
///
/// # Java Signature
///
/// ```java
/// public static native String getDictionaryPath();
/// ```
#[no_mangle]
#[allow(unsafe_code)]
pub extern "system" fn Java_com_mecab_ko_search_jni_NativeAnalyzer_getDictionaryPath(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    catch_unwind(AssertUnwindSafe(|| {
        let path = DICTIONARY_PATH.read().clone();
        env.new_string(path)
            .map_or_else(|_| JObject::null().into_raw() as jstring, JString::into_raw)
    }))
    .unwrap_or_else(|_| {
        eprintln!("[mecab-ko] panic in getDictionaryPath JNI");
        JObject::null().into_raw() as jstring
    })
}

/// 사전 경로 설정
///
/// # Java Signature
///
/// ```java
/// public static native boolean setDictionaryPath(String path);
/// ```
///
/// # Safety
///
/// JNI 호출에서만 사용되어야 하며, Java에서 올바른 인자 전달 필요
#[no_mangle]
#[allow(unsafe_code)]
pub extern "system" fn Java_com_mecab_ko_search_jni_NativeAnalyzer_setDictionaryPath(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
) -> jboolean {
    catch_unwind(AssertUnwindSafe(|| {
        match set_dictionary_path_impl(&mut env, &path) {
            Ok(()) => 1u8,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("{e}"));
                0u8
            }
        }
    }))
    .unwrap_or_else(|_| {
        eprintln!("[mecab-ko] panic in setDictionaryPath JNI");
        0u8
    })
}

fn set_dictionary_path_impl(env: &mut JNIEnv, path: &JString) -> Result<(), Error> {
    let path_str: String = env
        .get_string(path)
        .map_err(|e| Error::jni(format!("Failed to get path string: {e}")))?
        .into();

    *DICTIONARY_PATH.write() = path_str;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DecompoundMode;

    #[test]
    fn test_config_serialization_for_jni() {
        let config = AnalyzerConfig::new()
            .with_decompound_mode(DecompoundMode::Mixed)
            .with_stoptags(vec!["J".to_string(), "E".to_string()]);

        let json = serde_json::to_string(&config);
        assert!(json.is_ok());

        let deserialized: std::result::Result<AnalyzerConfig, serde_json::Error> =
            serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_token_serialization_for_jni() {
        use crate::tokenizer::Token;

        let token = Token::new("테스트".to_string(), "NNG".to_string(), 0, 3);
        let json = serde_json::to_string(&token);
        assert!(json.is_ok());

        let deserialized: std::result::Result<Token, serde_json::Error> =
            serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_handle_id_monotonically_increases() {
        let id1 = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
        let id2 = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
        assert!(id2 > id1, "Handle IDs must be monotonically increasing");
    }

    #[test]
    fn test_destroy_invalid_handle_returns_error() {
        let result = destroy_analyzer_impl(i64::MAX);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("Invalid or already destroyed"),
            "Expected 'Invalid or already destroyed' in error message, got: {err_msg}"
        );
    }

    #[test]
    fn test_dictionary_path_read_write() {
        let test_path = "/tmp/test_dict_path_jni_registry";
        *DICTIONARY_PATH.write() = test_path.to_string();
        let read_back = DICTIONARY_PATH.read().clone();
        assert_eq!(read_back, test_path);
    }
}
