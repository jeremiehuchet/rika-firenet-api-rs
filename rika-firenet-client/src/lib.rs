use std::sync::Arc;

use anyhow::Result;
use auth::RetryWithAuthMiddleware;
use lazy_static::lazy_static;
use log::debug;
use model::StatusDetail;
use nipper::Document;
use regex::Regex;
use reqwest::{redirect::Policy, Client};
use reqwest_middleware::{ClientBuilder, Middleware};
use rika_firenet_openapi::apis::{
    auth_api::{self, LoginParams, LogoutError},
    configuration::Configuration,
    stoves_api::{self, StoveControlsParams, StoveStatusParams},
};
pub use rika_firenet_openapi::models::StoveStatus;

mod auth;
pub mod model;

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

    pub async fn list_stoves(&self) -> Result<Vec<String>> {
        let stove_body: String = stoves_api::list_stoves(&self.configuration).await?;
        debug!("List stoves result: {stove_body}");
        let stove_ids = extract_stove_ids(&stove_body);
        debug!("Extracted stoves ids: {}", stove_ids.join(", "));
        Ok(stove_ids)
    }

    pub async fn status(&self, stove_id: String) -> Result<StoveStatus> {
        Ok(stoves_api::stove_status(&self.configuration, StoveStatusParams { stove_id }).await?)
    }

    pub async fn configure_mode(&self, stove_id: String, mode: Mode) -> Result<()> {
        let status = self.status(stove_id).await?;
        let mut params = build_control_params(status);
        match mode {
            Mode::Manual {
                heating_power_percent,
            } => {
                params.operating_mode = Some(0);
                params.heating_power = Some(heating_power_percent.into());
            }
            Mode::Auto {
                heating_power_percent,
            } => {
                params.operating_mode = Some(1);
                params.heating_power = Some(heating_power_percent.into());
            }
            Mode::Comfort { target_temperature } => {
                params.operating_mode = Some(2);
                params.target_temperature = Some(format!("{target_temperature}"));
            }
        }
        Ok(stoves_api::stove_controls(&self.configuration, params).await?)
    }

    pub async fn logout(&self) -> Result<()> {
        Ok(auth_api::logout(&self.configuration).await?)
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

pub enum Mode {
    Manual { heating_power_percent: u8 },
    Auto { heating_power_percent: u8 },
    Comfort { target_temperature: u8 },
}

fn build_control_params(status: StoveStatus) -> StoveControlsParams {
    StoveControlsParams {
        stove_id: status.stove_id,
        room_power_request: status.controls.room_power_request,
        bake_temperature: status.controls.bake_temperature,
        convection_fan1_active: status.controls.convection_fan1_active,
        convection_fan1_area: status.controls.convection_fan1_area,
        convection_fan1_level: status.controls.convection_fan1_level,
        convection_fan2_active: status.controls.convection_fan2_active,
        convection_fan2_area: status.controls.convection_fan2_area,
        convection_fan2_level: status.controls.convection_fan2_level,
        debug0: status.controls.debug0,
        debug1: status.controls.debug1,
        debug2: status.controls.debug2,
        debug3: status.controls.debug3,
        debug4: status.controls.debug4,
        eco_mode: status.controls.eco_mode,
        frost_protection_active: status.controls.frost_protection_active,
        frost_protection_temperature: status.controls.frost_protection_temperature,
        heating_power: status.controls.heating_power,
        heating_time_fri1: status.controls.heating_time_fri1,
        heating_time_fri2: status.controls.heating_time_fri2,
        heating_time_mon1: status.controls.heating_time_mon1,
        heating_time_mon2: status.controls.heating_time_mon2,
        heating_time_sat1: status.controls.heating_time_sat1,
        heating_time_sat2: status.controls.heating_time_sat2,
        heating_time_sun1: status.controls.heating_time_sun1,
        heating_time_sun2: status.controls.heating_time_sun2,
        heating_time_thu1: status.controls.heating_time_thu1,
        heating_time_thu2: status.controls.heating_time_thu2,
        heating_time_tue1: status.controls.heating_time_tue1,
        heating_time_tue2: status.controls.heating_time_tue2,
        heating_time_wed1: status.controls.heating_time_wed1,
        heating_time_wed2: status.controls.heating_time_wed2,
        heating_times_active_for_comfort: status.controls.heating_times_active_for_comfort,
        on_off: status.controls.on_off,
        operating_mode: status.controls.operating_mode,
        revision: Some(status.last_confirmed_revision),
        set_back_temperature: status.controls.set_back_temperature,
        target_temperature: status.controls.target_temperature,
        temperature_offset: status.controls.temperature_offset,
    }
}

pub trait HasDetailledStatus {
    fn get_status_details(&self) -> StatusDetail;
}

impl HasDetailledStatus for StoveStatus {
    fn get_status_details(&self) -> StatusDetail {
        let frost_started = self.sensors.status_frost_started;
        let main_state = self.sensors.status_main_state;
        let sub_state = self.sensors.status_sub_state;
        let bake_mode = self.controls.operating_mode.unwrap_or_default() == 3
            && "1024" != self.controls.bake_temperature.clone().unwrap_or_default()
            && "1024" != self.sensors.input_bake_temperature;
        let temp_diff = self
            .sensors
            .input_bake_temperature
            .parse::<i32>()
            .unwrap()
            .abs_diff(
                self.controls
                    .bake_temperature
                    .clone()
                    .unwrap_or_default()
                    .parse::<i32>()
                    .unwrap(),
            );

        if frost_started {
            return StatusDetail::FrostProtection;
        }
        if main_state == 1 {
            if sub_state == 0 {
                return StatusDetail::Off;
            } else if sub_state == 1 {
                return StatusDetail::Standby;
            } else if sub_state == 2 {
                return StatusDetail::ExternalRequest;
            } else if sub_state == 3 {
                return StatusDetail::Standby;
            }
            return StatusDetail::Unknown;
        } else if main_state == 2 {
            return StatusDetail::Ignition;
        } else if main_state == 3 {
            return StatusDetail::Startup;
        } else if main_state == 4 {
            if bake_mode && temp_diff < 10 {
                return StatusDetail::Bake;
            } else if bake_mode {
                return StatusDetail::Heat;
            } else {
                return StatusDetail::Control;
            }
        } else if main_state == 5 {
            if sub_state == 3 || sub_state == 4 {
                return StatusDetail::DeepCleaning;
            } else {
                return StatusDetail::Cleaning;
            }
        } else if main_state == 6 {
            return StatusDetail::Burnout;
        } else if main_state == 11
            || main_state == 13
            || main_state == 14
            || main_state == 16
            || main_state == 17
            || main_state == 50
        {
            return StatusDetail::WoodPresenceControl;
        } else if main_state == 20 || main_state == 21 {
            return StatusDetail::Wood;
        }
        return StatusDetail::Unknown;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        extract_stove_ids, model::StatusDetail, HasDetailledStatus, Mode, RikaFirenetClient,
    };
    use httpmock::{
        Method::{GET, POST},
        MockServer,
    };
    use regex::Regex;

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
    async fn can_get_stove_detailed_status() {
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

        assert_eq!(
            status.get_status_details(),
            StatusDetail::Standby,
            "stove status details"
        );
        status_mock.assert();
    }

    #[tokio::test]
    async fn can_set_stove_mode_to_manual() {
        let server = MockServer::start();
        let status_mock = server.mock(|when, then| {
            when.method(GET).path("/api/client/__stove_id__/status");
            then.status(200)
                .body_from_file("../mock/src/stove-status.json");
        });
        let control_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/client/__stove_id__/controls")
                .body_contains("&revision=1572181181&")
                .body_contains("&operatingMode=0&")
                .body_contains("&heatingPower=51&");
            then.status(200).body("OK");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .credentials("someone@rika.com", "Secret!")
            .build();

        let result = client
            .configure_mode(
                "__stove_id__".to_string(),
                Mode::Manual {
                    heating_power_percent: 51,
                },
            )
            .await;

        status_mock.assert();
        control_mock.assert();
        result.unwrap();
    }

    #[tokio::test]
    async fn can_set_stove_mode_to_automatic() {
        let server = MockServer::start();
        let status_mock = server.mock(|when, then| {
            when.method(GET).path("/api/client/__stove_id__/status");
            then.status(200)
                .body_from_file("../mock/src/stove-status.json");
        });
        let control_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/client/__stove_id__/controls")
                .body_contains("&revision=1572181181&")
                .body_contains("&operatingMode=1&")
                .body_contains("&heatingPower=52&");
            then.status(200).body("OK");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .credentials("someone@rika.com", "Secret!")
            .build();

        let result = client
            .configure_mode(
                "__stove_id__".to_string(),
                Mode::Auto {
                    heating_power_percent: 52,
                },
            )
            .await;

        status_mock.assert();
        control_mock.assert();
        result.unwrap();
    }

    #[tokio::test]
    async fn can_set_stove_mode_to_comfort() {
        let server = MockServer::start();
        let status_mock = server.mock(|when, then| {
            when.method(GET).path("/api/client/__stove_id__/status");
            then.status(200)
                .body_from_file("../mock/src/stove-status.json");
        });
        let control_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/client/__stove_id__/controls")
                .body_matches(Regex::new("^|&revision=1572181181&|$").unwrap())
                .body_matches(Regex::new("^|&operatingMode=2&|$").unwrap())
                .body_matches(Regex::new("^|&targetTemperature=19&|$").unwrap());
            then.status(200).body("OK");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .credentials("someone@rika.com", "Secret!")
            .build();

        let result = client
            .configure_mode(
                "__stove_id__".to_string(),
                Mode::Comfort {
                    target_temperature: 19,
                },
            )
            .await;

        status_mock.assert();
        control_mock.assert();
        result.unwrap();
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
