use axum::async_trait;

use crate::clock::Clock;
use crate::types::representation::Config;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {}

#[async_trait]
pub trait UseCase {
    async fn execute(&self, clock: &(dyn Clock + Send + Sync)) -> Result<Config, Error>;
}

pub fn create() -> impl UseCase {
    UseCaseImpl {}
}

#[derive(Clone)]
struct UseCaseImpl {}

#[async_trait]
impl UseCase for UseCaseImpl {
    async fn execute(&self, clock: &(dyn Clock + Send + Sync)) -> Result<Config, Error> {
        Ok(Config {
            system_time: clock.now(),
        })
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::clock::MockClock;
    use crate::use_cases::get_config::use_case::{UseCase, UseCaseImpl};
    use chrono::{DateTime, Utc};
    use similar_asserts::assert_eq;

    #[tokio::test]
    async fn should_get_config() {
        // given
        let mut mock_clock = MockClock::new();
        let now: DateTime<Utc> = DateTime::from(
            DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                .expect("failed to parse date"),
        );

        mock_clock.expect_now().return_const(now);

        // when
        let actual = UseCaseImpl {}.execute(&mock_clock).await;

        // then
        //TODO test for more config properties
        assert!(actual.is_ok());
        assert_eq!(actual.expect("failed to get config").system_time, now);
    }
}
