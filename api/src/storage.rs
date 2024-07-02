use std::env;
use std::future::Future;
use std::sync::Arc;

use axum::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;

use types::GateState;

use crate::storage::demo::ReadOnlyStorage;
use crate::storage::dynamodb::DynamoDbStorage;
use crate::types;
use crate::types::{Comment, Gate, GateKey};

mod demo;
pub mod dynamodb;

const fn is_local() -> bool {
    #[cfg(feature = "local")]
    return true;
    #[cfg(not(feature = "local"))]
    return false;
}

pub const DEMO_MODE_ACTIVE: &str = "DEMO_MODE";

fn is_demo_mode() -> bool {
    env::var(DEMO_MODE_ACTIVE).is_ok()
}

pub async fn default() -> Arc<dyn Storage + Send + Sync> {
    if is_local() {
        get_storage(get_local_database()).await
    } else {
        get_storage(get_live_database()).await
    }
}

async fn get_storage(
    database: impl Future<Output = DynamoDbStorage> + Sized + Send,
) -> Arc<dyn Storage + Send + Sync> {
    if is_demo_mode() {
        Arc::new(ReadOnlyStorage::new(Box::new(database.await)))
    } else {
        Arc::new(database.await)
    }
}

async fn get_live_database() -> DynamoDbStorage {
    DynamoDbStorage::new().await
}

async fn get_local_database() -> DynamoDbStorage {
    DynamoDbStorage::new_local(dynamodb::DEFAULT_LOCAL_DYNAMO_DB_PORT).await
}

#[allow(dead_code)]
pub async fn test(port: u16) -> impl Storage {
    DynamoDbStorage::new_local(port).await
}

#[derive(Debug, Serialize)]
pub enum UpdateError {
    ItemToUpdateNotFound(String),
    Other(String),
}

#[derive(Debug)]
pub enum InsertError {
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    ItemToDeleteNotFound(String),
    Other(String),
}

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
