//! Edge-path coverage for `middleware::otel::OtelTracingMiddleware`.
//!
//! The happy path (a request flows through, status is recorded, the W3C
//! `traceparent` header is read) is exercised end-to-end elsewhere. This file
//! pins down the paths that a Router-driven test cannot reach:
//!
//! - `Service::poll_ready` forwarding, including an inner readiness failure
//! - inner-service `call` failure propagating through the middleware
//! - a `traceparent` header that is present but not valid UTF-8, which must
//!   degrade to the `"-"` placeholder rather than panic or drop the request

use std::convert::Infallible;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use agileplus_api::middleware::otel::opentelemetry_tracing_layer;
use axum::body::Body;
use axum::http::{HeaderValue, Request, Response, StatusCode};
use tower::{Layer, Service};

/// Inner service that is never ready — models a downstream that reports its
/// own failure during readiness probing.
#[derive(Clone)]
struct NotReady;

impl Service<Request<Body>> for NotReady {
    type Response = Response<Body>;
    type Error = io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Err(io::Error::new(
            io::ErrorKind::ConnectionRefused,
            "down",
        )))
    }

    fn call(&mut self, _req: Request<Body>) -> Self::Future {
        unreachable!("readiness failed; call must not be reached in this test")
    }
}

/// Inner service that is always ready but fails when called.
#[derive(Clone)]
struct FailsOnCall;

impl Service<Request<Body>> for FailsOnCall {
    type Response = Response<Body>;
    type Error = io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request<Body>) -> Self::Future {
        Box::pin(async { Err(io::Error::new(io::ErrorKind::BrokenPipe, "boom")) })
    }
}

#[tokio::test]
async fn poll_ready_forwards_a_ready_inner_service() {
    let inner = tower::service_fn(|_req: Request<Body>| async {
        Ok::<_, Infallible>(
            Response::builder()
                .status(StatusCode::ACCEPTED)
                .body(Body::empty())
                .unwrap(),
        )
    });
    let mut service = opentelemetry_tracing_layer().layer(inner);
    let mut cx = Context::from_waker(Waker::noop());

    assert!(
        matches!(service.poll_ready(&mut cx), Poll::Ready(Ok(()))),
        "a ready inner service must make the middleware ready"
    );
}

#[tokio::test]
async fn poll_ready_surfaces_inner_readiness_failure() {
    let mut service = opentelemetry_tracing_layer().layer(NotReady);
    let mut cx = Context::from_waker(Waker::noop());

    match service.poll_ready(&mut cx) {
        Poll::Ready(Err(err)) => assert_eq!(err.kind(), io::ErrorKind::ConnectionRefused),
        other => panic!("expected the inner failure to be reported, got: {other:?}"),
    }
}

#[tokio::test]
async fn call_propagates_inner_service_error() {
    let mut service = opentelemetry_tracing_layer().layer(FailsOnCall);

    let req = Request::builder()
        .uri("/explode")
        .body(Body::empty())
        .unwrap();
    let err = service
        .call(req)
        .await
        .expect_err("the inner error must not be swallowed");

    assert_eq!(err.kind(), io::ErrorKind::BrokenPipe);
}

#[tokio::test]
async fn non_utf8_traceparent_header_degrades_to_placeholder() {
    let inner = tower::service_fn(|_req: Request<Body>| async {
        Ok::<_, Infallible>(
            Response::builder()
                .status(StatusCode::ACCEPTED)
                .body(Body::empty())
                .unwrap(),
        )
    });
    let mut service = opentelemetry_tracing_layer().layer(inner);

    let mut req = Request::builder()
        .uri("/propagated")
        .body(Body::empty())
        .unwrap();
    // Header values are byte strings; 0xFF/0xFE are not valid UTF-8, so
    // `HeaderValue::to_str()` fails and the middleware must fall back to "-".
    req.headers_mut().insert(
        "traceparent",
        HeaderValue::from_bytes(&[0xff, 0xfe]).unwrap(),
    );

    let resp = service
        .call(req)
        .await
        .expect("a non-UTF-8 traceparent must not fail the request");

    assert_eq!(resp.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn response_body_and_status_pass_through_untouched() {
    let inner = tower::service_fn(|_req: Request<Body>| async {
        Ok::<_, Infallible>(
            Response::builder()
                .status(StatusCode::CREATED)
                .header("x-inner", "kept")
                .body(Body::from("payload"))
                .unwrap(),
        )
    });
    let mut service = opentelemetry_tracing_layer().layer(inner);

    let req = Request::builder()
        .method("PATCH")
        .uri("/things/1")
        .body(Body::empty())
        .unwrap();
    let resp = service.call(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    assert_eq!(resp.headers().get("x-inner").unwrap(), "kept");
    let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    assert_eq!(&body[..], b"payload");
}
