use std::{num::ParseIntError, str::FromStr};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::ClientBuilder;
use rika_firenet_openapi::{
    apis::{
        Error as RikaError,
        configuration::Configuration,
        stove_api::{self, ListStovesError, ListStovesParams, LoginParams, StoveStatusParams, StoveStatusError, LogoutError, LogoutParams, LoginError},
    },
};
pub use rika_firenet_openapi::models::StoveStatus;

const API_BASE_URL: &str = "https://www.rika-firenet.com";
const FIREFOX_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:110.0) Gecko/20100101 Firefox/110.0";

lazy_static! {
    static ref STOVELIST_REGEX: Regex =
        Regex::new(r#"href="/web/stove/(?P<stoveId>[^"]+)""#).unwrap();

    static ref PARSE_HEATTIME_REGEC: Regex =
            Regex::new("(?P<firstStartHH>\\d{2})(?P<firstStartMM>\\d{2})(?P<firstEndHH>\\d{2})(?P<firstEndMM>\\d{2})(?P<secondStartHH>\\d{2})(?P<secondStartMM>\\d{2})(?P<secondndHH>\\d{2})(?P<secondEndMM>\\d{2})").unwrap();
}

#[derive(Clone)]
pub struct RikaFirenetClient {
    configuration: Configuration,
}

impl RikaFirenetClient {
    pub fn new() -> Self {
        Self::new_with_base_url(API_BASE_URL.to_string())
    }

    pub fn new_with_base_url(base_url: String) -> Self {
        RikaFirenetClient {
            configuration: Configuration {
                base_path: base_url,
                user_agent: Some(FIREFOX_USER_AGENT.to_string()),
                client: ClientBuilder::new()
                    .cookie_store(true)
                    .build()
                    .expect("an http client"),
                ..Default::default()
            },
        }
    }

    pub async fn login(&self, username: String, password: String) -> Result<(), RikaError<LoginError>> {
        stove_api::login(
            &self.configuration,
            LoginParams {
                email: username,
                password,
            },
        )
        .await
    }

    pub async fn list_stoves(&self) -> Result<Vec<String>, RikaError<ListStovesError>> {
        let stove_body =
            stove_api::list_stoves(&self.configuration, ListStovesParams::default()).await?;
        let stoves = STOVELIST_REGEX
            .captures_iter(&stove_body)
            .map(|caps| caps["stoveId"].to_string())
            .collect();
        Ok(stoves)
    }

    pub async fn status(&self, stove_id: String) -> Result<StoveStatus, RikaError<StoveStatusError>> {
        stove_api::stove_status(
            &self.configuration,
            StoveStatusParams {
                connect_period_sid: String::new(),
                stove_id,
            },
        )
        .await
    }

    pub async fn logout(&self) -> Result<(), RikaError<LogoutError>> {
        stove_api::logout(&self.configuration, LogoutParams::default()).await
    }
}

pub enum StoveControl {
    OperatingMode(OperatingMode),
    HeatingPower(i32),
    EnableHeatingSchedule(bool),
    HeatingSchedule(HeatingSchedule),
}

pub enum OperatingMode {
    Manual,

    Auto,

    Comfort,
}

impl Into<i32> for OperatingMode {
    fn into(self) -> i32 {
        match self {
            OperatingMode::Manual => 0,
            OperatingMode::Auto => 1,
            OperatingMode::Comfort => 2,
        }
    }
}

impl OperatingMode {
    fn parse(mode: i32) -> Result<Self, String> {
        match mode {
            0 => Ok(OperatingMode::Manual),
            1 => Ok(OperatingMode::Auto),
            2 => Ok(OperatingMode::Comfort),
            num => Err(format!("{num} is not a valid OperatingMode")),
        }
    }
}

pub struct HeatingSchedule {
    monday: DailySchedule,
    tuesday: DailySchedule,
    wednesday: DailySchedule,
    thursday: DailySchedule,
    friday: DailySchedule,
    saturday: DailySchedule,
    sunday: DailySchedule,
}

impl HeatingSchedule {
    fn from(status: StoveStatus) -> Result<Self, ParseIntError> {
        Ok(HeatingSchedule {
            monday: DailySchedule::from(
                status.controls.heating_time_mon1,
                status.controls.heating_time_mon2,
            )?,
            tuesday: DailySchedule::from(
                status.controls.heating_time_thu1,
                status.controls.heating_time_thu2,
            )?,
            wednesday: DailySchedule::from(
                status.controls.heating_time_wed1,
                status.controls.heating_time_wed2,
            )?,
            thursday: DailySchedule::from(
                status.controls.heating_time_tue1,
                status.controls.heating_time_tue2,
            )?,
            friday: DailySchedule::from(
                status.controls.heating_time_fri1,
                status.controls.heating_time_fri2,
            )?,
            saturday: DailySchedule::from(
                status.controls.heating_time_sat1,
                status.controls.heating_time_sat2,
            )?,
            sunday: DailySchedule::from(
                status.controls.heating_time_sun1,
                status.controls.heating_time_sun2,
            )?,
        })
    }
}

#[derive(Default)]
pub struct DailySchedule {
    first: HeatPeriod,
    second: HeatPeriod,
}

impl DailySchedule {
    pub fn single(heat_period: HeatPeriod) -> Self {
        DailySchedule {
            first: heat_period,
            second: Default::default(),
        }
    }
    pub fn dual(first_period: HeatPeriod, second_period: HeatPeriod) -> Self {
        DailySchedule {
            first: first_period,
            second: second_period,
        }
    }
    pub fn from(first: Option<String>, second: Option<String>) -> Result<Self, ParseIntError> {
        Ok(DailySchedule {
            first: first.unwrap_or_default().parse()?,
            second: second.unwrap_or_default().parse()?,
        })
    }
}

#[derive(Default)]
pub struct HeatPeriod {
    begin: HeatTime,
    end: HeatTime,
}

impl Into<String> for HeatPeriod {
    fn into(self) -> String {
        format!(
            "{:0>2}{:0>2}{:0>2}{:0>2}",
            self.begin.hours, self.begin.minutes, self.end.hours, self.end.minutes
        )
    }
}

impl FromStr for HeatPeriod {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, ParseIntError> {
        let (begin, end) = s.split_at(4);
        Ok(HeatPeriod {
            begin: FromStr::from_str(begin)?,
            end: FromStr::from_str(end)?,
        })
    }
}

pub struct HeatTime {
    hours: u8,
    minutes: u8,
}

impl FromStr for HeatTime {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, ParseIntError> {
        let (hh, mm) = s.split_at(2);
        Ok(HeatTime {
            hours: u8::from_str(hh)?,
            minutes: u8::from_str(mm)?,
        })
    }
}

impl Default for HeatTime {
    fn default() -> Self {
        HeatTime {
            hours: 0,
            minutes: 0,
        }
    }
}
