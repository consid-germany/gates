use chrono::{DateTime, NaiveTime, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use openapi::models;

pub mod app_state;
pub mod representation;
pub mod use_cases;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveHours {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

impl ActiveHours {
    pub fn is_outside_of_active_hours(&self, date_to_check: DateTime<Utc>) -> bool {
        let time_to_check = date_to_check.time();
        time_to_check < self.start || time_to_check > self.end
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveHoursPerWeek {
    pub monday: Option<ActiveHours>,
    pub tuesday: Option<ActiveHours>,
    pub wednesday: Option<ActiveHours>,
    pub thursday: Option<ActiveHours>,
    pub friday: Option<ActiveHours>,
    pub saturday: Option<ActiveHours>,
    pub sunday: Option<ActiveHours>,
}

impl ActiveHoursPerWeek {
    pub const fn active_hours_by_weekday(&self, weekday: Weekday) -> &Option<ActiveHours> {
        match weekday {
            Weekday::Mon => &self.monday,
            Weekday::Tue => &self.tuesday,
            Weekday::Wed => &self.wednesday,
            Weekday::Thu => &self.thursday,
            Weekday::Fri => &self.friday,
            Weekday::Sat => &self.saturday,
            Weekday::Sun => &self.sunday,
        }

    }


    pub fn default() -> Self {
        Self {
            monday: Some(ActiveHours {
                start: NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(18, 30, 0).unwrap(),
            }),
            tuesday: Some(ActiveHours {
                start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            }),
            wednesday: Some(ActiveHours {
                start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            }),
            thursday: Some(ActiveHours {
                start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            }),
            friday: Some(ActiveHours {
                start: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            }),
            saturday: None,
            sunday: None,
        }
    }
}

impl From<ActiveHoursPerWeek> for models::ActiveHoursPerWeek {
    fn from(value: ActiveHoursPerWeek) -> Self {
        Self {
            monday: value.monday.map(Into::into),
            tuesday: value.tuesday.map(Into::into),
            wednesday: value.wednesday.map(Into::into),
            thursday: value.thursday.map(Into::into),
            friday: value.friday.map(Into::into),
            saturday: value.saturday.map(Into::into),
            sunday: value.sunday.map(Into::into),
        }
    }
}

impl Into<Box<models::ActiveHours>> for ActiveHours {
    fn into(self) -> Box<models::ActiveHours> {
        Box::new(models::ActiveHours::from(self))
    }
}

impl From<ActiveHours> for models::ActiveHours {
    fn from(value: ActiveHours) -> Self {
        Self {
            start: value.start.to_string(),
            end: value.end.to_string(),
        }
    }
}

#[derive(Debug, serde::Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DayOfWeek {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub system_time: DateTime<Utc>,
    pub active_hours_per_week: ActiveHoursPerWeek,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateKey {
    pub group: String,
    pub service: String,
    pub environment: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    pub key: GateKey,
    pub state: GateState,
    pub comments: HashSet<Comment>,
    pub last_updated: DateTime<Utc>,
    pub display_order: Option<u32>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Comment {
    pub id: String,
    pub message: String,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum GateState {
    Open,
    #[default]
    Closed,
}

impl TryFrom<String> for GateState {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&format!("\"{value}\"")).map_or_else(
            |serde_error| {
                Err(format!(
                    "cannot convert {value} to GateState: {serde_error}"
                ))
            },
            Ok,
        )
    }
}

impl TryFrom<GateState> for String {
    type Error = Self;

    fn try_from(value: GateState) -> Result<Self, Self::Error> {
        serde_json::to_string(&value)
            .map(|gate_state| gate_state.replace('\"', ""))
            .map_or_else(
                |serde_error| {
                    Err(format!(
                        "cannot convert from GateState to string: {serde_error}"
                    ))
                },
                Ok,
            )
    }
}

#[cfg(test)]
mod unit_test {
    use std::str::FromStr;

    use chrono::{DateTime, NaiveTime, Utc};

    use crate::types::ActiveHours;

    fn given_active_hours() -> ActiveHours {
        ActiveHours {
            start: NaiveTime::from_str("07:00:00").unwrap(),
            end: NaiveTime::from_str("18:30:00").unwrap(),
        }
    }

    fn setup_date_time(time_str: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(&format!("1970-01-01T{time_str}+00:00"))
            .expect("failed to parse date")
            .into()
    }

    #[test]
    fn should_be_outside_of_active_hours_for_time_before_start() {
        // given
        let given_date_time = setup_date_time("06:00:00");
        let active_hours = given_active_hours();
        let expected = true;

        // when
        let actual = active_hours.is_outside_of_active_hours(given_date_time);

        // then
        assert_eq!(expected, actual);
    }

    #[test]
    fn should_not_be_outside_of_active_hours_for_time_equal_to_start() {
        // given
        let given_date_time = setup_date_time("07:00:00");
        let active_hours = given_active_hours();
        let expected = false;

        // when
        let actual = active_hours.is_outside_of_active_hours(given_date_time);

        // then
        assert_eq!(expected, actual);
    }

    #[test]
    fn should_not_be_outside_of_active_hours_for_time_between_start_and_end() {
        // given
        let given_date_time = setup_date_time("13:00:00");
        let active_hours = given_active_hours();
        let expected = false;

        // when
        let actual = active_hours.is_outside_of_active_hours(given_date_time);

        // then
        assert_eq!(expected, actual);
    }

    #[test]
    fn should_not_be_outside_of_active_hours_for_time_equal_to_end() {
        // given
        let given_date_time = setup_date_time("18:30:00");
        let active_hours = given_active_hours();
        let expected = false;

        // when
        let actual = active_hours.is_outside_of_active_hours(given_date_time);

        // then
        assert_eq!(expected, actual);
    }

    #[test]
    fn should_be_outside_of_active_hours_for_time_after_end() {
        // given
        let given_date_time = setup_date_time("19:00:00");
        let active_hours = given_active_hours();
        let expected = true;

        // when
        let actual = active_hours.is_outside_of_active_hours(given_date_time);

        // then
        assert_eq!(expected, actual);
    }
}
