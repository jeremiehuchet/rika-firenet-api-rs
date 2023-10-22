use std::sync::Arc;

use auth::RetryWithAuthMiddleware;
use lazy_static::lazy_static;
use log::debug;
use nipper::Document;
use regex::Regex;
use reqwest::{redirect::Policy, Client};
use reqwest_middleware::{ClientBuilder, Middleware};
use rika_firenet_openapi::apis::{
    auth_api::{self, LoginParams, LogoutError},
    configuration::Configuration,
    stoves_api::{self, ListStovesError, StoveStatusError, StoveStatusParams},
    Error as RikaError,
};
pub use rika_firenet_openapi::models::StoveStatus;

mod auth;
mod model;

const API_BASE_URL: &str = "https://www.rika-firenet.com";
const FIREFOX_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:110.0) Gecko/20100101 Firefox/110.0";

lazy_static! {
    static ref PARSE_HEATTIME_REGEC: Regex =
            Regex::new("(?P<firstStartHH>\\d{2})(?P<firstStartMM>\\d{2})(?P<firstEndHH>\\d{2})(?P<firstEndMM>\\d{2})(?P<secondStartHH>\\d{2})(?P<secondStartMM>\\d{2})(?P<secondndHH>\\d{2})(?P<secondEndMM>\\d{2})").unwrap();
}

#[derive(Default)]
pub struct RikaFirenetClientBuilder {
    base_url: Option<String>,
    credentials: Option<LoginParams>,
    reqwest_middleware: Option<Arc<dyn Middleware>>,
}

impl RikaFirenetClientBuilder {
    pub fn base_url<S: Into<String>>(mut self, base_url: S) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn credentials<S: Into<String>>(mut self, username: S, password: S) -> Self {
        self.credentials = Some(LoginParams {
            email: username.into(),
            password: password.into(),
        });
        self
    }

    pub fn enable_metrics(mut self, reqwest_middleware: Arc<dyn Middleware>) -> Self {
        self.reqwest_middleware = Some(reqwest_middleware);
        self
    }

    pub fn build(self) -> RikaFirenetClient {
        let inner_client = Client::builder()
            .cookie_store(true)
            .redirect(Policy::none())
            .build()
            .expect("an http client");

        let api_configuration = Configuration {
            base_path: self.base_url.unwrap_or(API_BASE_URL.to_string()),
            user_agent: Some(FIREFOX_USER_AGENT.to_string()),
            ..Default::default()
        };

        let mut login_client = ClientBuilder::new(inner_client.clone());
        if let Some(reqwest_middleware) = self.reqwest_middleware.as_ref() {
            login_client = login_client.with_arc(reqwest_middleware.clone());
        }

        let login_middleware = RetryWithAuthMiddleware::new(
            Configuration {
                client: login_client.build(),
                ..api_configuration.clone()
            },
            self.credentials
                .expect("API can't be used without credentials"),
        );

        let mut api_client = ClientBuilder::new(inner_client).with(login_middleware);
        if let Some(prometheus_middleware) = self.reqwest_middleware.as_ref() {
            api_client = api_client.with_arc(prometheus_middleware.clone());
        }
        RikaFirenetClient {
            configuration: Configuration {
                client: api_client.build(),
                ..api_configuration
            },
        }
    }
}

#[derive(Clone)]
pub struct RikaFirenetClient {
    configuration: Configuration,
}

impl RikaFirenetClient {
    pub fn builder() -> RikaFirenetClientBuilder {
        RikaFirenetClientBuilder::default()
    }

    pub async fn list_stoves(&self) -> Result<Vec<String>, RikaError<ListStovesError>> {
        let stove_body: String = stoves_api::list_stoves(&self.configuration).await?;
        debug!("List stoves result: {stove_body}");
        let stove_ids = extract_stove_ids(&stove_body);
        debug!("Extracted stoves ids: {}", stove_ids.join(", "));
        Ok(stove_ids)
    }

    pub async fn status(
        &self,
        stove_id: String,
    ) -> Result<StoveStatus, RikaError<StoveStatusError>> {
        stoves_api::stove_status(&self.configuration, StoveStatusParams { stove_id }).await
    }

    pub async fn logout(&self) -> Result<(), RikaError<LogoutError>> {
        auth_api::logout(&self.configuration).await
    }
}

fn extract_stove_ids(body: &String) -> Vec<String> {
    let document = Document::from(body);
    let links = document.select("ul#stoveList li a");
    links
        .iter()
        .filter_map(|link| link.attr("href"))
        .map(|href| href.to_string())
        .filter_map(|href| {
            href.strip_prefix("/web/stove/")
                .map(|stove_id| stove_id.to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{extract_stove_ids, RikaFirenetClient};
    use httpmock::{Method::GET, MockServer};

    const PARTIAL_SUMMARY_EXAMPLE: &str = r#"
    <div role="main" class="ui-content">
        <div data-role="controlgroup">
            <h3>You have access to the following stoves</h3>
            <ul id="stoveList" data-role="listview" data-inset="true" data-theme="a" data-split-theme="a"
                data-split-icon="fa-pencil">
                <li class="ui-li-has-alt ui-first-child">
                    <a href="/web/stove/12345" data-ajax="false" class="ui-btn ui-first-child">Stove n°12345</a>
                    <a href="/web/edit/12345" data-ajax="false" class="ui-btn ui-btn-icon-notext ui-icon-fa-pencil ui-btn-a ui-last-child" title=""></a>
                </li>
                <li class="ui-li-has-alt ui-last-child">
                    <a href="/web/stove/333444" data-ajax="false" class="ui-btn ui-first-child">Stove n°333444</a>
                    <a href="/web/edit/333444" data-ajax="false" class="ui-btn ui-btn-icon-notext ui-icon-fa-pencil ui-btn-a ui-last-child" title=""></a>
                </li>
            </ul>
        </div>
    </div>
    "#;

    #[test]
    fn can_extract_stove_ids() {
        let actual = extract_stove_ids(&PARTIAL_SUMMARY_EXAMPLE.to_string());
        let expected = vec!["12345".to_string(), "333444".to_string()];
        assert_eq!(actual, expected)
    }

    #[tokio::test]
    async fn can_list_stoves() {
        let server = MockServer::start();
        let summary_mock = server.mock(|when, then| {
            when.method(GET).path("/web/summary");
            then.status(200).body_from_file("../mock/src/summary.html");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .credentials("someone@rika.com", "Secret!")
            .build();

        let stoves = client.list_stoves().await.unwrap();

        assert_eq!(stoves, vec!["12345", "333444"], "expect 2 stoves ids");
        summary_mock.assert();
    }

    #[tokio::test]
    async fn can_get_stove_status() {
        let server = MockServer::start();
        let status_mock = server.mock(|when, then| {
            when.method(GET).path("/api/client/12345/status");
            then.status(200)
                .body_from_file("../mock/src/stove-status.json");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .credentials("someone@rika.com", "Secret!")
            .build();

        let status = client.status("12345".to_string()).await.unwrap();

        assert_eq!(status.stove_id, "__stove_id__", "stove id");
        assert_eq!(status.name, "Stove __stove_id__", "stove name");
        assert_eq!(
            status.sensors.input_room_temperature, "19.6",
            "sensor value"
        );
        status_mock.assert();
    }

    #[tokio::test]
    async fn can_logout() {
        let server = MockServer::start();
        let logout_mock = server.mock(|when, then| {
            when.method(GET).path("/web/logout");
            then.status(302).header(
                "Set-Cookie",
                "connect.sid=xxx.xxx; Path=/; Expires=Fri, 10 Mar 2063 15:14:41 GMT; HttpOnly",
            );
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .credentials("someone@rika.com", "Secret!")
            .build();

        client.logout().await.unwrap();

        logout_mock.assert();
    }
}
