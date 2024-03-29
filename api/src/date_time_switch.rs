use crate::types::Gate;
use crate::types::GateState::Closed;
use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};

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
        is_in_window(utc)
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

fn is_in_window(utc: DateTime<Utc>) -> bool {
    match utc.weekday() {
        Weekday::Sun | Weekday::Sat => true,
        Weekday::Fri => is_after_12(utc),
        _ => false,
    }
}

fn is_after_12(utc: DateTime<Utc>) -> bool {
    utc.hour() > 12
}

pub fn default() -> impl DateTimeSwitch {
    DefaultDateTimeCircuitBreaker {}
}

#[cfg(test)]
mod unit_test {
    use crate::date_time_switch;
    use crate::date_time_switch::DateTimeSwitch;
    use crate::types::GateState::{Closed, Open};
    use crate::types::{Gate, GateKey};
    use chrono::{DateTime, Datelike, Duration, Utc, Weekday};
    use std::collections::HashSet;

    //*****
    fn get_start_date(utc: DateTime<Utc>, close_start: &str) -> DateTime<Utc> {
        let week_day_now = utc.weekday().number_from_monday();

        let close_day_start = close_start.parse::<Weekday>().unwrap();
        let close_start_day = close_day_start.number_from_monday();
        utc + Duration::days(i64::from(close_start_day)) - Duration::days(i64::from(week_day_now))
    }

    fn get_end_date(
        utc: DateTime<Utc>,
        close_start: &str,
        close_end: &str,
        week_day_now: u32,
    ) -> DateTime<Utc> {
        let close_day_start = close_start.parse::<Weekday>().unwrap();
        let close_start_day = close_day_start.number_from_monday();

        let close_day_end = close_end.parse::<Weekday>().unwrap();
        let close_end_day = close_day_end.number_from_monday();
        if close_start_day > close_end_day {
            utc - Duration::days(i64::from(week_day_now))
                + Duration::days(i64::from(close_end_day + 7))
        } else {
            utc + Duration::days(i64::from(close_end_day - week_day_now))
        }
    }

    fn date_in_window(utc: DateTime<Utc>, close_start: &str, close_end: &str) -> bool {
        let close_end = get_end_date(
            utc,
            close_start,
            close_end,
            utc.weekday().number_from_monday(),
        );
        (get_start_date(utc, close_start) < utc) && (utc < close_end)
    }

    //***
    #[test]
    fn should_find_if_date_in_window() {
        let now = DateTime::parse_from_rfc3339("2023-06-01T13:59:59+02:00") //Donnerstag
            .expect("failed to parse date");
        assert!(!date_in_window(DateTime::from(now), "fri", "mon"));
        assert!(date_in_window(DateTime::from(now), "wed", "sat"));
    }

    #[test]
    fn should_be_open_in_week() {
        let date_time_switch = date_time_switch::default();
        let now = DateTime::parse_from_rfc3339("2023-04-14T13:59:59+02:00") //friday
            .expect("failed to parse date");
        let open = !date_time_switch.is_closed(DateTime::from(now));
        assert!(open);

        let now = DateTime::parse_from_rfc3339("2023-04-13T22:10:57+02:00") //thurs
            .expect("failed to parse date");
        let open = !date_time_switch.is_closed(DateTime::from(now));
        assert!(open);

        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00") //wednesday
            .expect("failed to parse date");
        let open = !date_time_switch.is_closed(DateTime::from(now));
        assert!(open);

        let now = DateTime::parse_from_rfc3339("2023-04-11T22:10:57+02:00") //thuesday
            .expect("failed to parse date");
        let open = !date_time_switch.is_closed(DateTime::from(now));
        assert!(open);

        let now = DateTime::parse_from_rfc3339("2023-04-10T22:10:57+02:00") //monday
            .expect("failed to parse date");
        let open = !date_time_switch.is_closed(DateTime::from(now));
        assert!(open);
    }

    #[test]
    fn should_block_on_weekends() {
        let date_time_switch = date_time_switch::default();

        let now = DateTime::parse_from_rfc3339("2023-04-16T22:10:57+02:00") //sunday
            .expect("failed to parse date");
        let closed = date_time_switch.is_closed(DateTime::from(now));
        assert!(closed);

        let now = DateTime::parse_from_rfc3339("2023-04-15T22:10:57+02:00") //sat
            .expect("failed to parse date");
        let closed = date_time_switch.is_closed(DateTime::from(now));
        assert!(closed);

        let now = DateTime::parse_from_rfc3339("2023-04-14T22:10:57+02:00") //friday
            .expect("failed to parse date");
        let closed = date_time_switch.is_closed(DateTime::from(now));
        assert!(closed);
    }

    #[test]
    fn should_give_closed_gate() {
        let date_time_switch = date_time_switch::default();

        let now = DateTime::parse_from_rfc3339("2023-04-16T22:10:57+02:00") //sunday
            .expect("failed to parse date");
        let closed = date_time_switch.is_closed(DateTime::from(now));
        assert!(closed);

        let closed_gate = date_time_switch.close_if_time(
            now.into(),
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

        assert_eq!(closed_gate.state, Closed);
    }
}
