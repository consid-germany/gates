use axum::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;

use types::GateState;

use crate::types;
use crate::types::{Comment, Gate, GateKey};

pub mod dynamodb;

const fn is_local() -> bool {
    #[cfg(feature = "local")]
    return true;
    #[cfg(not(feature = "local"))]
    return false;
}

pub async fn default() -> impl Storage {
    if is_local() {
        dynamodb::DynamoDbStorage::new_local(dynamodb::DEFAULT_LOCAL_DYNAMO_DB_PORT).await
    } else {
        dynamodb::DynamoDbStorage::new().await
    }
}
#[allow(dead_code)]
pub async fn test(port: u16) -> impl Storage {
    dynamodb::DynamoDbStorage::new_local(port).await
}

#[derive(Debug, Serialize)]
pub enum UpdateError {
    ItemToUpdateNotFound(String),
    Other(String),
}

#[derive(Debug)]
pub enum InsertError {
    ItemAlreadyExists(String),
    Other(String),
}

#[derive(Debug)]
pub enum FindError {
    ItemCouldNotBeDecoded(String),
    Other(String),
}

#[derive(Debug)]
pub enum DeleteError {
    ItemToDeleteNotFound(String),
    Other(String),
}

/**
Every storage type as to implement this storage type, so we have a consistent interface
between multiple implementations.
For testing purpose implement a in_memory_storage
 */
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Storage {
    async fn insert(&self, gate: &Gate) -> Result<(), InsertError>;
    async fn find_one(&self, key: GateKey) -> Result<Option<Gate>, FindError>;
    async fn find_all(&self) -> Result<Vec<Gate>, FindError>;
    async fn delete(&self, key: GateKey) -> Result<(), DeleteError>;

    async fn update_state_and_last_updated(
        &self,
        key: GateKey,
        state: GateState,
        last_updated: DateTime<Utc>,
    ) -> Result<Gate, UpdateError>;

    async fn update_display_order_and_last_updated(
        &self,
        key: GateKey,
        display_order: u32,
        last_updated: DateTime<Utc>,
    ) -> Result<Gate, UpdateError>;

    async fn update_comment_and_last_updated(
        &self,
        key: GateKey,
        comment: Comment,
        last_updated: DateTime<Utc>,
    ) -> Result<Gate, UpdateError>;
    async fn delete_comment_by_id_and_update_last_updated(
        &self,
        key: GateKey,
        comment_id: String,
        last_updated: DateTime<Utc>,
    ) -> Result<Gate, UpdateError>;
}
