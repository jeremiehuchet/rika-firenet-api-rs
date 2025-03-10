use async_trait::async_trait;
use http::Extensions;
use log::debug;
use reqwest::{Request, Response};
use reqwest_middleware::{Error, Middleware, Next, Result};
use rika_firenet_openapi::apis::auth_api::LoginParams;
use rika_firenet_openapi::apis::auth_api::{AuthApi, AuthApiClient};
use std::sync::Arc;

pub(crate) struct RetryWithAuthMiddleware {
    auth_api: Arc<AuthApiClient>,
    rika_credentials: LoginParams,
}
impl RetryWithAuthMiddleware {
    pub(crate) fn new(
        api: Arc<AuthApiClient>,
        credentials: LoginParams,
    ) -> RetryWithAuthMiddleware {
        RetryWithAuthMiddleware {
            auth_api: api.clone(),
            rika_credentials: credentials,
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
