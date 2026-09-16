//! Public handler functions for info and metadata endpoints.

use axum::Json;

/// `GET /info` — API metadata (name, version).
pub async fn info_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "agileplus-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn info_handler_returns_json() {
        let resp = info_handler().await;
        let json: serde_json::Value = resp.0;
        assert_eq!(json["name"], "agileplus-api");
        assert!(json["version"].is_string());
        assert!(!json["version"].as_str().unwrap().is_empty());
    }
}
