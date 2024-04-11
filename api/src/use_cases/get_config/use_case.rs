use axum::async_trait;
use openapi::models;
use openapi::models::Config;

use crate::clock::Clock;
use crate::types::ActiveHoursPerWeek;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {}

#[async_trait]
pub trait UseCase {
    async fn execute(
        &self,
        clock: &(dyn Clock + Send + Sync),
        active_hours_per_week: ActiveHoursPerWeek,
    ) -> Result<Config, Error>;
}

pub fn create() -> impl UseCase {
    UseCaseImpl {}
}

#[derive(Clone)]
struct UseCaseImpl {}

#[async_trait]
impl UseCase for UseCaseImpl {
    async fn execute(
        &self,
        clock: &(dyn Clock + Send + Sync),
        active_hours_per_week: ActiveHoursPerWeek,
    ) -> Result<Config, Error> {
        let openapi_active_hours_per_week: models::ActiveHoursPerWeek = active_hours_per_week.into();
        Ok(Config {
            system_time: clock.now().to_string(),
            active_hours_per_week: serde_json::to_value(openapi_active_hours_per_week).unwrap()
        })
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::clock::MockClock;
    use crate::types::{ ActiveHours, ActiveHoursPerWeek};
    use crate::use_cases::get_config::use_case::{UseCase, UseCaseImpl};
    use chrono::{DateTime, NaiveTime, Utc};
    use openapi::models;
    use rstest::rstest;
    use similar_asserts::assert_eq;

    pub fn test_data() -> ActiveHoursPerWeek {
        ActiveHoursPerWeek {
            monday: Some(ActiveHours {
                start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            }),
            tuesday: Some(ActiveHours {
                start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            }),
            wednesday: None,
            thursday: None,
            friday: Some(ActiveHours {
                start: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            }),
            saturday: None,
            sunday: None,
        }
    }

    #[rstest(active_hours_per_week, expected_active_hours,
        case(test_data(), test_data().into()),
        case(ActiveHoursPerWeek::default(), ActiveHoursPerWeek::default().into()),
    )]
    #[tokio::test]
    async fn should_get_config(
        active_hours_per_week: ActiveHoursPerWeek,
        expected_active_hours: models::ActiveHoursPerWeek,
    ) {
        // given
        let mut mock_clock = MockClock::new();
        let now: DateTime<Utc> = DateTime::from(
            DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                .expect("failed to parse date"),
        );

        mock_clock.expect_now().return_const(now);

        // when
        let actual = UseCaseImpl {}
            .execute(&mock_clock, active_hours_per_week)
            .await;

        // then
        assert!(actual.is_ok());
        let config_result = actual.unwrap();
        assert_eq!(config_result.system_time, now.to_string());
        assert_eq!(config_result.active_hours_per_week, serde_json::to_value(expected_active_hours).unwrap());
    }
}
