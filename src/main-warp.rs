use warp::{Filter, Reply};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Prediction {
    breed_en: String,
    breed_ko: String,
    class_id: u32,
    score: f64,
}

#[derive(Debug, Serialize)]
struct ModelResult {
    success: bool,
    model: String,
    predictions: Vec<Prediction>,
    response_time: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct Comparison {
    comparison_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    top1_agreement: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top1_model_a: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top1_model_b: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    score_difference: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top5_overlap: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    winner: Option<String>,
}

#[derive(Debug, Serialize)]
struct CompareResponse {
    model_a: ModelResult,
    model_b: ModelResult,
    comparison: Comparison,
}

#[derive(Clone)]
struct AppState {
    model_a_url: String,
    model_b_url: String,
}

async fn predict_model(url: &str, image_data: bytes::Bytes, model_name: &str) -> ModelResult {
    let start = std::time::Instant::now();
    let client = reqwest::Client::new();
    
    let part = reqwest::multipart::Part::bytes(image_data.to_vec())
        .file_name("image.png")
        .mime_str("image/png")
        .unwrap();
    
    let form = reqwest::multipart::Form::new().part("image", part);
    
    match client.post(url).multipart(form).send().await {
        Ok(resp) if resp.status().is_success() => {
            let elapsed = start.elapsed().as_secs_f64();
            match resp.json::<serde_json::Value>().await {
                Ok(json) => {
                    let predictions: Vec<Prediction> = serde_json::from_value(
                        json.get("predictions").cloned().unwrap_or(serde_json::json!([]))
                    ).unwrap_or_default();
                    
                    ModelResult {
                        success: true,
                        model: model_name.to_string(),
                        predictions,
                        response_time: (elapsed * 1000.0).round() / 1000.0,
                        error: None,
                    }
                }
                Err(e) => ModelResult {
                    success: false,
                    model: model_name.to_string(),
                    predictions: vec![],
                    response_time: (start.elapsed().as_secs_f64() * 1000.0).round() / 1000.0,
                    error: Some(format!("JSON parse error: {}", e)),
                },
            }
        }
        Ok(resp) => ModelResult {
            success: false,
            model: model_name.to_string(),
            predictions: vec![],
            response_time: (start.elapsed().as_secs_f64() * 1000.0).round() / 1000.0,
            error: Some(format!("HTTP {}", resp.status())),
        },
        Err(e) => ModelResult {
            success: false,
            model: model_name.to_string(),
            predictions: vec![],
            response_time: (start.elapsed().as_secs_f64() * 1000.0).round() / 1000.0,
            error: Some(e.to_string()),
        },
    }
}

fn compare_predictions(result_a: &ModelResult, result_b: &ModelResult) -> Comparison {
    if !result_a.success || !result_b.success || result_a.predictions.is_empty() || result_b.predictions.is_empty() {
        return Comparison {
            comparison_available: false,
            top1_agreement: None,
            top1_model_a: None,
            top1_model_b: None,
            score_difference: None,
            top5_overlap: None,
            winner: None,
        };
    }
    
    let top1_a = &result_a.predictions[0];
    let top1_b = &result_b.predictions[0];
    let agreement = top1_a.breed_en == top1_b.breed_en;
    let score_diff = (top1_a.score - top1_b.score).abs();
    
    let top5_a: std::collections::HashSet<_> = result_a.predictions.iter().take(5).map(|p| &p.breed_en).collect();
    let top5_b: std::collections::HashSet<_> = result_b.predictions.iter().take(5).map(|p| &p.breed_en).collect();
    let overlap = top5_a.intersection(&top5_b).count();
    
    Comparison {
        comparison_available: true,
        top1_agreement: Some(agreement),
        top1_model_a: Some(serde_json::json!({
            "breed": top1_a.breed_ko,
            "breed_en": top1_a.breed_en,
            "score": (top1_a.score * 10000.0).round() / 10000.0,
        })),
        top1_model_b: Some(serde_json::json!({
            "breed": top1_b.breed_ko,
            "breed_en": top1_b.breed_en,
            "score": (top1_b.score * 10000.0).round() / 10000.0,
        })),
        score_difference: Some((score_diff * 10000.0).round() / 10000.0),
        top5_overlap: Some(format!("{}/5", overlap)),
        winner: Some(if top1_a.score > top1_b.score { "Model A" } else { "Model B" }.to_string()),
    }
}

async fn handle_compare(
    mut form: warp::multipart::FormData,
    state: Arc<RwLock<AppState>>,
) -> Result<impl Reply, warp::Rejection> {
    use futures::StreamExt;
    
    let mut image_data = bytes::BytesMut::new();
    
    while let Some(Ok(part)) = form.next().await {
        let mut stream = part.stream();
        while let Some(Ok(chunk)) = stream.next().await {
            image_data.extend_from_slice(&chunk);
        }
    }
    
    let image_bytes = image_data.freeze();
    let state_read = state.read().await;
    let url_a = state_read.model_a_url.clone();
    let url_b = state_read.model_b_url.clone();
    drop(state_read);
    
    let (result_a, result_b) = tokio::join!(
        predict_model(&url_a, image_bytes.clone(), "Model A"),
        predict_model(&url_b, image_bytes, "Model B")
    );
    
    let comparison = compare_predictions(&result_a, &result_b);
    
    Ok(warp::reply::json(&CompareResponse {
        model_a: result_a,
        model_b: result_b,
        comparison,
    }))
}

async fn handle_health(state: Arc<RwLock<AppState>>) -> Result<impl Reply, warp::Rejection> {
    let state_read = state.read().await;
    Ok(warp::reply::json(&serde_json::json!({
        "status": "ok",
        "model_a_url": state_read.model_a_url,
        "model_b_url": state_read.model_b_url,
    })))
}

#[derive(Deserialize)]
struct ConfigUpdate {
    model_a_url: Option<String>,
    model_b_url: Option<String>,
}

async fn handle_config(
    config: ConfigUpdate,
    state: Arc<RwLock<AppState>>,
) -> Result<impl Reply, warp::Rejection> {
    let mut state_write = state.write().await;
    
    if let Some(ref url) = config.model_a_url {
        state_write.model_a_url = url.clone();
    }
    if let Some(ref url) = config.model_b_url {
        state_write.model_b_url = url.clone();
    }
    
    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "model_a_url": state_write.model_a_url,
        "model_b_url": state_write.model_b_url,
    })))
}

#[tokio::main]
async fn main() {
    let state = Arc::new(RwLock::new(AppState {
        model_a_url: "http://192.168.0.59:8891/predict".to_string(),
        model_b_url: "http://192.168.0.59:8892/predict".to_string(),
    }));
    
    let state_filter = warp::any().map(move || state.clone());
    
    let compare = warp::post()
        .and(warp::path("compare"))
        .and(warp::multipart::form().max_length(10_000_000))
        .and(state_filter.clone())
        .and_then(handle_compare);
    
    let health = warp::get()
        .and(warp::path("health"))
        .and(state_filter.clone())
        .and_then(handle_health);
    
    let config = warp::post()
        .and(warp::path("config"))
        .and(warp::body::json())
        .and(state_filter)
        .and_then(handle_config);
    
    let routes = compare.or(health).or(config);
    
    println!("Starting server on 0.0.0.0:8893");
    warp::serve(routes).run(([0, 0, 0, 0], 8893)).await;
}
