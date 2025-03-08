use std::str::FromStr;

use anyhow::{Result, bail, ensure};
use rika_firenet_openapi::models::StoveControls;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum StatusDetail {
    Baking,
    BurnOff,
    Cleaning,
    Running,
    DeepCleaning,
    ExternalRequest,
    FrostProtection,
    HeatingUp,
    Ignition,
    Off,
    Standby,
    Startup,
    Unknown,
    SplitLogMode,
    SplitLogCheck,
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

impl Into<u8> for OperatingMode {
    fn into(self) -> u8 {
        match self {
            OperatingMode::Manual => 0,
            OperatingMode::Auto => 1,
            OperatingMode::Comfort => 2,
        }
    }
}

impl OperatingMode {
    pub fn parse(mode: i32) -> Result<Self> {
        match mode {
            0 => Ok(OperatingMode::Manual),
            1 => Ok(OperatingMode::Auto),
            2 => Ok(OperatingMode::Comfort),
            num => bail!("{num} is not a valid OperatingMode"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HeatingSchedule {
    pub monday: DailySchedule,
    pub tuesday: DailySchedule,
    pub wednesday: DailySchedule,
    pub thursday: DailySchedule,
    pub friday: DailySchedule,
    pub saturday: DailySchedule,
    pub sunday: DailySchedule,
}

impl HeatingSchedule {
    pub fn all_same(day_schedule: DailySchedule) -> Self {
        HeatingSchedule {
            monday: day_schedule.clone(),
            tuesday: day_schedule.clone(),
            wednesday: day_schedule.clone(),
            thursday: day_schedule.clone(),
            friday: day_schedule.clone(),
            saturday: day_schedule.clone(),
            sunday: day_schedule,
        }
    }

    pub fn week_vs_end_days(week_day: DailySchedule, week_end: DailySchedule) -> Self {
        HeatingSchedule {
            monday: week_day.clone(),
            tuesday: week_day.clone(),
            wednesday: week_day.clone(),
            thursday: week_day.clone(),
            friday: week_day,
            saturday: week_end.clone(),
            sunday: week_end,
        }
    }
}

impl From<StoveControls> for HeatingSchedule {
    fn from(controls: StoveControls) -> Self {
        HeatingSchedule {
            monday: DailySchedule::from(controls.heating_time_mon1, controls.heating_time_mon2),
            tuesday: DailySchedule::from(controls.heating_time_thu1, controls.heating_time_thu2),
            wednesday: DailySchedule::from(controls.heating_time_wed1, controls.heating_time_wed2),
            thursday: DailySchedule::from(controls.heating_time_tue1, controls.heating_time_tue2),
            friday: DailySchedule::from(controls.heating_time_fri1, controls.heating_time_fri2),
            saturday: DailySchedule::from(controls.heating_time_sat1, controls.heating_time_sat2),
            sunday: DailySchedule::from(controls.heating_time_sun1, controls.heating_time_sun2),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DailySchedule {
    pub first: HeatPeriod,
    pub second: HeatPeriod,
}

impl DailySchedule {
    pub fn new(first_period: HeatPeriod, second_period: Option<HeatPeriod>) -> Self {
        DailySchedule {
            first: first_period,
            second: second_period.unwrap_or_default(),
        }
    }

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
    pub fn from(first: Option<String>, second: Option<String>) -> Self {
        DailySchedule {
            first: first
                .unwrap_or("00000000".to_string())
                .parse()
                .unwrap_or_default(),
            second: second
                .unwrap_or("00000000".to_string())
                .parse()
                .unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HeatPeriod {
    begin: HeatTime,
    end: HeatTime,
}

impl HeatPeriod {
    pub fn new(begin_hours: u8, begin_minutes: u8, end_hours: u8, end_minutes: u8) -> Result<Self> {
        if begin_hours == end_hours {
            ensure!(
                begin_minutes < end_minutes,
                "Heat period can't overlap 2 days"
            );
        } else {
            ensure!(begin_hours < end_hours, "Heat period can't overlap 2 days");
        }
        Ok(HeatPeriod {
            begin: HeatTime {
                hours: begin_hours,
                minutes: begin_minutes,
            },
            end: HeatTime {
                hours: end_hours,
                minutes: end_minutes,
            },
        })
    }
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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (begin, end) = s.split_at(4);
        Ok(HeatPeriod {
            begin: FromStr::from_str(begin)?,
            end: FromStr::from_str(end)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HeatTime {
    hours: u8,
    minutes: u8,
}

impl HeatTime {
    pub fn new(hours: u8, minutes: u8) -> Result<Self> {
        ensure!(hours <= 23, "hours must be 0 <= hh <= 23");
        ensure!(minutes <= 59, "minutes must be 0 <= hh <= 59");
        Ok(HeatTime { hours, minutes })
    }
}

impl FromStr for HeatTime {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (hh, mm) = s.split_at(2);
        HeatTime::new(u8::from_str(hh)?, u8::from_str(mm)?)
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

#[cfg(test)]
mod tests {
    use rika_firenet_openapi::models::StoveControls;

    use crate::model::{DailySchedule, HeatPeriod, HeatingSchedule};

    #[test]
    fn can_parse_missing_schedule() {
        let stove_controls = StoveControls {
            heating_time_mon1: None,
            heating_time_mon2: None,
            heating_time_thu1: None,
            heating_time_thu2: None,
            heating_time_wed1: None,
            heating_time_wed2: None,
            heating_time_tue1: None,
            heating_time_tue2: None,
            heating_time_fri1: None,
            heating_time_fri2: None,
            heating_time_sat1: None,
            heating_time_sat2: None,
            heating_time_sun1: None,
            heating_time_sun2: None,
            ..Default::default()
        };
        let actual = HeatingSchedule::from(stove_controls);
        let expected = HeatingSchedule {
            monday: DailySchedule::default(),
            tuesday: DailySchedule::default(),
            wednesday: DailySchedule::default(),
            thursday: DailySchedule::default(),
            friday: DailySchedule::default(),
            saturday: DailySchedule::default(),
            sunday: DailySchedule::default(),
        };
        assert_eq!(actual, expected)
    }

    #[test]
    fn can_parse_empty_schedule() {
        let stove_controls = StoveControls {
            heating_time_mon1: Some("00000000".to_string()),
            heating_time_mon2: Some("00000000".to_string()),
            heating_time_thu1: Some("00000000".to_string()),
            heating_time_thu2: Some("00000000".to_string()),
            heating_time_wed1: Some("00000000".to_string()),
            heating_time_wed2: Some("00000000".to_string()),
            heating_time_tue1: Some("00000000".to_string()),
            heating_time_tue2: Some("00000000".to_string()),
            heating_time_fri1: Some("00000000".to_string()),
            heating_time_fri2: Some("00000000".to_string()),
            heating_time_sat1: Some("00000000".to_string()),
            heating_time_sat2: Some("00000000".to_string()),
            heating_time_sun1: Some("00000000".to_string()),
            heating_time_sun2: Some("00000000".to_string()),
            ..Default::default()
        };
        let actual = HeatingSchedule::from(stove_controls);
        let expected = HeatingSchedule {
            monday: DailySchedule::default(),
            tuesday: DailySchedule::default(),
            wednesday: DailySchedule::default(),
            thursday: DailySchedule::default(),
            friday: DailySchedule::default(),
            saturday: DailySchedule::default(),
            sunday: DailySchedule::default(),
        };
        assert_eq!(actual, expected)
    }

    #[test]
    fn can_parse_mixed_schedule() {
        let stove_controls = StoveControls {
            heating_time_mon1: Some("08001230".to_string()),
            heating_time_mon2: Some("18452200".to_string()),
            heating_time_thu1: Some("02000330".to_string()),
            heating_time_thu2: Some("08002330".to_string()),
            heating_time_wed1: Some("07001115".to_string()),
            heating_time_wed2: Some("16301945".to_string()),
            heating_time_tue1: Some("06010702".to_string()),
            heating_time_tue2: Some("08030904".to_string()),
            heating_time_fri1: Some("10101212".to_string()),
            heating_time_fri2: Some("13131414".to_string()),
            heating_time_sat1: Some("01000600".to_string()),
            heating_time_sat2: Some("12091532".to_string()),
            heating_time_sun1: Some("20202121".to_string()),
            heating_time_sun2: Some("22222323".to_string()),
            ..Default::default()
        };
        let actual = HeatingSchedule::from(stove_controls);
        let expected = HeatingSchedule {
            monday: DailySchedule::dual(
                HeatPeriod::new(8, 0, 12, 30).unwrap(),
                HeatPeriod::new(18, 45, 22, 00).unwrap(),
            ),
            tuesday: DailySchedule::dual(
                HeatPeriod::new(2, 0, 3, 30).unwrap(),
                HeatPeriod::new(8, 0, 23, 30).unwrap(),
            ),
            wednesday: DailySchedule::dual(
                HeatPeriod::new(7, 0, 11, 15).unwrap(),
                HeatPeriod::new(16, 30, 19, 45).unwrap(),
            ),
            thursday: DailySchedule::dual(
                HeatPeriod::new(6, 1, 7, 2).unwrap(),
                HeatPeriod::new(8, 3, 9, 4).unwrap(),
            ),
            friday: DailySchedule::dual(
                HeatPeriod::new(10, 10, 12, 12).unwrap(),
                HeatPeriod::new(13, 13, 14, 14).unwrap(),
            ),
            saturday: DailySchedule::dual(
                HeatPeriod::new(1, 0, 6, 0).unwrap(),
                HeatPeriod::new(12, 9, 15, 32).unwrap(),
            ),
            sunday: DailySchedule::dual(
                HeatPeriod::new(20, 20, 21, 21).unwrap(),
                HeatPeriod::new(22, 22, 23, 23).unwrap(),
            ),
        };
        assert_eq!(actual, expected)
    }

    #[tokio::test]
    async fn cant_create_cross_day_schedule() {
        let error = HeatPeriod::new(18, 00, 2, 30).unwrap_err();
        assert_eq!(error.to_string(), "Heat period can't overlap 2 days");

        let error = HeatPeriod::new(18, 30, 18, 29).unwrap_err();
        assert_eq!(error.to_string(), "Heat period can't overlap 2 days");
    }
}
