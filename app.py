#!/usr/bin/env python3
"""
Dog Breed Classification Model A/B Testing API
강아지 견종 분류 모델 A/B 테스트 API
"""

from flask import Flask, request, jsonify
import requests
from concurrent.futures import ThreadPoolExecutor
import time
from io import BytesIO

app = Flask(__name__)

# 기본 모델 엔드포인트
MODEL_A_URL = "http://192.168.0.59:8891/predict"
MODEL_B_URL = "http://192.168.0.59:8892/predict"


def predict_model(url, image_data, model_name):
    """
    단일 모델에 예측 요청을 보냅니다.
    
    Args:
        url: 모델 API 엔드포인트
        image_data: 이미지 바이너리 데이터
        model_name: 모델 식별자 (예: "Model A")
    
    Returns:
        dict: 예측 결과 및 메타데이터
    """
    try:
        start_time = time.time()
        files = {'image': ('image.png', image_data, 'image/png')}
        response = requests.post(url, files=files, timeout=30)
        elapsed = time.time() - start_time
        
        if response.status_code == 200:
            return {
                "success": True,
                "model": model_name,
                "predictions": response.json().get('predictions', []),
                "response_time": round(elapsed, 3)
            }
        else:
            return {
                "success": False,
                "model": model_name,
                "error": f"HTTP {response.status_code}",
                "response_time": round(elapsed, 3)
            }
    except Exception as e:
        return {
            "success": False,
            "model": model_name,
            "error": str(e),
            "response_time": None
        }


def compare_predictions(result_a, result_b):
    """
    두 모델의 예측 결과를 비교 분석합니다.
    
    Args:
        result_a: Model A 예측 결과
        result_b: Model B 예측 결과
    
    Returns:
        dict: 비교 분석 결과
    """
    if not (result_a['success'] and result_b['success']):
        return {"comparison_available": False}
    
    preds_a = result_a['predictions']
    preds_b = result_b['predictions']
    
    if not preds_a or not preds_b:
        return {"comparison_available": False}
    
    top1_a = preds_a[0]
    top1_b = preds_b[0]
    
    # Top-1 일치 여부
    agreement = top1_a['breed_en'] == top1_b['breed_en']
    
    # 점수 차이
    score_diff = abs(top1_a['score'] - top1_b['score'])
    
    # Top-5 일치율
    top5_breeds_a = {p['breed_en'] for p in preds_a[:5]}
    top5_breeds_b = {p['breed_en'] for p in preds_b[:5]}
    top5_overlap = len(top5_breeds_a & top5_breeds_b)
    
    return {
        "comparison_available": True,
        "top1_agreement": agreement,
        "top1_model_a": {
            "breed": top1_a['breed_ko'],
            "breed_en": top1_a['breed_en'],
            "score": round(top1_a['score'], 4)
        },
        "top1_model_b": {
            "breed": top1_b['breed_ko'],
            "breed_en": top1_b['breed_en'],
            "score": round(top1_b['score'], 4)
        },
        "score_difference": round(score_diff, 4),
        "top5_overlap": f"{top5_overlap}/5",
        "winner": "Model A" if top1_a['score'] > top1_b['score'] else "Model B",
        "response_time_diff": round(result_a['response_time'] - result_b['response_time'], 3)
    }


@app.route('/compare', methods=['POST'])
def compare():
    """
    A/B 테스트 엔드포인트
    
    이미지를 두 모델에 동시에 보내고 결과를 비교합니다.
    
    Form Data:
        image: 이미지 파일 (required)
        model_a_url: Model A URL (optional, 기본값: MODEL_A_URL)
        model_b_url: Model B URL (optional, 기본값: MODEL_B_URL)
    
    Returns:
        JSON: 두 모델의 예측 결과 및 비교 분석
    """
    if 'image' not in request.files:
        return jsonify({"error": "No image provided"}), 400
    
    image_file = request.files['image']
    image_data = image_file.read()
    
    # 커스텀 모델 URL (선택적)
    model_a_url = request.form.get('model_a_url', MODEL_A_URL)
    model_b_url = request.form.get('model_b_url', MODEL_B_URL)
    
    # 동시에 두 모델에 요청
    with ThreadPoolExecutor(max_workers=2) as executor:
        future_a = executor.submit(predict_model, model_a_url, BytesIO(image_data), "Model A")
        future_b = executor.submit(predict_model, model_b_url, BytesIO(image_data), "Model B")
        
        result_a = future_a.result()
        result_b = future_b.result()
    
    # 비교 결과 생성
    comparison = compare_predictions(result_a, result_b)
    
    return jsonify({
        "model_a": result_a,
        "model_b": result_b,
        "comparison": comparison
    })


@app.route('/health', methods=['GET'])
def health():
    """헬스체크 엔드포인트"""
    return jsonify({
        "status": "ok",
        "model_a_url": MODEL_A_URL,
        "model_b_url": MODEL_B_URL
    })


@app.route('/config', methods=['POST'])
def update_config():
    """
    모델 URL 동적 변경 엔드포인트
    
    JSON Body:
        model_a_url: Model A URL (optional)
        model_b_url: Model B URL (optional)
    """
    global MODEL_A_URL, MODEL_B_URL
    
    data = request.json
    if 'model_a_url' in data:
        MODEL_A_URL = data['model_a_url']
    if 'model_b_url' in data:
        MODEL_B_URL = data['model_b_url']
    
    return jsonify({
        "success": True,
        "model_a_url": MODEL_A_URL,
        "model_b_url": MODEL_B_URL
    })


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8893, debug=False)
