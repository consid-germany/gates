use crate::storage::Storage;
use crate::types::CONFIG_ID;
use crate::{storage, types};
use axum::async_trait;
use openapi::models::Config;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Internal(String),
}

impl From<storage::FindError> for Error {
    fn from(value: storage::FindError) -> Self {
        match value {
            storage::FindError::ItemCouldNotBeDecoded(error) | storage::FindError::Other(error) => {
                Self::Internal(error)
            }
        }
    }
}

#[async_trait]
pub trait UseCase {
    async fn execute(&self, storage: &(dyn Storage + Send + Sync)) -> Result<Config, Error>;
}

pub fn create() -> impl UseCase {
    UseCaseImpl {}
}

#[derive(Clone)]
struct UseCaseImpl {}

#[async_trait]
impl UseCase for UseCaseImpl {
    async fn execute(&self, storage: &(dyn Storage + Send + Sync)) -> Result<Config, Error> {
        let config = storage.get_config(CONFIG_ID).await?.unwrap_or_else(|| {
            // Handle the None case here and return a default value or handle the error gracefully
            eprintln!("Error: Config not found. Returning default config.");
            types::Config::default() // Return default config
        });

        Ok(config.into())
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::storage::MockStorage;
    use crate::types::{BusinessTimes, BusinessWeek, Config, CONFIG_ID};
    use crate::use_cases::get_config::use_case::{UseCase, UseCaseImpl};
    use chrono::NaiveTime;
    use mockall::predicate::eq;
    use openapi::models;
    use rstest::rstest;
    use similar_asserts::assert_eq;

    pub fn business_week_test_data() -> BusinessWeek {
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
    pub fn config_test_data() -> Config {
        Config {
            id: "id".to_string(),
            business_week: business_week_test_data(),
        }
    }

    #[rstest(config, expected_config,
        case(config_test_data(), config_test_data().into()),
        case(Config::default(), Config::default().into()),
    )]
    #[tokio::test]
    async fn should_get_config(config: Config, expected_config: models::Config) {
        // given
        let mut mock_storage = MockStorage::new();

        mock_storage
            .expect_get_config()
            .with(eq(CONFIG_ID))
            .return_once(move |_| Ok(Some(config)));
        // when
        let actual = UseCaseImpl {}.execute(&mock_storage).await;

        // then
        assert!(actual.is_ok());
        let config_result = actual.unwrap();
        assert_eq!(config_result, expected_config);
    }

    #[tokio::test]
    async fn should_return_default_config_when_not_found() {
        // given
        let mut mock_storage = MockStorage::new();

        mock_storage
            .expect_get_config()
            .with(eq(CONFIG_ID))
            .return_once(move |_| Ok(None));

        // when
        let actual = UseCaseImpl {}.execute(&mock_storage).await;

        // then
        assert!(actual.is_ok());
        let config_result = actual.unwrap();
        assert_eq!(config_result, Config::default().into());
    }
}
