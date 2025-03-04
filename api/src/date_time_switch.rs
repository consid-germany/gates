use chrono::{DateTime, Datelike, Utc};

use crate::types::GateState::Closed;
use crate::types::{BusinessWeek, Gate};

pub struct DefaultDateTimeCircuitBreaker;

#[cfg_attr(test, mockall::automock)]
pub trait DateTimeSwitch {
    fn is_closed(&self, utc: DateTime<Utc>) -> bool;
    fn close_if_time(&self, utc: DateTime<Utc>, gate: Gate) -> Gate;
}

impl DateTimeSwitch for DefaultDateTimeCircuitBreaker {
    fn is_closed(&self, utc: DateTime<Utc>) -> bool {
        #[cfg(not(feature = "date_time_switch"))]
        return false;
        is_outside_of_business_times(&BusinessWeek::default(), utc)
    }

    fn close_if_time(&self, utc: DateTime<Utc>, gate: Gate) -> Gate {
        if self.is_closed(utc) {
            Gate {
                key: gate.key,
                state: Closed,
                comments: gate.comments,
                last_updated: gate.last_updated,
                display_order: gate.display_order,
            }
        } else {
            gate
        }
    }
}

fn is_outside_of_business_times(
    business_week: &BusinessWeek,
    time_to_check: DateTime<Utc>,
) -> bool {
    business_week
        .business_times_by_weekday(time_to_check.weekday())
        .as_ref()
        .is_none_or(|hours| hours.is_outside_of_business_times(time_to_check))
}

pub fn default() -> impl DateTimeSwitch {
    DefaultDateTimeCircuitBreaker {}
}

#[cfg(test)]
mod unit_test {
    use std::collections::HashSet;
    use std::str::FromStr;

    //
    use chrono::{DateTime, NaiveTime};
    use rstest::rstest;

    use crate::date_time_switch;
    use crate::date_time_switch::DateTimeSwitch;
    use crate::types::GateState::{Closed, Open};
    use crate::types::{BusinessTimes, BusinessWeek, Gate, GateKey};

    // TODO use this test configuration
    #[allow(dead_code)]
    fn get_test_configuration() -> BusinessWeek {
        BusinessWeek {
            monday: Some(BusinessTimes {
                start: NaiveTime::from_str("07:00:00").unwrap(),
                end: NaiveTime::from_str("18:30:00").unwrap(),
            }),
            tuesday: None,
            wednesday: None,
            thursday: None,
            friday: None,
            saturday: None,
            sunday: None,
        }
    }

    #[rstest]
    #[test]
    fn should_be_open_during_business_times(#[values("08", "11", "18")] hour: &str) {
        // given
        let monday = DateTime::parse_from_rfc3339(&format!("2023-06-05T{hour}:00:00+00:00"))
            .expect("failed to parse date");
        let expected = false;
        let switch = date_time_switch::default();

        // when
        let actual = switch.is_closed(DateTime::from(monday));

        // then
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[test]
    fn should_be_closed_outside_of_business_times(#[values("06", "20")] hour: &str) {
        // given
        let monday = DateTime::parse_from_rfc3339(&format!("2023-06-05T{hour}:00:00+00:00"))
            .expect("failed to parse date");
        let expected = true;
        let switch = date_time_switch::default();

        // when
        let actual = switch.is_closed(DateTime::from(monday));

        // then
        assert_eq!(expected, actual);
    }

    #[test]
    fn should_be_closed_on_a_day_without_configured_business_times() {
        // given
        let sunday = DateTime::parse_from_rfc3339("2023-06-04T13:59:59+00:00")
            .expect("failed to parse date");
        let switch = date_time_switch::default();

        // when
        let closed = switch.is_closed(DateTime::from(sunday));

        // then
        assert!(closed);
    }

    #[rstest]
    #[case("06:59", true, "should be closed right before start business times")]
    #[case("07:00", false, "should be open at the start of business times")]
    #[case("07:01", false, "should be open a second into business times")]
    #[test]
    fn should_be_open_at_start(
        #[case] hour_and_minute: &str,
        #[case] expected: bool,
        #[case] msg: String,
    ) {
        // given
        let monday =
            DateTime::parse_from_rfc3339(&format!("2023-06-05T{hour_and_minute}:00+00:00"))
                .expect("failed to parse date");
        let switch = date_time_switch::default();

        // when
        let actual = switch.is_closed(DateTime::from(monday));

        // then
        assert_eq!(expected, actual, "{msg}");
    }

    #[rstest]
    #[case("18:29", false, "should be open right before end of business times")]
    #[case("18:30", false, "should be open at the end of business times")]
    #[case("18:31", true, "should be closed a second after business times")]
    #[test]
    fn should_be_closed_at_end(
        #[case] hour_and_minute: &str,
        #[case] expected: bool,
        #[case] msg: String,
    ) {
        // given
        let monday =
            DateTime::parse_from_rfc3339(&format!("2023-06-05T{hour_and_minute}:00+00:00"))
                .expect("failed to parse date");
        let switch = date_time_switch::default();

        // when
        let actual = switch.is_closed(DateTime::from(monday));

        // then
        assert_eq!(expected, actual, "{msg}");
    }

    #[test]
    fn should_return_closed_gate() {
        // given
        let sunday = DateTime::parse_from_rfc3339("2023-06-04T13:59:59+00:00")
            .expect("failed to parse date");
        let switch = date_time_switch::default();
        assert!(switch.is_closed(DateTime::from(sunday)));

        // when
        let actual = switch.close_if_time(
            sunday.into(),
            Gate {
                key: GateKey {
                    group: "unused".to_string(),
                    service: "unused".to_string(),
                    environment: "unused".to_string(),
                },
                state: Open,
                comments: HashSet::default(),
                last_updated: DateTime::default(),
                display_order: Option::default(),
            },
        );

        // then

        assert_eq!(actual.state, Closed);
    }
}
