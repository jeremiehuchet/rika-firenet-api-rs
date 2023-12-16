use std::{num::ParseIntError, str::FromStr};

use rika_firenet_openapi::models::StoveStatus;

#[derive(PartialEq, Eq, Debug)]
pub enum StatusDetail {
    Bake,
    Burnout,
    Cleaning,
    Control,
    DeepCleaning,
    ExternalRequest,
    FrostProtection,
    Heat,
    Ignition,
    Off,
    Standby,
    Startup,
    Unknown,
    Wood,
    WoodPresenceControl,
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
