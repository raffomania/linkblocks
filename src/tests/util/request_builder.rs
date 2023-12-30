use askama_axum::IntoResponse;
use axum::{
    body::Body,
    http::{self, HeaderMap, Request, Response, StatusCode},
    Form, Router,
};
use http_body_util::BodyExt;
use mime_guess::mime;
use serde::Serialize;
use tower::{Service, ServiceExt};
use visdom::Vis;

pub struct RequestBuilder<'app> {
    router: &'app mut axum::Router,
    /// This is the HTTP status that we expect the backend to return.
    /// If it returns a different status, we'll panic.
    expected_status: StatusCode,
}

impl<'app> RequestBuilder<'app> {
    pub fn new(router: &'app mut Router) -> Self {
        RequestBuilder {
            router: router,
            expected_status: StatusCode::OK,
        }
    }

    pub fn expect_status(mut self, expected: StatusCode) -> Self {
        self.expected_status = expected;
        self
    }

    pub async fn post<Input>(mut self, url: &str, input: &Input) -> TestResponse
    where
        Input: Serialize,
    {
        let request = Request::builder()
            .method(http::Method::POST)
            .uri(url)
            .header(
                http::header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
            )
            .body(Form(input).into_response().into_body())
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut self.router)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        assert_eq!(response.status(), self.expected_status);

        TestResponse { response }
    }

    pub async fn get(mut self, url: &str) -> TestResponse {
        let request = Request::builder().uri(url).body(Body::empty()).unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut self.router)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        assert_eq!(response.status(), self.expected_status);
        TestResponse { response: response }
    }
}

pub struct TestResponse {
    response: Response<Body>,
}

impl TestResponse {
    pub async fn dom(self) -> visdom::types::Elements<'static> {
        let body = self
            .response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec();
        Vis::load(String::from_utf8(body).unwrap()).unwrap()
    }

    pub fn headers(&self) -> &HeaderMap {
        self.response.headers()
    }
}
