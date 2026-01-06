//! JNI 바인딩
//!
//! Java/Elasticsearch와의 통합을 위한 JNI 인터페이스를 제공합니다.
//!
//! # 아키텍처
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │  Java/Elasticsearch                 │
//! │  ├─ NoriTokenizer.java              │
//! │  ├─ NoriAnalyzer.java               │
//! │  └─ NoriTokenizerFactory.java       │
//! └─────────────────────────────────────┘
//!            ↕ JNI
//! ┌─────────────────────────────────────┐
//! │  Rust Native Library (libmecab_ko)  │
//! │  ├─ create_analyzer()               │
//! │  ├─ analyze_text()                  │
//! │  ├─ destroy_analyzer()              │
//! │  └─ Token serialization             │
//! └─────────────────────────────────────┘
//! ```
//!
//! # Java 사용 예제
//!
//! ```java
//! public class NoriTokenizer {
//!     static {
//!         System.loadLibrary("mecab_ko_elasticsearch");
//!     }
//!
//!     private native long createAnalyzer(String config);
//!     private native String analyzeText(long handle, String text);
//!     private native void destroyAnalyzer(long handle);
//! }
//! ```

use crate::analyzer::NoriAnalyzer;
use crate::config::AnalyzerConfig;
use crate::error::Error;
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jlong, jstring};
use jni::JNIEnv;
use std::sync::{Arc, Mutex};

/// Analyzer 핸들 타입
type AnalyzerHandle = Arc<Mutex<NoriAnalyzer>>;

/// 핸들 관리 컨테이너
static ANALYZER_HANDLES: once_cell::sync::Lazy<Mutex<Vec<AnalyzerHandle>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(Vec::new()));

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
pub extern "system" fn Java_com_mecab_ko_elasticsearch_NoriAnalyzer_createAnalyzer(
    env: JNIEnv,
    _class: JClass,
    config_json: JString,
) -> jlong {
    match create_analyzer_impl(&env, config_json) {
        Ok(handle) => handle,
        Err(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", format!("{e}"));
            0
        }
    }
}

fn create_analyzer_impl(env: &JNIEnv, config_json: JString) -> Result<jlong, Error> {
    let config_str: String = env
        .get_string(config_json)
        .map_err(|e| Error::jni(format!("Failed to get config string: {e}")))?
        .into();

    let config: AnalyzerConfig = serde_json::from_str(&config_str)?;
    let analyzer = NoriAnalyzer::new(config)?;

    let handle = Arc::new(Mutex::new(analyzer));
    let handle_id = handle.as_ref() as *const _ as jlong;

    ANALYZER_HANDLES
        .lock()
        .map_err(|e| Error::jni(format!("Failed to lock handle store: {e}")))?
        .push(handle);

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
pub extern "system" fn Java_com_mecab_ko_elasticsearch_NoriAnalyzer_analyzeText(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    text: JString,
) -> jstring {
    match analyze_text_impl(&env, handle, text) {
        Ok(result) => result,
        Err(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", format!("{e}"));
            JObject::null().into_inner()
        }
    }
}

fn analyze_text_impl(env: &JNIEnv, handle: jlong, text: JString) -> Result<jstring, Error> {
    let text_str: String = env
        .get_string(text)
        .map_err(|e| Error::jni(format!("Failed to get text string: {e}")))?
        .into();

    // SAFETY: handle은 create_analyzer에서 생성된 유효한 포인터
    #[allow(unsafe_code)]
    let analyzer = unsafe { &*(handle as *const Mutex<NoriAnalyzer>) };

    let analyzer = analyzer
        .lock()
        .map_err(|e| Error::jni(format!("Failed to lock analyzer: {e}")))?;

    let tokens = analyzer.analyze(&text_str)?;
    let result_json = serde_json::to_string(&tokens)?;

    env.new_string(result_json)
        .map(|s| s.into_inner())
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
pub extern "system" fn Java_com_mecab_ko_elasticsearch_NoriAnalyzer_destroyAnalyzer(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if let Err(e) = destroy_analyzer_impl(handle) {
        let _ = env.throw_new("java/lang/RuntimeException", format!("{e}"));
    }
}

fn destroy_analyzer_impl(handle: jlong) -> Result<(), Error> {
    let mut handles = ANALYZER_HANDLES
        .lock()
        .map_err(|e| Error::jni(format!("Failed to lock handle store: {e}")))?;

    handles.retain(|h| {
        let h_id = h.as_ref() as *const _ as jlong;
        h_id != handle
    });

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
pub extern "system" fn Java_com_mecab_ko_elasticsearch_NoriAnalyzer_getVersion(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    match env.new_string(crate::VERSION) {
        Ok(s) => s.into_inner(),
        Err(_) => JObject::null().into_inner(),
    }
}

/// 설정 유효성 검증
///
/// # Java Signature
///
/// ```java
/// public static native boolean validateConfig(String configJson);
/// ```
#[no_mangle]
pub extern "system" fn Java_com_mecab_ko_elasticsearch_NoriAnalyzer_validateConfig(
    env: JNIEnv,
    _class: JClass,
    config_json: JString,
) -> bool {
    validate_config_impl(&env, config_json).is_ok()
}

fn validate_config_impl(env: &JNIEnv, config_json: JString) -> Result<(), Error> {
    let config_str: String = env
        .get_string(config_json)
        .map_err(|e| Error::jni(format!("Failed to get config string: {e}")))?
        .into();

    let config: AnalyzerConfig = serde_json::from_str(&config_str)?;
    config.validate()?;

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
}
