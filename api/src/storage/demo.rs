use axum::async_trait;
use chrono::{DateTime, Utc};
use std::iter::Iterator;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::storage;
use crate::storage::{DeleteError, FindError, InsertError, UpdateError};
use crate::types::{Comment, Gate, GateKey, GateState};

type DynStorage = dyn storage::Storage + Send + Sync;

pub struct ReadOnlyStorage {
    pub proxy: Box<DynStorage>,
}

const QUOTES_STR: &str = include_str!("demo_quotes.txt");

#[cfg(test)]
fn random_quote() -> String {
    "random quote".to_string()
}

#[cfg(not(test))]
fn random_quote() -> String {
    if 1 == 1 {
        return "random quote".to_string();
    }
    let quotes: Vec<&str> = QUOTES_STR.split("\n").collect();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time not retrieved")
        .as_millis() as usize;
    quotes
        .get(now % quotes.len())
        .unwrap_or_else(|| panic!("quote could not be obtained"))
        .to_string()
}

#[async_trait]
impl storage::Storage for ReadOnlyStorage {
    async fn insert(&self, _: &Gate) -> Result<(), InsertError> {
        Err(InsertError::Other("not allowed in demo mode".to_owned()))
    }

    async fn find_one(&self, key: GateKey) -> Result<Option<Gate>, FindError> {
        self.proxy.find_one(key).await
    }
    async fn find_all(&self) -> Result<Vec<Gate>, FindError> {
        self.proxy.find_all().await
    }

    async fn delete(&self, _: GateKey) -> Result<(), DeleteError> {
        Err(DeleteError::Other("not allowed in demo mode".to_owned()))
    }

    async fn update_state_and_last_updated(
        &self,
        key: GateKey,
        state: GateState,
        last_updated: DateTime<Utc>,
    ) -> Result<Gate, UpdateError> {
        self.proxy
            .update_state_and_last_updated(key, state, last_updated)
            .await
    }

    async fn update_display_order_and_last_updated(
        &self,
        key: GateKey,
        display_order: u32,
        last_updated: DateTime<Utc>,
    ) -> Result<Gate, UpdateError> {
        self.proxy
            .update_display_order_and_last_updated(key, display_order, last_updated)
            .await
    }

    async fn update_comment_and_last_updated(
        &self,
        key: GateKey,
        comment: Comment,
        last_updated: DateTime<Utc>,
    ) -> Result<Gate, UpdateError> {
        self.proxy
            .update_comment_and_last_updated(
                key,
                Comment {
                    id: comment.id,
                    message: random_quote(),
                    created: last_updated,
                },
                last_updated,
            )
            .await
    }

    async fn delete_comment_by_id_and_update_last_updated(
        &self,
        key: GateKey,
        comment_id: String,
        last_updated: DateTime<Utc>,
    ) -> Result<Gate, UpdateError> {
        self.proxy
            .delete_comment_by_id_and_update_last_updated(key, comment_id, last_updated)
            .await
    }
}

impl ReadOnlyStorage {
    pub fn new(proxy: Box<DynStorage>) -> Self {
        Self { proxy }
    }
}

#[cfg(test)]
mod unit_test {
    use std::collections::HashSet;

    use chrono::{DateTime, Utc};
    use mockall::predicate::eq;

    use crate::storage::demo::ReadOnlyStorage;
    use crate::storage::{MockStorage, Storage};
    use crate::types::{Comment, Gate, GateKey, GateState};

    #[tokio::test]
    async fn should_not_insert() {
        // when
        let mock_storage = MockStorage::new();

        let actual = ReadOnlyStorage {
            proxy: Box::new(mock_storage),
        }
        .insert(&Gate {
            key: GateKey {
                group: String::new(),
                service: String::new(),
                environment: String::new(),
            },
            state: GateState::default(),
            comments: HashSet::default(),
            last_updated: DateTime::default(),
            display_order: None,
        })
        .await;
        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn should_not_delete() {
        // when
        let mock_storage = MockStorage::new();

        let actual = ReadOnlyStorage {
            proxy: Box::new(mock_storage),
        }
        .delete(GateKey {
            group: String::new(),
            service: String::new(),
            environment: String::new(),
        })
        .await;
        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn should_sanitize_last_updated_comment() {
        // when
        let mut mock_storage = MockStorage::new();
        let now = Utc::now();

        mock_storage
            .expect_update_comment_and_last_updated()
            .with(
                eq(GateKey {
                    group: String::new(),
                    service: String::new(),
                    environment: String::new(),
                }),
                eq(Comment {
                    id: "some_id".to_owned(),
                    message: "random quote".to_owned(),
                    created: now,
                }),
                eq(now),
            )
            .return_once(move |key, _, last_updated| {
                Ok(Gate {
                    key,
                    state: GateState::default(),
                    comments: HashSet::from([Comment {
                        id: "some_id".to_owned(),
                        message: "random quote".to_owned(),
                        created: last_updated,
                    }]),
                    last_updated: now,
                    display_order: None,
                })
            });
        let actual = ReadOnlyStorage {
            proxy: Box::new(mock_storage),
        }
        .update_comment_and_last_updated(
            GateKey {
                group: String::new(),
                service: String::new(),
                environment: String::new(),
            },
            Comment {
                id: "some_id".to_owned(),
                message: "some dirty comment message".to_owned(),
                created: now,
            },
            now,
        )
        .await;

        assert!(actual.is_ok());
        assert_eq!(
            actual.expect(""),
            Gate {
                key: GateKey {
                    group: String::new(),
                    service: String::new(),
                    environment: String::new(),
                },
                state: GateState::default(),
                comments: HashSet::from([Comment {
                    id: "some_id".to_owned(),
                    message: "random quote".to_owned(),
                    created: now,
                }]),
                last_updated: now,
                display_order: None,
            }
        );
    }

    #[tokio::test]
    async fn should_read_one_gate_from_proxy() {
        let mut storage = MockStorage::new();
        storage
            .expect_find_one()
            .with(eq(GateKey {
                group: "input".to_owned(),
                service: "input".to_owned(),
                environment: "input".to_owned(),
            }))
            .return_once(move |gate_key| {
                Ok(Some(Gate {
                    key: gate_key,
                    state: GateState::default(),
                    comments: HashSet::default(),
                    last_updated: DateTime::default(),
                    display_order: None,
                }))
            });
        let actual = ReadOnlyStorage::new(Box::new(storage))
            .find_one(GateKey {
                group: "input".to_owned(),
                service: "input".to_owned(),
                environment: "input".to_owned(),
            })
            .await;
        assert!(actual.is_ok());
        assert_eq!(
            actual.unwrap(),
            Some(Gate {
                key: GateKey {
                    group: "input".to_owned(),
                    service: "input".to_owned(),
                    environment: "input".to_owned(),
                },
                state: GateState::default(),
                comments: HashSet::default(),
                last_updated: DateTime::default(),
                display_order: None,
            })
        );
    }

    #[tokio::test]
    async fn should_read_all_gates_from_proxy() {
        let mut storage = MockStorage::new();
        storage.expect_find_all().return_once(move || {
            Ok(Vec::from([Gate {
                key: GateKey {
                    group: "output".to_owned(),
                    service: "output".to_owned(),
                    environment: "output".to_owned(),
                },
                state: GateState::default(),
                comments: HashSet::default(),
                last_updated: DateTime::default(),
                display_order: None,
            }]))
        });
        let actual = ReadOnlyStorage::new(Box::new(storage)).find_all().await;
        assert!(actual.is_ok());
        assert_eq!(
            actual.unwrap(),
            Vec::from([Gate {
                key: GateKey {
                    group: "output".to_owned(),
                    service: "output".to_owned(),
                    environment: "output".to_owned(),
                },
                state: GateState::default(),
                comments: HashSet::default(),
                last_updated: DateTime::default(),
                display_order: None,
            }])
        );
    }

    #[tokio::test]
    async fn should_update_state_and_last_updated_on_proxy() {
        let now = Utc::now();
        let mut storage = MockStorage::new();
        storage
            .expect_update_state_and_last_updated()
            .with(
                eq(GateKey {
                    group: "input".to_owned(),
                    service: "input".to_owned(),
                    environment: "input".to_owned(),
                }),
                eq(GateState::Closed),
                eq(now),
            )
            .return_once(move |gate_key: GateKey, state: GateState, last_updated| {
                Ok(Gate {
                    key: gate_key,
                    state,
                    comments: HashSet::default(),
                    last_updated,
                    display_order: None,
                })
            });
        let actual = ReadOnlyStorage::new(Box::new(storage))
            .update_state_and_last_updated(
                GateKey {
                    group: "input".to_owned(),
                    service: "input".to_owned(),
                    environment: "input".to_owned(),
                },
                GateState::Closed,
                now,
            )
            .await;
        assert!(actual.is_ok());
        assert_eq!(
            actual.unwrap(),
            Gate {
                key: GateKey {
                    group: "input".to_owned(),
                    service: "input".to_owned(),
                    environment: "input".to_owned(),
                },
                state: GateState::default(),
                comments: HashSet::default(),
                last_updated: now,
                display_order: None,
            }
        );
    }

    #[tokio::test]
    async fn should_update_display_order_and_last_updated_on_proxy() {
        let now = Utc::now();
        let mut storage = MockStorage::new();
        storage
            .expect_update_display_order_and_last_updated()
            .with(
                eq(GateKey {
                    group: "input".to_owned(),
                    service: "input".to_owned(),
                    environment: "input".to_owned(),
                }),
                eq(0),
                eq(now),
            )
            .return_once(move |key: GateKey, display_order, last_updated| {
                Ok(Gate {
                    key,
                    state: GateState::default(),
                    comments: HashSet::default(),
                    last_updated,
                    display_order: Some(display_order),
                })
            });
        let actual = ReadOnlyStorage::new(Box::new(storage))
            .update_display_order_and_last_updated(
                GateKey {
                    group: "input".to_owned(),
                    service: "input".to_owned(),
                    environment: "input".to_owned(),
                },
                0,
                now,
            )
            .await;
        assert!(actual.is_ok());
        assert_eq!(
            actual.unwrap(),
            Gate {
                key: GateKey {
                    group: "input".to_owned(),
                    service: "input".to_owned(),
                    environment: "input".to_owned(),
                },
                state: GateState::default(),
                comments: HashSet::default(),
                last_updated: now,
                display_order: Some(0),
            }
        );
    }

    #[tokio::test]
    async fn should_delete_comment_by_id_and_update_last_updated_on_proxy() {
        let now = Utc::now();
        let mut storage = MockStorage::new();
        storage
            .expect_delete_comment_by_id_and_update_last_updated()
            .with(
                eq(GateKey {
                    group: "input".to_owned(),
                    service: "input".to_owned(),
                    environment: "input".to_owned(),
                }),
                eq(0.to_string()),
                eq(now),
            )
            .return_once(move |key, _, last_updated| {
                Ok(Gate {
                    key,
                    state: GateState::default(),
                    comments: HashSet::default(),
                    last_updated,
                    display_order: None,
                })
            });
        let actual = ReadOnlyStorage::new(Box::new(storage))
            .delete_comment_by_id_and_update_last_updated(
                GateKey {
                    group: "input".to_owned(),
                    service: "input".to_owned(),
                    environment: "input".to_owned(),
                },
                0.to_string(),
                now,
            )
            .await;
        assert!(actual.is_ok());
        assert_eq!(
            actual.unwrap(),
            Gate {
                key: GateKey {
                    group: "input".to_owned(),
                    service: "input".to_owned(),
                    environment: "input".to_owned(),
                },
                state: GateState::default(),
                comments: HashSet::default(),
                last_updated: now,
                display_order: None,
            }
        );
    }
}
