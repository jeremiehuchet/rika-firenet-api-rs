use std::sync::Arc;

use crate::api_internals::OverrideResponseContentTypeHeader;
use anyhow::{Result, ensure};
use api_internals::RetryWithAuthMiddleware;
use async_trait::async_trait;
use bon::bon;
use lazy_static::lazy_static;
use log::debug;
use model::{HeatingSchedule, OperatingMode, StatusDetail};
use nipper::Document;
use regex::Regex;
use reqwest::{Client, redirect::Policy};
use reqwest_middleware::{ClientBuilder, Middleware};
use rika_firenet_openapi::apis::auth_api::{AuthApi, AuthApiClient};
use rika_firenet_openapi::apis::stoves_api::{StovesApi, StovesApiClient};
use rika_firenet_openapi::apis::{
    configuration::Configuration,
    stoves_api::{StoveControlsParams, StoveStatusParams},
};
pub use rika_firenet_openapi::models::{StoveControls, StoveStatus};

mod api_internals;
pub mod model;

const API_BASE_URL: &str = "https://www.rika-firenet.com";
const FIREFOX_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:110.0) Gecko/20100101 Firefox/110.0";

lazy_static! {
    static ref PARSE_HEATTIME_REGEC: Regex =
            Regex::new("(?P<firstStartHH>\\d{2})(?P<firstStartMM>\\d{2})(?P<firstEndHH>\\d{2})(?P<firstEndMM>\\d{2})(?P<secondStartHH>\\d{2})(?P<secondStartMM>\\d{2})(?P<secondndHH>\\d{2})(?P<secondEndMM>\\d{2})").unwrap();
}

#[async_trait]
pub trait RikaFirenet {
    async fn list_stoves(&self) -> Result<Vec<String>>;
    async fn status<S: Into<String> + Send>(&self, stove_id: S) -> Result<StoveStatus>;
    async fn restore_controls<S: Into<String> + Send>(
        &self,
        stove_id: S,
        controls: StoveControls,
    ) -> Result<()>;
    async fn turn_on<S: Into<String> + Send>(&self, stove_id: S) -> Result<()>;

    async fn turn_off<S: Into<String> + Send>(&self, stove_id: S) -> Result<()>;
    async fn set_manual_mode<S: Into<String> + Send>(
        &self,
        stove_id: S,
        heating_power_percent: u8,
    ) -> Result<()>;
    async fn set_auto_mode<S: Into<String> + Send>(
        &self,
        stove_id: S,
        heating_power_percent: u8,
    ) -> Result<()>;
    async fn set_comfort_mode<S: Into<String> + Send>(
        &self,
        stove_id: S,
        idle_temperature: u8,
        target_temperature: u8,
    ) -> Result<()>;
    async fn enable_frost_protection<S: Into<String> + Send>(
        &self,
        stove_id: S,
        frost_protection_temperature: u8,
    ) -> Result<()>;
    async fn disable_frost_protection<S: Into<String> + Send>(&self, stove_id: S) -> Result<()>;
    async fn enable_schedule<S: Into<String> + Send>(
        &self,
        stove_id: S,
        schedule: HeatingSchedule,
    ) -> Result<()>;

    async fn logout(&self) -> Result<()>;
}

pub struct RikaFirenetClient {
    auth_api: Arc<dyn AuthApi>,
    stoves_api: Arc<dyn StovesApi>,
}

#[bon]
impl RikaFirenetClient {
    #[builder(on(String, into))]
    pub fn new(
        #[builder(finish_fn)] email: String,
        #[builder(finish_fn)] password: String,
        base_url: Option<String>,
        reqwest_middleware: Option<Arc<dyn Middleware>>,
    ) -> Self {
        let inner_client = Client::builder()
            .cookie_store(true)
            .redirect(Policy::none())
            .build()
            .expect("an http client");

        let base_config = Configuration {
            base_path: base_url
                .unwrap_or_else(|| API_BASE_URL.to_string())
                .trim_end_matches('/')
                .to_string(),
            user_agent: Some(FIREFOX_USER_AGENT.to_string()),
            ..Default::default()
        };

        let mut auth_client = ClientBuilder::new(inner_client.clone());
        if let Some(reqwest_middleware) = reqwest_middleware.as_ref() {
            auth_client = auth_client.with_arc(reqwest_middleware.clone());
        }
        let auth_api = Arc::new(AuthApiClient::new(
            Configuration {
                client: auth_client.build(),
                ..base_config.clone()
            }
            .into(),
        ));

        let mut stoves_client = ClientBuilder::new(inner_client)
            .with(OverrideResponseContentTypeHeader::new())
            .with(
                RetryWithAuthMiddleware::builder()
                    .api(auth_api.clone())
                    .build(email, password),
            );
        if let Some(prometheus_middleware) = reqwest_middleware {
            stoves_client = stoves_client.with_arc(prometheus_middleware);
        }
        Self {
            auth_api,
            stoves_api: Arc::new(StovesApiClient::new(
                Configuration {
                    client: stoves_client.build(),
                    ..base_config
                }
                .into(),
            )),
        }
    }
}

#[async_trait]
impl RikaFirenet for RikaFirenetClient {
    async fn list_stoves(&self) -> Result<Vec<String>> {
        let stove_body: String = self.stoves_api.list_stoves().await?;
        debug!("List stoves result: {stove_body}");
        let stove_ids = extract_stove_ids(&stove_body);
        debug!("Extracted stoves ids: {}", stove_ids.join(", "));
        Ok(stove_ids)
    }

    async fn status<S: Into<String> + Send>(&self, stove_id: S) -> Result<StoveStatus> {
        Ok(self
            .stoves_api
            .stove_status(StoveStatusParams {
                stove_id: stove_id.into(),
            })
            .await?)
    }

    async fn restore_controls<S: Into<String> + Send>(
        &self,
        stove_id: S,
        controls: StoveControls,
    ) -> Result<()> {
        let current_status = self.status(stove_id).await?;
        let restore_status = StoveStatus {
            controls,
            ..current_status
        };
        let params = restore_status.into_stove_controls();
        Ok(self.stoves_api.stove_controls(params).await?)
    }

    async fn turn_on<S: Into<String> + Send>(&self, stove_id: S) -> Result<()> {
        let params = StoveControlsParams {
            on_off: Some(true),
            ..self.status(stove_id).await?.into_stove_controls()
        };
        Ok(self.stoves_api.stove_controls(params).await?)
    }

    async fn turn_off<S: Into<String> + Send>(&self, stove_id: S) -> Result<()> {
        let params = StoveControlsParams {
            on_off: Some(false),
            ..self.status(stove_id).await?.into_stove_controls()
        };
        Ok(self.stoves_api.stove_controls(params).await?)
    }

    async fn set_manual_mode<S: Into<String> + Send>(
        &self,
        stove_id: S,
        heating_power_percent: u8,
    ) -> Result<()> {
        ensure!(
            (0..100).contains(&heating_power_percent),
            "Heating power must be 0 <= power <= 100 but it was {heating_power_percent}"
        );
        let params = StoveControlsParams {
            operating_mode: Some(OperatingMode::Manual.into()),
            heating_power: Some(heating_power_percent.into()),
            ..self.status(stove_id).await?.into_stove_controls()
        };
        Ok(self.stoves_api.stove_controls(params).await?)
    }

    async fn set_auto_mode<S: Into<String> + Send>(
        &self,
        stove_id: S,
        heating_power_percent: u8,
    ) -> Result<()> {
        ensure!(
            (0..100).contains(&heating_power_percent),
            "Heating power must be 0 <= power <= 100 but it was {heating_power_percent}"
        );
        let params = StoveControlsParams {
            operating_mode: Some(OperatingMode::Auto.into()),
            heating_power: Some(heating_power_percent.into()),
            ..self.status(stove_id).await?.into_stove_controls()
        };
        Ok(self.stoves_api.stove_controls(params).await?)
    }

    async fn set_comfort_mode<S: Into<String> + Send>(
        &self,
        stove_id: S,
        idle_temperature: u8,
        target_temperature: u8,
    ) -> Result<()> {
        ensure!(
            (12..21).contains(&idle_temperature),
            "Idle temperature must be 12 <= temp <= 20°C but it was {idle_temperature}"
        );
        ensure!(
            (14..29).contains(&target_temperature),
            "Target temperature must be 14 <= temp <= 28°C but it was {target_temperature}"
        );
        ensure!(
            idle_temperature < target_temperature,
            "Target temperature must be greater than idle temperature"
        );
        let params = StoveControlsParams {
            operating_mode: Some(OperatingMode::Comfort.into()),
            target_temperature: Some(target_temperature.to_string()),
            set_back_temperature: Some(idle_temperature.to_string()),
            ..self.status(stove_id).await?.into_stove_controls()
        };
        Ok(self.stoves_api.stove_controls(params).await?)
    }

    async fn enable_frost_protection<S: Into<String> + Send>(
        &self,
        stove_id: S,
        frost_protection_temperature: u8,
    ) -> Result<()> {
        ensure!(
            (4..10).contains(&frost_protection_temperature),
            "Frost protection temperature must be 4 <= temp <= 10°C but it was {frost_protection_temperature}"
        );
        let params = StoveControlsParams {
            frost_protection_active: Some(true),
            frost_protection_temperature: Some(frost_protection_temperature.to_string()),
            ..self.status(stove_id).await?.into_stove_controls()
        };
        Ok(self.stoves_api.stove_controls(params).await?)
    }

    async fn disable_frost_protection<S: Into<String> + Send>(&self, stove_id: S) -> Result<()> {
        let params = StoveControlsParams {
            frost_protection_active: Some(false),
            ..self.status(stove_id).await?.into_stove_controls()
        };
        Ok(self.stoves_api.stove_controls(params).await?)
    }

    async fn enable_schedule<S: Into<String> + Send>(
        &self,
        stove_id: S,
        schedule: HeatingSchedule,
    ) -> Result<()> {
        let params = StoveControlsParams {
            heating_times_active_for_comfort: Some(true),
            heating_time_mon1: Some(schedule.monday.first.into()),
            heating_time_mon2: Some(schedule.monday.second.into()),
            heating_time_tue1: Some(schedule.tuesday.first.into()),
            heating_time_tue2: Some(schedule.tuesday.second.into()),
            heating_time_wed1: Some(schedule.wednesday.first.into()),
            heating_time_wed2: Some(schedule.wednesday.second.into()),
            heating_time_thu1: Some(schedule.thursday.first.into()),
            heating_time_thu2: Some(schedule.thursday.second.into()),
            heating_time_fri1: Some(schedule.friday.first.into()),
            heating_time_fri2: Some(schedule.friday.second.into()),
            heating_time_sat1: Some(schedule.saturday.first.into()),
            heating_time_sat2: Some(schedule.saturday.second.into()),
            heating_time_sun1: Some(schedule.sunday.first.into()),
            heating_time_sun2: Some(schedule.sunday.second.into()),
            ..self.status(stove_id).await?.into_stove_controls()
        };
        println!("{params:?}");
        Ok(self.stoves_api.stove_controls(params).await?)
    }

    async fn logout(&self) -> Result<()> {
        Ok(self.auth_api.logout().await?)
    }
}

fn extract_stove_ids(body: &str) -> Vec<String> {
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

trait IntoStoveControlsParams {
    fn into_stove_controls(self) -> StoveControlsParams;
}

impl IntoStoveControlsParams for StoveStatus {
    fn into_stove_controls(self) -> StoveControlsParams {
        StoveControlsParams {
            stove_id: self.stove_id,
            room_power_request: self.controls.room_power_request,
            bake_temperature: self.controls.bake_temperature,
            convection_fan1_active: self.controls.convection_fan1_active,
            convection_fan1_area: self.controls.convection_fan1_area,
            convection_fan1_level: self.controls.convection_fan1_level,
            convection_fan2_active: self.controls.convection_fan2_active,
            convection_fan2_area: self.controls.convection_fan2_area,
            convection_fan2_level: self.controls.convection_fan2_level,
            debug0: self.controls.debug0,
            debug1: self.controls.debug1,
            debug2: self.controls.debug2,
            debug3: self.controls.debug3,
            debug4: self.controls.debug4,
            eco_mode: self.controls.eco_mode,
            frost_protection_active: self.controls.frost_protection_active,
            frost_protection_temperature: self.controls.frost_protection_temperature,
            heating_power: self.controls.heating_power,
            heating_time_fri1: self.controls.heating_time_fri1,
            heating_time_fri2: self.controls.heating_time_fri2,
            heating_time_mon1: self.controls.heating_time_mon1,
            heating_time_mon2: self.controls.heating_time_mon2,
            heating_time_sat1: self.controls.heating_time_sat1,
            heating_time_sat2: self.controls.heating_time_sat2,
            heating_time_sun1: self.controls.heating_time_sun1,
            heating_time_sun2: self.controls.heating_time_sun2,
            heating_time_thu1: self.controls.heating_time_thu1,
            heating_time_thu2: self.controls.heating_time_thu2,
            heating_time_tue1: self.controls.heating_time_tue1,
            heating_time_tue2: self.controls.heating_time_tue2,
            heating_time_wed1: self.controls.heating_time_wed1,
            heating_time_wed2: self.controls.heating_time_wed2,
            heating_times_active_for_comfort: self.controls.heating_times_active_for_comfort,
            on_off: self.controls.on_off,
            operating_mode: self.controls.operating_mode,
            revision: Some(self.last_confirmed_revision),
            set_back_temperature: self.controls.set_back_temperature,
            target_temperature: self.controls.target_temperature,
            temperature_offset: self.controls.temperature_offset,
        }
    }
}

pub trait HasDetailledStatus {
    fn get_status_details(&self) -> StatusDetail;
    fn get_heating_schedule(&self) -> HeatingSchedule;
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

        // based on rika-firenet.com algorithm:
        //     if (frostStarted) {
        //         return "Frost protection";
        //     }
        //     if (mainState == 1) {
        //         if (subState == 0)
        //             return "Stove off";
        //         else if (subState == 1)
        //             return "Standby";
        //         else if (subState == 2)
        //             return "External Request";
        //         else if (subState == 3)
        //             return "Standby";
        //         return "Substate Unknown";
        //     } else if (mainState == 2)
        //         return "Ignition on";
        //     else if (mainState == 3)
        //         return "Starting up";
        //     else if (mainState == 4) {
        //         if (bakeMode && tempDiff < 10)
        //             return "Baking";
        //         else if (bakeMode)
        //             return "Heating up";
        //         else
        //             return "Running";
        //     } else if (mainState == 5) {
        //         if (subState == 3 || subState == 4)
        //             return "Big Clean";
        //         else
        //             return "Clean";
        //     } else if (mainState == 6)
        //         return "Burn off";
        //     else if (mainState == 11 || mainState == 13 || mainState == 14 || mainState == 16 || mainState == 17 || mainState == 50)
        //         return "Split log check";
        //     else if (mainState == 20 || mainState == 21)
        //         return "Split log mode";
        //     return "Unknown";
        // }
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
                return StatusDetail::Baking;
            } else if bake_mode {
                return StatusDetail::HeatingUp;
            } else {
                return StatusDetail::Running;
            }
        } else if main_state == 5 {
            if sub_state == 3 || sub_state == 4 {
                return StatusDetail::DeepCleaning;
            } else {
                return StatusDetail::Cleaning;
            }
        } else if main_state == 6 {
            return StatusDetail::BurnOff;
        } else if main_state == 11
            || main_state == 13
            || main_state == 14
            || main_state == 16
            || main_state == 17
            || main_state == 50
        {
            return StatusDetail::SplitLogCheck;
        } else if main_state == 20 || main_state == 21 {
            return StatusDetail::SplitLogMode;
        }
        return StatusDetail::Unknown;
    }

    fn get_heating_schedule(&self) -> HeatingSchedule {
        HeatingSchedule::from(self.controls.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        HasDetailledStatus, RikaFirenet, RikaFirenetClient, extract_stove_ids,
        model::{DailySchedule, HeatingSchedule, StatusDetail},
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
            .build("someone@rika.com", "Secret!");

        let stoves = client.list_stoves().await.expect("a successful operation");

        assert_eq!(stoves, vec!["12345", "333444"], "expect 2 stoves ids");
        summary_mock.assert();
    }

    #[tokio::test]
    async fn can_get_stove_status() {
        let server = MockServer::start();
        let status_mock = server.mock(|when, then| {
            when.method(GET).path("/api/client/12345/status");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("../mock/src/stove-status.json");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .build("someone@rika.com", "Secret!");

        let status = client
            .status("12345")
            .await
            .expect("a successful operation");

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
                .header("content-type", "application/json")
                .body_from_file("../mock/src/stove-status.json");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .build("someone@rika.com", "Secret!");

        let status = client
            .status("12345")
            .await
            .expect("a successful operation");

        assert_eq!(
            status.get_status_details(),
            StatusDetail::Standby,
            "stove status details"
        );
        status_mock.assert();
    }

    #[tokio::test]
    async fn can_turn_on_stove() {
        let server = MockServer::start();
        let status_mock = server.mock(|when, then| {
            when.method(GET).path("/api/client/__stove_id__/status");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("../mock/src/stove-status.json");
        });
        let control_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/client/__stove_id__/controls")
                .body_matches(Regex::new("(^|&)revision=1572181181(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)onOff=true(&|$)").unwrap());
            then.status(200).body("OK");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .build("someone@rika.com", "Secret!");

        client
            .turn_on("__stove_id__")
            .await
            .expect("a successful operation");

        status_mock.assert();
        control_mock.assert();
    }

    #[tokio::test]
    async fn can_turn_off_stove() {
        let server = MockServer::start();
        let status_mock = server.mock(|when, then| {
            when.method(GET).path("/api/client/__stove_id__/status");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("../mock/src/stove-status.json");
        });
        let control_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/client/__stove_id__/controls")
                .body_matches(Regex::new("(^|&)revision=1572181181(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)onOff=false(&|$)").unwrap());
            then.status(200).body("OK");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .build("someone@rika.com", "Secret!");

        client
            .turn_off("__stove_id__")
            .await
            .expect("a successful operation");

        status_mock.assert();
        control_mock.assert();
    }

    #[tokio::test]
    async fn can_set_stove_mode_to_manual() {
        let server = MockServer::start();
        let status_mock = server.mock(|when, then| {
            when.method(GET).path("/api/client/__stove_id__/status");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("../mock/src/stove-status.json");
        });
        let control_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/client/__stove_id__/controls")
                .body_matches(Regex::new("(^|&)revision=1572181181(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)operatingMode=0(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingPower=51(&|$)").unwrap());
            then.status(200).body("OK");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .build("someone@rika.com", "Secret!");

        client
            .set_manual_mode("__stove_id__", 51)
            .await
            .expect("a successful operation");

        status_mock.assert();
        control_mock.assert();
    }

    #[tokio::test]
    async fn cant_set_stove_mode_to_manual_with_an_invalid_power_heating_value() {
        let client = RikaFirenetClient::builder()
            .base_url("http://localhost")
            .build("someone@rika.com", "Secret!");

        let error = client
            .set_manual_mode("__stove_id__", 101)
            .await
            .unwrap_err();
        let root_cause = error.root_cause();
        assert_eq!(
            format!("{root_cause}"),
            "Heating power must be 0 <= power <= 100 but it was 101"
        );
    }

    #[tokio::test]
    async fn can_set_stove_mode_to_automatic() {
        let server = MockServer::start();
        let status_mock = server.mock(|when, then| {
            when.method(GET).path("/api/client/__stove_id__/status");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("../mock/src/stove-status.json");
        });
        let control_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/client/__stove_id__/controls")
                .body_matches(Regex::new("(^|&)revision=1572181181(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)operatingMode=1(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingPower=52(&|$)").unwrap());
            then.status(200).body("OK");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .build("someone@rika.com", "Secret!");

        client
            .set_auto_mode("__stove_id__", 52)
            .await
            .expect("a successful operation");

        status_mock.assert();
        control_mock.assert();
    }

    #[tokio::test]
    async fn cant_set_stove_mode_to_auto_with_an_invalid_power_heating_value() {
        let client = RikaFirenetClient::builder()
            .base_url("http://localhost")
            .build("someone@rika.com", "Secret!");

        let error = client.set_auto_mode("__stove_id__", 101).await.unwrap_err();
        let root_cause = error.root_cause();
        assert_eq!(
            format!("{root_cause}"),
            "Heating power must be 0 <= power <= 100 but it was 101"
        );
    }

    #[tokio::test]
    async fn can_set_stove_mode_to_comfort() {
        let server = MockServer::start();
        let status_mock = server.mock(|when, then| {
            when.method(GET).path("/api/client/__stove_id__/status");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("../mock/src/stove-status.json");
        });
        let control_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/client/__stove_id__/controls")
                .body_matches(Regex::new("(^|&)revision=1572181181(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)operatingMode=2(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)setBackTemperature=17(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)targetTemperature=19(&|$)").unwrap());
            then.status(200).body("OK");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .build("someone@rika.com", "Secret!");

        client
            .set_comfort_mode("__stove_id__", 17, 19)
            .await
            .expect("a successful operation");

        status_mock.assert();
        control_mock.assert();
    }

    #[tokio::test]
    async fn cant_set_stove_mode_to_comfort_with_an_invalid_target_or_idle_temperature_value() {
        let client = RikaFirenetClient::builder()
            .base_url("http://localhost")
            .build("someone@rika.com", "Secret!");

        let error = client
            .set_comfort_mode("__stove_id__", 12, 13)
            .await
            .unwrap_err();
        let root_cause = error.root_cause();
        assert_eq!(
            format!("{root_cause}"),
            "Target temperature must be 14 <= temp <= 28°C but it was 13"
        );

        let error = client
            .set_comfort_mode("__stove_id__", 20, 29)
            .await
            .unwrap_err();
        let root_cause = error.root_cause();
        assert_eq!(
            format!("{root_cause}"),
            "Target temperature must be 14 <= temp <= 28°C but it was 29"
        );

        let error = client
            .set_comfort_mode("__stove_id__", 21, 28)
            .await
            .unwrap_err();
        let root_cause = error.root_cause();
        assert_eq!(
            format!("{root_cause}"),
            "Idle temperature must be 12 <= temp <= 20°C but it was 21"
        );

        let error = client
            .set_comfort_mode("__stove_id__", 11, 22)
            .await
            .unwrap_err();
        let root_cause = error.root_cause();
        assert_eq!(
            format!("{root_cause}"),
            "Idle temperature must be 12 <= temp <= 20°C but it was 11"
        );

        let error = client
            .set_comfort_mode("__stove_id__", 19, 17)
            .await
            .unwrap_err();
        let root_cause = error.root_cause();
        assert_eq!(
            format!("{root_cause}"),
            "Target temperature must be greater than idle temperature"
        );
    }

    #[tokio::test]
    async fn can_configure_schedule() {
        let server = MockServer::start();
        let status_mock = server.mock(|when, then| {
            when.method(GET).path("/api/client/__stove_id__/status");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("../mock/src/stove-status.json");
        });
        let control_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/client/__stove_id__/controls")
                .body_matches(Regex::new("(^|&)heatingTimeMon1=06300900(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingTimeMon2=18152245(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingTimeTue1=06300900(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingTimeTue2=18152245(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingTimeWed1=06300900(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingTimeWed2=18152245(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingTimeThu1=06300900(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingTimeThu2=18152245(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingTimeFri1=06300900(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingTimeFri2=18152245(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingTimeSat1=10002230(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingTimeSat2=00000000(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingTimeSun1=10002230(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)heatingTimeSun2=00000000(&|$)").unwrap());
            then.status(200).body("OK");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .build("someone@rika.com", "Secret!");

        let schedule = HeatingSchedule::week_vs_end_days(
            DailySchedule::dual("06300900".parse().unwrap(), "18152245".parse().unwrap()),
            DailySchedule::single("10002230".parse().unwrap()),
        );
        client
            .enable_schedule("__stove_id__", schedule)
            .await
            .expect("a successful operation");

        status_mock.assert();
        control_mock.assert();
    }

    #[tokio::test]
    async fn can_enable_frost_mode() {
        let server = MockServer::start();
        let status_mock = server.mock(|when, then| {
            when.method(GET).path("/api/client/__stove_id__/status");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("../mock/src/stove-status.json");
        });
        let control_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/client/__stove_id__/controls")
                .body_matches(Regex::new("(^|&)revision=1572181181(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)frostProtectionActive=true(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)frostProtectionTemperature=8(&|$)").unwrap());
            then.status(200).body("OK");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .build("someone@rika.com", "Secret!");

        client
            .enable_frost_protection("__stove_id__", 8)
            .await
            .expect("a successful operation");

        status_mock.assert();
        control_mock.assert();
    }

    #[tokio::test]
    async fn cant_enable_frost_protection_with_an_invalid_temperature_value() {
        let client = RikaFirenetClient::builder()
            .base_url("http://localhost")
            .build("someone@rika.com", "Secret!");

        let error = client
            .enable_frost_protection("__stove_id__", 3)
            .await
            .unwrap_err();
        let root_cause = error.root_cause();
        assert_eq!(
            format!("{root_cause}"),
            "Frost protection temperature must be 4 <= temp <= 10°C but it was 3"
        );

        let error = client
            .enable_frost_protection("__stove_id__", 11)
            .await
            .unwrap_err();
        let root_cause = error.root_cause();
        assert_eq!(
            format!("{root_cause}"),
            "Frost protection temperature must be 4 <= temp <= 10°C but it was 11"
        );
    }

    #[tokio::test]
    async fn can_disable_frost_mode() {
        let server = MockServer::start();
        let status_mock = server.mock(|when, then| {
            when.method(GET).path("/api/client/__stove_id__/status");
            then.status(200)
                .header("content-type", "application/json")
                .body_from_file("../mock/src/stove-status.json");
        });
        let control_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/client/__stove_id__/controls")
                .body_matches(Regex::new("(^|&)revision=1572181181(&|$)").unwrap())
                .body_matches(Regex::new("(^|&)frostProtectionActive=false(&|$)").unwrap());
            then.status(200).body("OK");
        });

        let client = RikaFirenetClient::builder()
            .base_url(server.base_url())
            .build("someone@rika.com", "Secret!");

        client
            .disable_frost_protection("__stove_id__")
            .await
            .expect("a successful operation");

        status_mock.assert();
        control_mock.assert();
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
            .build("someone@rika.com", "Secret!");

        client.logout().await.expect("a successful operation");

        logout_mock.assert();
    }

    #[tokio::test]
    async fn should_remove_trailing_slashes_from_base_url() {
        let server = MockServer::start();
        let logout_mock = server.mock(|when, then| {
            when.method(GET).path("/web/logout");
            then.status(418);
        });

        let client = RikaFirenetClient::builder()
            .base_url(format!("{}///", server.base_url()))
            .build("someone@rika.com", "Secret!");

        client
            .logout()
            .await
            .expect_err("error in response: status code 418 I'm a teapot");

        logout_mock.assert();
    }
}
