use axum::{
    body::Body,
    http::{self, request, HeaderMap, HeaderName, HeaderValue, Request, Response, StatusCode},
    Router,
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
    request: request::Builder,
}

impl<'app> RequestBuilder<'app> {
    pub fn new(router: &'app mut Router) -> Self {
        RequestBuilder {
            router,
            expected_status: StatusCode::OK,
            request: Request::builder(),
        }
    }

    pub fn expect_status(mut self, expected: StatusCode) -> Self {
        self.expected_status = expected;
        self
    }

    pub fn header<V>(mut self, key: HeaderName, val: V) -> Self
    where
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.request = self.request.header(key, val);
        self
    }

    pub async fn post<Input>(mut self, url: &str, input: &Input) -> TestResponse
    where
        Input: Serialize,
    {
        let request = self
            .request
            .method(http::Method::POST)
            .uri(url)
            .header(
                http::header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.as_ref(),
            )
            .body(serde_qs::to_string(input).unwrap())
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut self.router)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        Self::assert_expected_status(self.expected_status, &response, "GET", url);

        TestResponse { response }
    }

    pub async fn get(mut self, url: &str) -> TestResponse {
        let request = self.request.uri(url).body(Body::empty()).unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut self.router)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        Self::assert_expected_status(self.expected_status, &response, "GET", url);
        TestResponse { response }
    }

    fn assert_expected_status(
        expected_status: StatusCode,
        response: &Response<Body>,
        method: &str,
        url: &str,
    ) {
        assert_eq!(
            response.status(),
            expected_status,
            "expected {expected_status}: {method} {url}"
        );
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
