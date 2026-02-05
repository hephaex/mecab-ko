//! # 핫 리로드 사전 데모
//!
//! 실시간 사전 업데이트 기능을 시연합니다.
//!
//! ## 실행 방법
//!
//! ```bash
//! cargo run --example hot_reload_demo
//! ```
//!
//! ## 주요 기능
//!
//! 1. 실시간 엔트리 추가/제거/수정
//! 2. 델타 업데이트 (배치 작업)
//! 3. 버전 관리 및 롤백
//! 4. 파일 변경 감지 (선택적)

use mecab_ko_dict::{DeltaUpdate, HotReloadDictionary, UserDictionary};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MeCab-Ko 핫 리로드 사전 데모 ===\n");

    // 1. 핫 리로드 사전 생성 (테스트용 사전 사용)
    println!("1. 핫 리로드 사전 초기화...");
    let dict = create_test_dictionary()?;
    let dict = Arc::new(dict);
    println!("   현재 버전: {}\n", dict.current_version());

    // 2. 실시간 엔트리 추가
    println!("2. 실시간 엔트리 추가");
    let v1 = dict.add_entry("딥러닝", "NNG", -1000, None)?;
    println!("   '딥러닝' 추가 (버전: {v1})");

    let entries = dict.lookup("딥러닝")?;
    println!("   조회 결과: {:?}\n", entries);

    let v2 = dict.add_entry("머신러닝", "NNG", -1000, Some("머신러닝".to_string()))?;
    println!("   '머신러닝' 추가 (버전: {v2})");

    let v3 = dict.add_entry("자연어처리", "NNG", -1000, None)?;
    println!("   '자연어처리' 추가 (버전: {v3})\n");

    // 3. 델타 업데이트 (배치 작업)
    println!("3. 델타 업데이트 (배치 작업)");
    let delta = DeltaUpdate::builder()
        .add("챗GPT", "NNP", -2000)
        .add("클로드", "NNP", -2000)
        .add("라마", "NNP", -2000)
        .add_with_reading("앤트로픽", "NNP", -2000, "앤트로픽")
        .build();

    println!(
        "   추가 작업: {} 건, 제거 작업: {} 건, 수정 작업: {} 건",
        delta.addition_count(),
        delta.removal_count(),
        delta.modification_count()
    );

    let v4 = dict.apply_delta(delta)?;
    println!("   델타 적용 완료 (버전: {v4})\n");

    // 4. 엔트리 조회
    println!("4. 엔트리 조회");
    let entries = dict.lookup("챗GPT")?;
    println!("   '챗GPT': {:?}", entries);

    let entries = dict.lookup("클로드")?;
    println!("   '클로드': {:?}\n", entries);

    // 5. 엔트리 수정
    println!("5. 엔트리 수정");
    let v5 = dict.update_entry("딥러닝", |entry| {
        entry.cost = -3000; // 비용 변경
        entry.reading = Some("딥러닝".to_string());
    })?;
    println!("   '딥러닝' 비용 변경 (버전: {v5})");

    let entries = dict.lookup("딥러닝")?;
    println!("   조회 결과: {:?}\n", entries);

    // 6. 엔트리 제거
    println!("6. 엔트리 제거");
    let (v6, removed_count) = dict.remove_entry("머신러닝")?;
    println!("   '머신러닝' 제거 ({removed_count}건, 버전: {v6})");

    let entries = dict.lookup("머신러닝")?;
    println!("   조회 결과: {:?}\n", entries);

    // 7. 버전 히스토리 조회
    println!("7. 버전 히스토리");
    let history = dict.version_history()?;
    for version_info in &history {
        let age = version_info
            .age()
            .map(|d| format!("{}ms", d.as_millis()))
            .unwrap_or_else(|| "N/A".to_string());
        println!(
            "   버전 {}: 사용자 사전 {} 엔트리, 생성 시간: {age} 전",
            version_info.version, version_info.user_entry_count
        );
    }
    println!();

    // 8. 롤백
    println!("8. 버전 롤백");
    println!("   현재 버전: {}", dict.current_version());
    println!("   버전 2로 롤백...");
    dict.rollback(2)?;
    println!("   롤백 완료, 현재 버전: {}", dict.current_version());

    let entries = dict.lookup("챗GPT")?;
    println!("   '챗GPT' 조회 결과 (롤백 후): {:?}\n", entries);

    // 9. 사용자 사전 내보내기/가져오기
    println!("9. 사용자 사전 내보내기/가져오기");
    let exported = dict.export_user_dict()?;
    println!("   내보낸 사전 크기: {} 엔트리", exported.len());

    let mut new_user_dict = UserDictionary::new();
    new_user_dict.add_entry("트랜스포머", "NNG", Some(-2000), None);
    new_user_dict.add_entry("어텐션", "NNG", Some(-2000), None);

    let v7 = dict.import_user_dict(new_user_dict)?;
    println!("   사전 가져오기 완료 (버전: {v7})\n");

    // 10. 동시성 테스트
    println!("10. 동시성 테스트 (멀티스레드)");
    let dict_clone1 = Arc::clone(&dict);
    let dict_clone2 = Arc::clone(&dict);
    let dict_clone3 = Arc::clone(&dict);

    let handle1 = thread::spawn(move || {
        for i in 0..5 {
            let _ = dict_clone1.add_entry(
                format!("테스트{i}"),
                "NNG",
                -1000,
                Some(format!("테스트{i}")),
            );
            thread::sleep(Duration::from_millis(10));
        }
    });

    let handle2 = thread::spawn(move || {
        for _ in 0..5 {
            let _ = dict_clone2.lookup("딥러닝");
            thread::sleep(Duration::from_millis(10));
        }
    });

    let handle3 = thread::spawn(move || {
        for i in 0..3 {
            let delta = DeltaUpdate::builder()
                .add(format!("배치{i}"), "NNG", -1000)
                .build();
            let _ = dict_clone3.apply_delta(delta);
            thread::sleep(Duration::from_millis(15));
        }
    });

    handle1.join().expect("thread 1 failed");
    handle2.join().expect("thread 2 failed");
    handle3.join().expect("thread 3 failed");

    println!("   동시성 테스트 완료");
    println!("   최종 버전: {}\n", dict.current_version());

    // 11. 델타 히스토리
    println!("11. 델타 히스토리");
    let delta_history = dict.delta_history()?;
    println!("   델타 히스토리 크기: {}", delta_history.len());
    for (i, delta) in delta_history.iter().take(5).enumerate() {
        println!("   델타 {}: {} 변경", i + 1, delta.total_changes());
    }

    println!("\n=== 데모 완료 ===");
    Ok(())
}

/// 테스트용 사전 생성
///
/// 실제 환경에서는 `HotReloadDictionary::new()` 또는
/// `HotReloadDictionary::new_default()`를 사용합니다.
fn create_test_dictionary() -> Result<HotReloadDictionary, Box<dyn std::error::Error>> {
    // 실제 사전이 설치된 경로를 사용
    // 만약 사전이 없으면 에러 메시지와 함께 종료
    match HotReloadDictionary::new_default() {
        Ok(dict) => Ok(dict),
        Err(_) => {
            eprintln!("경고: 시스템 사전을 찾을 수 없습니다.");
            eprintln!("테스트용 더미 사전으로 계속합니다...\n");

            // 더미 사전 생성 (실제로는 사용 불가)
            // 여기서는 예제를 위해 에러를 반환
            Err("System dictionary not found. Please install mecab-ko-dic or set MECAB_DICDIR environment variable.".into())
        }
    }
}
