use axum::async_trait;
use openapi::models;
use openapi::models::Config;

use crate::clock::Clock;
use crate::types::BusinessWeek;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {}

#[async_trait]
pub trait UseCase {
    async fn execute(
        &self,
        clock: &(dyn Clock + Send + Sync),
        business_week: BusinessWeek,
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
        business_week: BusinessWeek,
    ) -> Result<Config, Error> {
        let openapi_business_week: models::BusinessWeek = business_week.into();
        Ok(Config::new(clock.now().to_rfc3339(), openapi_business_week))
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::clock::MockClock;
    use crate::types::{BusinessTimes, BusinessWeek};
    use crate::use_cases::get_config::use_case::{UseCase, UseCaseImpl};
    use chrono::{DateTime, NaiveTime, Utc};
    use openapi::models;
    use rstest::rstest;
    use similar_asserts::assert_eq;

    pub const fn test_data() -> BusinessWeek {
        BusinessWeek {
            monday: Some(BusinessTimes {
                start: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            }),
            tuesday: Some(BusinessTimes {
                start: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            }),
            wednesday: None,
            thursday: None,
            friday: Some(BusinessTimes {
                start: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                end: NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            }),
            saturday: None,
            sunday: None,
        }
    }

    #[rstest(business_week, expected_business_times,
        case(test_data(), test_data().into()),
        case(BusinessWeek::default(), BusinessWeek::default().into()),
    )]
    #[tokio::test]
    async fn should_get_config(
        business_week: BusinessWeek,
        expected_business_times: models::BusinessWeek,
    ) {
        // given
        let mut mock_clock = MockClock::new();
        let now: DateTime<Utc> = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date")
            .to_utc();
        mock_clock.expect_now().return_const(now);

        // when
        let actual = UseCaseImpl {}.execute(&mock_clock, business_week).await;

        // then
        assert!(actual.is_ok());
        let config_result = actual.unwrap();
        assert_eq!(config_result.system_time, now.to_rfc3339());
        assert_eq!(config_result.business_week, expected_business_times);
    }
}
