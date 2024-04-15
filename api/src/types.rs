use chrono::{DateTime, NaiveTime, Utc, Weekday};
use itertools::Itertools;
use openapi::models;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod app_state;
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

#[allow(clippy::use_self)]
impl From<ActiveHours> for Box<models::ActiveHours> {
    fn from(active_hours: ActiveHours) -> Self {
        Box::new(models::ActiveHours::from(active_hours))
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

impl From<Gate> for models::GateStateRep {
    fn from(value: Gate) -> Self {
        Self {
            state: value.state.into(),
        }
    }
}

impl From<GateState> for models::GateState {
    fn from(source: GateState) -> Self {
        match source {
            GateState::Closed => models::GateState::Closed,
            GateState::Open => models::GateState::Open,
        }
    }
}
impl From<Gate> for models::Gate {
    fn from(value: Gate) -> Self {
        Self {
            group: value.key.group,
            service: value.key.service,
            environment: value.key.environment,
            state: value.state.into(),
            comments: value
                .comments
                .into_iter()
                .map_into::<models::Comment>()
                .sorted_by_key(|comment| comment.created.to_string())
                .collect(),
            last_updated: value.last_updated.to_string(),
            display_order: value.display_order.map(|v| v as f64),
        }
    }
}

impl From<Comment> for models::Comment {
    fn from(value: Comment) -> Self {
        Self {
            id: value.id,
            message: value.message,
            created: value.created.to_string(),
        }
    }
}

#[cfg(test)]
mod unit_test {
    use std::collections::HashSet;
    use std::str::FromStr;

    use crate::types;
    use chrono::{DateTime, NaiveTime, Utc};
    use openapi::models;

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

    #[test]
    fn should_convert_domain_gate_to_open_api_gate() {
        let gate = some_gate("some-group", "some-service", "some-environment");
        let actual: models::Gate = gate.into();
        let expected = models::Gate {
            group: "some-group".to_owned(),
            service: "some-service".to_owned(),
            environment: "some-environment".to_owned(),
            state: models::GateState::Open,
            comments: vec![
                models::Comment {
                    id: "Comment1".into(),
                    message: "Some comment message".into(),
                    created: DateTime::parse_from_rfc3339("2021-04-12T20:10:57Z")
                        .expect("can not convert date")
                        .to_utc()
                        .to_string(),
                },
                models::Comment {
                    id: "Comment2".into(),
                    message: "Some other comment message".into(),
                    created: DateTime::parse_from_rfc3339("2022-04-12T20:10:57Z")
                        .expect("can not convert date")
                        .to_utc()
                        .to_string(),
                },
            ],
            last_updated: DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                .expect("can not convert date")
                .to_utc()
                .to_string(),
            display_order: Option::default(),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_convert_comment() {
        let actual: models::Comment = types::Comment {
            id: "1234".to_string(),
            message: "Gate closed because of ticket #63468".to_owned(),
            created: DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                .expect("can not convert date")
                .into(),
        }
        .into();

        let expected = models::Comment {
            id: "1234".to_string(),
            message: "Gate closed because of ticket #63468".to_owned(),
            created: DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                .expect("can not convert date")
                .to_utc()
                .to_string(),
        };
        assert_eq!(actual, expected);
    }

    fn some_gate(group: &str, service: &str, environment: &str) -> types::Gate {
        types::Gate {
            key: types::GateKey {
                group: group.to_owned(),
                service: service.to_owned(),
                environment: environment.to_owned(),
            },
            state: types::GateState::Open,
            comments: HashSet::from([
                types::Comment {
                    id: "Comment1".to_owned(),
                    message: "Some comment message".to_owned(),
                    created: DateTime::parse_from_rfc3339("2021-04-12T22:10:57+02:00")
                        .expect("failed creating date")
                        .into(),
                },
                types::Comment {
                    id: "Comment2".to_owned(),
                    message: "Some other comment message".to_owned(),
                    created: DateTime::from(
                        DateTime::parse_from_rfc3339("2022-04-12T22:10:57+02:00")
                            .expect("failed creating date"),
                    ),
                },
            ]),
            last_updated: DateTime::from(
                DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                    .expect("failed creating date"),
            ),
            display_order: Option::default(),
        }
    }
}
