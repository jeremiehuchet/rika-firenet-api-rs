use async_trait::async_trait;
use bon::bon;
use http::header::CONTENT_TYPE;
use http::{Extensions, HeaderValue};
use log::debug;
use reqwest::{Request, Response};
use reqwest_middleware::{Error, Middleware, Next, Result};
use rika_firenet_openapi::apis::auth_api::LoginParams;
use rika_firenet_openapi::apis::auth_api::{AuthApi, AuthApiClient};
use std::sync::Arc;

pub(crate) struct RetryWithAuthMiddleware {
    auth_api: Arc<dyn AuthApi>,
    rika_credentials: LoginParams,
}

#[bon]
impl RetryWithAuthMiddleware {
    #[builder(on(String, into))]
    pub(crate) fn new(
        #[builder(finish_fn)] email: String,
        #[builder(finish_fn)] password: String,
        api: Arc<AuthApiClient>,
    ) -> RetryWithAuthMiddleware {
        RetryWithAuthMiddleware {
            auth_api: api.clone(),
            rika_credentials: LoginParams { email, password },
        }
    }
}

#[async_trait]
impl Middleware for RetryWithAuthMiddleware {
    async fn handle(
        &self,
        request: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let initial_result = next
            .clone()
            .run(
                request
                    .try_clone()
                    .expect("request shouldn't have a stream body"),
                extensions,
            )
            .await?;

        if !is_login_or_logout_request(&request) && is_login_redirection(&initial_result) {
            debug!("Login redirect response detected");
            return match self.auth_api.login(self.rika_credentials.clone()).await {
                Ok(()) => {
                    debug!("Retrying original request");
                    next.run(request, extensions).await
                }
                Err(e) => Err(Error::Middleware(anyhow::Error::new(e))),
            };
        }

        return Ok(initial_result);
    }
}

fn is_login_or_logout_request(request: &Request) -> bool {
    match request.url().path() {
        "/web/login" => true,
        "/web/logout" => true,
        _ => false,
    }
}

fn is_login_redirection(response: &Response) -> bool {
    if response.status() == 401 {
        return true;
    }
    if response.status() != 302 {
        return false;
    }
    let location = response
        .headers()
        .get("Location")
        .and_then(|header| header.to_str().ok());
    match location {
        Some("/web/") => true,
        Some("/web/login") => true,
        Some("401") => true,
        _ => false,
    }
}

pub(crate) struct OverrideResponseContentTypeHeader {}

impl OverrideResponseContentTypeHeader {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
#[async_trait]
impl Middleware for OverrideResponseContentTypeHeader {
    async fn handle(
        &self,
        request: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let mut response = next
            .clone()
            .run(
                request
                    .try_clone()
                    .expect("request shouldn't have a stream body"),
                extensions,
            )
            .await?;
        let headers = response.headers_mut();
        let content_type = headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok());
        match content_type {
            Some(content_type) => {
                if !content_type.starts_with("application") || !content_type.contains("json") {
                    headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
                }
            }
            None => {
                headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
            }
        }
        return Ok(response);
    }
}
