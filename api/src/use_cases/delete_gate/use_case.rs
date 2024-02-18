use crate::storage;
use crate::storage::Storage;
use crate::types::GateKey;
use axum::async_trait;

#[derive(Debug)]
pub struct Input {
    pub group: String,
    pub service: String,
    pub environment: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    GateNotFound,
    Internal(String),
}

impl From<storage::DeleteError> for Error {
    fn from(value: storage::DeleteError) -> Self {
        match value {
            storage::DeleteError::ItemToDeleteNotFound(_) => Self::GateNotFound,
            storage::DeleteError::Other(error) => Self::Internal(error),
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UseCase {
    async fn execute<'required_for_mocking>(
        &self,
        input: Input,
        storage: &(dyn Storage + Send + Sync + 'required_for_mocking),
    ) -> Result<(), Error>;
}

pub fn create() -> impl UseCase {
    UseCaseImpl {}
}

#[derive(Clone)]
struct UseCaseImpl;

#[async_trait]
impl UseCase for UseCaseImpl {
    async fn execute<'required_for_mocking>(
        &self,
        Input {
            group,
            service,
            environment,
        }: Input,
        storage: &(dyn Storage + Send + Sync + 'required_for_mocking),
    ) -> Result<(), Error> {
        Ok(storage
            .delete_one(GateKey {
                group,
                service,
                environment,
            })
            .await?)
    }
}

#[cfg(test)]
mod unit_tests {
    use chrono::DateTime;
    use mockall::predicate::eq;

    use crate::clock::MockClock;
    use crate::storage::{DeleteError, MockStorage};
    use crate::types::GateKey;

    use super::*;

    #[tokio::test]
    async fn should_delete_gate() {
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();
        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        mock_storage
            .expect_delete_one()
            .with(eq(GateKey {
                group: "group".to_string(),
                service: "service".to_string(),
                environment: "develop".to_string(),
            }))
            .return_once(|_| Ok(()));

        let left = UseCaseImpl {}
            .execute(
                Input {
                    group: "group".to_owned(),
                    service: "service".to_owned(),
                    environment: "develop".to_owned(),
                },
                &mock_storage,
            )
            .await;

        assert!(left.is_ok());
    }

    #[tokio::test]
    async fn should_return_gate_not_found_error_if_storage_could_not_find_item_to_delete() {
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();

        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        mock_storage.expect_delete_one().returning(move |_| {
            Err(DeleteError::ItemToDeleteNotFound(
                "ConditionalCheckFailedException".to_owned(),
            ))
        });

        let left = UseCaseImpl {}
            .execute(
                Input {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                },
                &mock_storage,
            )
            .await;

        assert!(left.is_err());
        assert_eq!(left.expect_err("Error expected here"), Error::GateNotFound);
    }

    #[tokio::test]
    async fn should_return_storage_error() {
        let mut mock_storage = MockStorage::new();
        let mut mock_clock = MockClock::new();

        let now = DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
            .expect("failed to parse date");
        mock_clock.expect_now().return_const(now);

        mock_storage
            .expect_delete_one()
            .returning(move |_| Err(DeleteError::Other("some error".to_owned())));

        let left = UseCaseImpl {}
            .execute(
                Input {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                },
                &mock_storage,
            )
            .await;

        assert!(left.is_err());
        assert_eq!(
            left.expect_err("Error expected here"),
            Error::Internal("some error".to_owned())
        );
    }
}
