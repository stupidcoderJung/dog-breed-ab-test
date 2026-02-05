# 🐕 Dog Breed Classification A/B Testing API

강아지 견종 분류 모델을 위한 A/B 테스트 API입니다. 두 개의 모델을 동시에 실행하고 성능을 실시간으로 비교할 수 있습니다.

## ✨ 주요 기능

- ✅ **동시 예측**: 두 모델에 동시에 요청을 보내 속도 최적화
- 📊 **상세 비교**: Top-1 일치율, 신뢰도 점수 차이, Top-5 겹침 분석
- ⚡ **경량 설계**: 메모리 사용량 ~36MB, CPU 효율적
- 🔧 **동적 설정**: 런타임 중 모델 URL 변경 가능
- 🌐 **REST API**: 간단한 HTTP 인터페이스

## 📦 설치

### 1. 저장소 클론

```bash
git clone <repository-url>
cd dog-breed-ab-test
```

### 2. 의존성 설치

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y python3-flask python3-requests
```

**macOS/다른 환경:**
```bash
pip3 install -r requirements.txt
```

## 🚀 실행

### 기본 실행

```bash
python3 app.py
```

기본 포트: `8893`

### 백그라운드 실행 (권장)

```bash
nohup python3 app.py > app.log 2>&1 &
```

### 프로세스 확인

```bash
ps aux | grep app.py
```

## 📖 API 사용법

### 1️⃣ 헬스체크

서버가 정상 작동하는지 확인합니다.

```bash
curl http://192.168.0.40:8893/health | jq
```

**응답:**
```json
{
  "status": "ok",
  "model_a_url": "http://192.168.0.59:8891/predict",
  "model_b_url": "http://192.168.0.59:8892/predict"
}
```

### 2️⃣ A/B 테스트 (이미지 비교)

이미지를 두 모델에 보내고 결과를 비교합니다.

```bash
curl -X POST http://192.168.0.40:8893/compare \
  -F "image=@/path/to/dog_image.png" | jq
```

**응답 예시:**
```json
{
  "model_a": {
    "success": true,
    "model": "Model A",
    "predictions": [
      {
        "breed_en": "Border collie",
        "breed_ko": "보더 콜리",
        "class_id": 81,
        "score": 0.2687
      },
      ...
    ],
    "response_time": 0.234
  },
  "model_b": {
    "success": true,
    "model": "Model B",
    "predictions": [
      {
        "breed_en": "Border collie",
        "breed_ko": "보더 콜리",
        "class_id": 81,
        "score": 0.8521
      },
      ...
    ],
    "response_time": 0.189
  },
  "comparison": {
    "comparison_available": true,
    "top1_agreement": true,
    "top1_model_a": {
      "breed": "보더 콜리",
      "breed_en": "Border collie",
      "score": 0.2687
    },
    "top1_model_b": {
      "breed": "보더 콜리",
      "breed_en": "Border collie",
      "score": 0.8521
    },
    "score_difference": 0.5834,
    "top5_overlap": "5/5",
    "winner": "Model B",
    "response_time_diff": 0.045
  }
}
```

### 3️⃣ 커스텀 모델 URL 지정

요청마다 다른 모델 URL을 사용할 수 있습니다.

```bash
curl -X POST http://192.168.0.40:8893/compare \
  -F "image=@dog.png" \
  -F "model_a_url=http://192.168.0.59:8891/predict" \
  -F "model_b_url=http://192.168.0.59:8899/predict" | jq
```

### 4️⃣ 기본 모델 URL 변경

런타임 중 기본 모델 URL을 변경합니다.

```bash
curl -X POST http://192.168.0.40:8893/config \
  -H "Content-Type: application/json" \
  -d '{
    "model_a_url": "http://192.168.0.59:8891/predict",
    "model_b_url": "http://192.168.0.59:8892/predict"
  }' | jq
```

**응답:**
```json
{
  "success": true,
  "model_a_url": "http://192.168.0.59:8891/predict",
  "model_b_url": "http://192.168.0.59:8892/predict"
}
```

## 🔍 비교 결과 해석

| 필드 | 설명 |
|------|------|
| `top1_agreement` | 두 모델의 1위 예측이 동일한지 여부 |
| `score_difference` | 두 모델의 신뢰도 점수 차이 (절대값) |
| `top5_overlap` | Top-5 예측에서 겹치는 견종 개수 |
| `winner` | 더 높은 신뢰도 점수를 가진 모델 |
| `response_time_diff` | 응답 시간 차이 (초) |

## 🛠️ 설정

`app.py` 파일에서 기본 설정을 변경할 수 있습니다:

```python
# 기본 모델 엔드포인트
MODEL_A_URL = "http://192.168.0.59:8891/predict"
MODEL_B_URL = "http://192.168.0.59:8892/predict"
```

```python
# 서버 포트 변경
if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8893, debug=False)
```

## 📊 성능

- **메모리 사용량**: ~36 MB (idle)
- **CPU 사용률**: 0.0% (idle)
- **동시 요청**: ThreadPoolExecutor로 병렬 처리
- **타임아웃**: 30초 (모델 응답 대기)

## 🐛 문제 해결

### 서버가 시작되지 않는 경우

1. 포트가 이미 사용 중인지 확인:
   ```bash
   lsof -i :8893
   ```

2. 의존성 재설치:
   ```bash
   sudo apt install -y python3-flask python3-requests
   ```

### 모델 연결 실패

1. 모델 서버가 실행 중인지 확인:
   ```bash
   curl http://192.168.0.59:8891/health
   curl http://192.168.0.59:8892/health
   ```

2. 방화벽 규칙 확인

### 로그 확인

```bash
tail -f app.log
```

## 📄 라이선스

MIT License

## 👥 기여

Issues와 Pull Requests는 언제나 환영합니다!

## 📞 문의

문제가 발생하면 Issue를 생성해주세요.

---

**Created by Telecro 🖤**

## 🚀 Rust 버전 (최적화)

**88% 메모리 절감!** Python 버전 (36MB) → Rust 버전 (4.3MB)

### 빌드 방법

```bash
cargo build --release
```

### 실행

```bash
./target/release/dog-breed-ab-test
```

또는 백그라운드:
```bash
nohup ./target/release/dog-breed-ab-test > server.log 2>&1 &
```

### 성능 비교

| 버전 | 메모리 | 개선율 |
|------|--------|---------|
| Python (Flask) | 36.0 MB | - |
| **Rust (actix-web)** | **4.3 MB** | **88% ↓** |

자세한 내용은 [PERFORMANCE.md](PERFORMANCE.md)를 참조하세요.

