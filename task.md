# Task: Rust로 경량 A/B Test API 재작성

## 목표
- 현재 Python Flask 앱 (36MB) → Rust로 재작성하여 ~3.6MB로 경량화

## 요구사항

### 기능
1. `/compare` - 이미지를 두 모델에 동시 전송하고 비교
2. `/health` - 헬스체크
3. `/config` - 모델 URL 동적 변경

### 기술 스택
- actix-web 또는 warp
- tokio for async
- reqwest for HTTP requests
- serde for JSON

### 성능 목표
- 메모리: ~3.6MB (현재의 10%)
- 동시 요청 처리
- 정적 컴파일 가능

## 현재 Python 코드 참고
app.py 파일 참조

## 출력
- Cargo.toml
- src/main.rs
