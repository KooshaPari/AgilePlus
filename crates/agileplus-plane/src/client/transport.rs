use anyhow::{Context, Result};
use reqwest::{Method, Response};
use serde::{Serialize, de::DeserializeOwned};

pub(super) async fn request_json<T: Serialize + ?Sized>(
    client: &reqwest::Client,
    method: Method,
    url: &str,
    api_key: &str,
    body: &T,
) -> Result<Response> {
    client
        .request(method, url)
        .header("X-API-Key", api_key)
        .json(body)
        .send()
        .await
        .context("request with json body failed")
}

pub(super) async fn request_json_value(
    client: &reqwest::Client,
    method: Method,
    url: &str,
    api_key: &str,
    body: &serde_json::Value,
) -> Result<Response> {
    client
        .request(method, url)
        .header("X-API-Key", api_key)
        .json(body)
        .send()
        .await
        .context("request with json body failed")
}

pub(super) async fn request_raw_body(
    client: &reqwest::Client,
    method: Method,
    url: &str,
    api_key: &str,
    raw_body: &str,
) -> Result<Response> {
    client
        .request(method, url)
        .header("X-API-Key", api_key)
        .header("Content-Type", "application/json")
        .body(raw_body.to_string())
        .send()
        .await
        .context("request with raw body failed")
}

pub(super) async fn request_without_body(
    client: &reqwest::Client,
    method: Method,
    url: &str,
    api_key: &str,
) -> Result<Response> {
    client
        .request(method, url)
        .header("X-API-Key", api_key)
        .send()
        .await
        .context("request without body failed")
}

pub(super) async fn read_text_response(response: Response, context: &str) -> Result<String> {
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Plane.so API error {status}: {body}");
    }

    response.text().await.context(context.to_owned())
}

pub(super) async fn read_json_response<T: DeserializeOwned>(
    response: Response,
    context: &str,
) -> Result<T> {
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Plane.so API error {status}: {body}");
    }

    response.json().await.context(context.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn server_with(path_str: &str, method_str: &str, status: u16, body: &str) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method(method_str))
            .and(path(path_str))
            .respond_with(ResponseTemplate::new(status).set_body_string(body.to_string()))
            .mount(&server)
            .await;
        server
    }

    #[tokio::test]
    async fn request_json_sends_api_key_and_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/x"))
            .and(header("X-API-Key", "secret"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let resp = request_json(
            &client,
            Method::POST,
            &format!("{}/x", server.uri()),
            "secret",
            &serde_json::json!({"a": 1}),
        )
        .await
        .unwrap();
        assert_eq!(resp.status().as_u16(), 200);
    }

    #[tokio::test]
    async fn request_json_connection_error_is_contextual() {
        let client = reqwest::Client::new();
        let err = request_json(
            &client,
            Method::POST,
            "http://127.0.0.1:1/nope",
            "k",
            &serde_json::json!({}),
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("json body failed"));
    }

    #[tokio::test]
    async fn request_json_value_sends_json() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/y"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let resp = request_json_value(
            &client,
            Method::PATCH,
            &format!("{}/y", server.uri()),
            "k",
            &serde_json::json!({"b": 2}),
        )
        .await
        .unwrap();
        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn request_json_value_error_has_context() {
        let client = reqwest::Client::new();
        let err = request_json_value(
            &client,
            Method::POST,
            "http://127.0.0.1:1/nope",
            "k",
            &serde_json::json!({}),
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("json body failed"));
    }

    #[tokio::test]
    async fn request_raw_body_sets_content_type() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/raw"))
            .and(header("Content-Type", "application/json"))
            .and(header("X-API-Key", "k"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let resp = request_raw_body(
            &client,
            Method::POST,
            &format!("{}/raw", server.uri()),
            "k",
            "{\"a\":1}",
        )
        .await
        .unwrap();
        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn request_raw_body_error_has_context() {
        let client = reqwest::Client::new();
        let err = request_raw_body(&client, Method::POST, "http://127.0.0.1:1/x", "k", "{}")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("raw body failed"));
    }

    #[tokio::test]
    async fn request_without_body_succeeds() {
        let server = server_with("/z", "GET", 200, "ok").await;
        let client = reqwest::Client::new();
        let resp = request_without_body(&client, Method::GET, &format!("{}/z", server.uri()), "k")
            .await
            .unwrap();
        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn request_without_body_error_has_context() {
        let client = reqwest::Client::new();
        let err = request_without_body(&client, Method::GET, "http://127.0.0.1:1/x", "k")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("without body failed"));
    }

    #[tokio::test]
    async fn read_text_response_returns_body_on_success() {
        let server = server_with("/t", "GET", 200, "hello").await;
        let resp = reqwest::get(format!("{}/t", server.uri())).await.unwrap();
        assert_eq!(read_text_response(resp, "ctx").await.unwrap(), "hello");
    }

    #[tokio::test]
    async fn read_text_response_errors_on_non_success() {
        let server = server_with("/t", "GET", 404, "missing").await;
        let resp = reqwest::get(format!("{}/t", server.uri())).await.unwrap();
        let err = read_text_response(resp, "ctx").await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("404"), "{msg}");
        assert!(msg.contains("missing"), "{msg}");
    }

    #[tokio::test]
    async fn read_json_response_parses_success_body() {
        let server = server_with("/j", "GET", 200, r#"{"n":5}"#).await;
        let resp = reqwest::get(format!("{}/j", server.uri())).await.unwrap();
        let value: serde_json::Value = read_json_response(resp, "ctx").await.unwrap();
        assert_eq!(value["n"], 5);
    }

    #[tokio::test]
    async fn read_json_response_errors_on_non_success() {
        let server = server_with("/j", "GET", 500, "boom").await;
        let resp = reqwest::get(format!("{}/j", server.uri())).await.unwrap();
        let err = read_json_response::<serde_json::Value>(resp, "ctx")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("500"));
    }

    #[tokio::test]
    async fn read_json_response_errors_on_malformed_json() {
        let server = server_with("/j", "GET", 200, "not-json").await;
        let resp = reqwest::get(format!("{}/j", server.uri())).await.unwrap();
        let err = read_json_response::<serde_json::Value>(resp, "ctx")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("ctx"));
    }

    #[tokio::test]
    async fn read_text_response_error_includes_empty_body_as_default() {
        let server = server_with("/e", "GET", 502, "").await;
        let resp = reqwest::get(format!("{}/e", server.uri())).await.unwrap();
        let err = read_text_response(resp, "ctx").await.unwrap_err();
        assert!(err.to_string().contains("502"));
    }
}
