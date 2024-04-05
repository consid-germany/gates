use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};

use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::config::{Credentials, Region};
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::update_item::builders::UpdateItemFluentBuilder;
use aws_sdk_dynamodb::operation::{delete_item, get_item, put_item, query, scan, update_item};
use aws_sdk_dynamodb::types::{
    AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ProvisionedThroughput,
    ReturnValue, ScalarAttributeType,
};
use aws_sdk_dynamodb::{config, Client};
use axum::async_trait;
use chrono::{DateTime, Utc};

use crate::storage::{DeleteError, FindError, InsertError, Storage, UpdateError};
use crate::types::{Comment, Gate, GateKey, GateState};

const GROUP: &str = "group";
const SERVICE_ENVIRONMENT: &str = "service_environment";
const SERVICE: &str = "service";
const ENVIRONMENT: &str = "environment";
const STATE: &str = "state";
const LAST_UPDATED: &str = "last_updated";
const DISPLAY_ORDER: &str = "display_order";
const COMMENTS: &str = "comments";
const ID: &str = "id";
const MESSAGE: &str = "message";
const CREATED: &str = "created";

const LOCAL_GATES_TABLE_NAME: &str = "GatesLocal";
const ENV_GATES_DYNAMO_DB_TABLE_NAME: &str = "GATES_DYNAMO_DB_TABLE_NAME";

pub(super) const DEFAULT_LOCAL_DYNAMO_DB_PORT: u16 = 8000;

#[derive(Clone)]
pub struct DynamoDbStorage {
    pub client: Client,
    pub table: String,
}

#[async_trait]
impl Storage for DynamoDbStorage {
    async fn insert(&self, gate: &Gate) -> Result<(), InsertError> {
        self.client
            .put_item()
            .table_name(&self.table)
            .set_item(Some(gate.into()))
            .condition_expression("attribute_not_exists(#g)")
            .expression_attribute_names("#g", GROUP)
            .send()
            .await?;

        Ok(())
    }

    async fn find_one(
        &self,
        GateKey {
            group,
            service,
            environment,
        }: GateKey,
    ) -> Result<Option<Gate>, FindError> {
        self.client
            .get_item()
            .table_name(&self.table)
            .key(GROUP, AttributeValue::S(group))
            .key(
                SERVICE_ENVIRONMENT,
                AttributeValue::S(get_service_environment(
                    service.as_str(),
                    environment.as_str(),
                )),
            )
            .send()
            .await?
            .item()
            .map(|item| {
                item.try_into().map_err(|error| {
                    FindError::ItemCouldNotBeDecoded(format!(
                        "could not decode gate (mapping error: {error})"
                    ))
                })
            })
            .transpose()
    }

    async fn find_all(&self) -> Result<Vec<Gate>, FindError> {
        self.client
            .scan()
            .table_name(&self.table)
            .into_paginator()
            .items()
            .send()
            .collect::<Result<Vec<_>, _>>()
            .await
            .map_err(FindError::from)
            .map(|result| {
                result
                    .into_iter()
                    .map(|item| {
                        Gate::try_from(&item).map_err(|error| {
                            FindError::ItemCouldNotBeDecoded(format!(
                                "could not decode gate (mapping error: {error})"
                            ))
                        })
                    })
                    .collect()
            })?
    }

    async fn delete(
        &self,
        GateKey {
            group,
            service,
            environment,
        }: GateKey,
    ) -> Result<(), DeleteError> {
        self.client
            .delete_item()
            .table_name(&self.table)
            .key(GROUP, AttributeValue::S(group))
            .key(
                SERVICE_ENVIRONMENT,
                AttributeValue::S(get_service_environment(
                    service.as_str(),
                    environment.as_str(),
                )),
            )
            .condition_expression("attribute_exists(#g)")
            .expression_attribute_names("#g", GROUP)
            .send()
            .await?;

        Ok(())
    }

    async fn update_state_and_last_updated(
        &self,
        GateKey {
            group,
            service,
            environment,
        }: GateKey,
        state: GateState,
        last_updated: DateTime<Utc>,
    ) -> Result<Gate, UpdateError> {
        self.prepare_update(group.as_str(), service.as_str(), environment.as_str())
            .update_expression("SET #s = :newState, #lu = :newLastUpdated")
            .condition_expression("attribute_exists(#g)")
            .expression_attribute_names("#s", STATE)
            .expression_attribute_names("#lu", LAST_UPDATED)
            .expression_attribute_names("#g", GROUP)
            .expression_attribute_values(
                ":newState",
                AttributeValue::S(state.try_into().map_err(UpdateError::Other)?),
            )
            .expression_attribute_values(
                ":newLastUpdated",
                AttributeValue::S(last_updated.to_rfc3339()),
            )
            .send()
            .await?
            .attributes()
            .ok_or_else(|| UpdateError::Other("missing updated gate".to_owned()))?
            .try_into()
            .map_err(|error| {
                UpdateError::Other(format!("could not decode gate (mapping error: {error})"))
            })
    }

    async fn update_display_order_and_last_updated(
        &self,
        GateKey {
            group,
            service,
            environment,
        }: GateKey,
        display_order: u32,
        last_updated: DateTime<Utc>,
    ) -> Result<Gate, UpdateError> {
        self.prepare_update(group.as_str(), service.as_str(), environment.as_str())
            .update_expression("SET #dp = :display_order, #lu = :newLastUpdated")
            .condition_expression("attribute_exists(#g)")
            .expression_attribute_names("#dp", DISPLAY_ORDER)
            .expression_attribute_names("#lu", LAST_UPDATED)
            .expression_attribute_names("#g", GROUP)
            .expression_attribute_values(
                ":display_order",
                AttributeValue::N(display_order.to_string()),
            )
            .expression_attribute_values(
                ":newLastUpdated",
                AttributeValue::S(last_updated.to_rfc3339()),
            )
            .send()
            .await?
            .attributes()
            .ok_or_else(|| UpdateError::Other("missing updated gate".to_owned()))?
            .try_into()
            .map_err(|error| {
                UpdateError::Other(format!("could not decode gate (mapping error: {error})"))
            })
    }

    async fn update_comment_and_last_updated(
        &self,
        GateKey {
            group,
            service,
            environment,
        }: GateKey,
        comment: Comment,
        last_updated: DateTime<Utc>,
    ) -> Result<Gate, UpdateError> {
        self.prepare_update(group.as_str(), service.as_str(), environment.as_str())
            .update_expression("SET #c.#i = :newComment, #lu = :newLastUpdated")
            .condition_expression("attribute_exists(#g)")
            .expression_attribute_names("#c", COMMENTS)
            .expression_attribute_names("#i", comment.id.clone())
            .expression_attribute_names("#lu", LAST_UPDATED)
            .expression_attribute_names("#g", GROUP)
            .expression_attribute_values(":newComment", AttributeValue::M(HashMap::from(&comment)))
            .expression_attribute_values(
                ":newLastUpdated",
                AttributeValue::S(last_updated.to_rfc3339()),
            )
            .send()
            .await?
            .attributes()
            .ok_or_else(|| UpdateError::Other("missing updated gate".to_owned()))?
            .try_into()
            .map_err(|error| {
                UpdateError::Other(format!("could not decode gate (mapping error: {error})"))
            })
    }

    async fn delete_comment_by_id_and_update_last_updated(
        &self,
        GateKey {
            group,
            service,
            environment,
        }: GateKey,
        comment_id: String,
        last_updated: DateTime<Utc>,
    ) -> Result<Gate, UpdateError> {
        self.prepare_update(group.as_str(), service.as_str(), environment.as_str())
            .update_expression("REMOVE #c.#i SET #lu = :newLastUpdated")
            .condition_expression("attribute_exists(#g) AND attribute_exists(#c.#i)")
            .expression_attribute_names("#c", COMMENTS)
            .expression_attribute_names("#i", comment_id)
            .expression_attribute_names("#lu", LAST_UPDATED)
            .expression_attribute_names("#g", GROUP)
            .expression_attribute_values(
                ":newLastUpdated",
                AttributeValue::S(last_updated.to_rfc3339()),
            )
            .send()
            .await?
            .attributes()
            .ok_or_else(|| UpdateError::Other("missing updated gate".to_owned()))?
            .try_into()
            .map_err(|error| {
                UpdateError::Other(format!("could not decode gate (mapping error: {error})"))
            })
    }
}

impl DynamoDbStorage {
    pub async fn new() -> Self {
        let aws_config = &aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;
        let client = Client::from_conf(config::Builder::from(aws_config).build());

        Self {
            client,
            table: std::env::var(ENV_GATES_DYNAMO_DB_TABLE_NAME).unwrap(),
        }
    }

    pub async fn new_local(port: u16) -> Self {
        let client = Client::from_conf(
            config::Builder::new()
                .behavior_version(BehaviorVersion::v2023_11_09())
                .endpoint_url(format!("http://localhost:{port}/"))
                .credentials_provider(Credentials::new(
                    "AccessKeyId",
                    "SecretAccessKeyId",
                    None,
                    None,
                    "static",
                ))
                .region(Region::new("eu-central-1"))
                .build(),
        );

        create_local_table(&client).await;

        Self {
            client,
            table: LOCAL_GATES_TABLE_NAME.to_owned(),
        }
    }

    fn prepare_update(
        &self,
        group: &str,
        service: &str,
        environment: &str,
    ) -> UpdateItemFluentBuilder {
        self.client
            .update_item()
            .table_name(&self.table)
            .key(GROUP, AttributeValue::S(group.to_owned()))
            .key(
                SERVICE_ENVIRONMENT,
                AttributeValue::S(get_service_environment(service, environment)),
            )
            .return_values(ReturnValue::AllNew)
    }
}

async fn create_local_table(client: &Client) {
    _ = client
        .create_table()
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(1)
                .write_capacity_units(1)
                .build()
                .expect("failed to build ProvisionedThroughput"),
        )
        .table_name(LOCAL_GATES_TABLE_NAME.to_owned())
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name(GROUP.to_owned())
                .attribute_type(ScalarAttributeType::S)
                .build()
                .expect("failed to build AttributeDefinition"),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name(SERVICE_ENVIRONMENT.to_owned())
                .attribute_type(ScalarAttributeType::S)
                .build()
                .expect("failed to build AttributeDefinition"),
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(GROUP.to_owned())
                .key_type(KeyType::Hash)
                .build()
                .expect("failed to build KeySchemaElement"),
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(SERVICE_ENVIRONMENT.to_owned())
                .key_type(KeyType::Range)
                .build()
                .expect("failed to build KeySchemaElement"),
        )
        .send()
        .await;
}

fn get_service_environment(service: &str, environment: &str) -> String {
    format!("{service}#{environment}")
}

/////////////////////////////////////////////////////////////////////////////
// Encode
/////////////////////////////////////////////////////////////////////////////

fn encode_string(field: &str, value: String) -> (String, AttributeValue) {
    (field.to_owned(), AttributeValue::S(value))
}

fn encode_datetime_utc(field: &str, value: DateTime<Utc>) -> (String, AttributeValue) {
    (field.to_owned(), AttributeValue::S(value.to_rfc3339()))
}

fn encode_map(field: &str, value: HashMap<String, AttributeValue>) -> (String, AttributeValue) {
    (field.to_owned(), AttributeValue::M(value))
}

fn encode_u32(field: &str, value: u32) -> (String, AttributeValue) {
    (field.to_owned(), AttributeValue::N(value.to_string()))
}

impl From<&Gate> for HashMap<String, AttributeValue, RandomState> {
    fn from(value: &Gate) -> Self {
        let mut fields = vec![
            encode_string(GROUP, value.key.group.clone()),
            encode_string(
                SERVICE_ENVIRONMENT,
                get_service_environment(value.key.service.as_str(), value.key.environment.as_str()),
            ),
            encode_string(SERVICE, value.key.service.clone()),
            encode_string(ENVIRONMENT, value.key.environment.clone()),
            encode_string(
                STATE,
                value
                    .state
                    .clone()
                    .try_into()
                    .expect("TODO Can not convert value to state"),
            ),
            encode_datetime_utc(LAST_UPDATED, value.last_updated),
            encode_map(
                COMMENTS,
                value
                    .comments
                    .iter()
                    .map(|comment| (comment.id.clone(), AttributeValue::M(comment.into())))
                    .collect(),
            ),
        ];

        if value.display_order.is_some() {
            fields.push(encode_u32(
                DISPLAY_ORDER,
                value.display_order.expect("TODO"),
            ));
        }

        Self::from_iter(fields)
    }
}

impl From<&Comment> for HashMap<String, AttributeValue, RandomState> {
    fn from(value: &Comment) -> Self {
        Self::from([
            (ID.to_owned(), AttributeValue::S(value.id.clone())),
            (MESSAGE.to_owned(), AttributeValue::S(value.message.clone())),
            (
                CREATED.to_owned(),
                AttributeValue::S(value.created.to_rfc3339()),
            ),
        ])
    }
}

/////////////////////////////////////////////////////////////////////////////
// Decode
/////////////////////////////////////////////////////////////////////////////

type DecodeError = String;

fn decode_string(
    field: &str,
    input: &HashMap<String, AttributeValue>,
) -> Result<String, DecodeError> {
    input
        .get(field)
        .ok_or(format!("field {field} could not be found"))?
        .as_s()
        .map_err(|_| format!("field {field} could not be parsed as string"))
        .cloned()
}

fn decode_datetime_utc(
    field: &str,
    input: &HashMap<String, AttributeValue>,
) -> Result<DateTime<Utc>, DecodeError> {
    DateTime::parse_from_rfc3339(&decode_string(field, input)?)
        .map_err(|_| format!("field {field} could not be parsed as datetime"))
        .map(std::convert::Into::into)
}

fn decode_optional_u32(
    field: &str,
    input: &HashMap<String, AttributeValue>,
) -> Result<Option<u32>, DecodeError> {
    input
        .get(field)
        .map(|value| {
            value
                .as_n()
                .map_err(|_| format!("field {field} could not be parsed as string"))
                .and_then(|value| {
                    value.parse().map_err(|error| {
                        format!("field {field} could not be parsed as u32: {error}")
                    })
                })
        })
        .transpose()
}

fn decode_map<'a>(
    field: &str,
    input: &'a HashMap<String, AttributeValue>,
) -> Result<&'a HashMap<String, AttributeValue>, DecodeError> {
    input
        .get(field)
        .ok_or(format!("field {field} could not be found"))?
        .as_m()
        .map_err(|_| format!("field {field} could not be parsed as map"))
}

impl TryFrom<&HashMap<String, AttributeValue>> for Gate {
    type Error = String;

    fn try_from(value: &HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(Self {
            key: GateKey {
                group: decode_string(GROUP, value)?,
                service: decode_string(SERVICE, value)?,
                environment: decode_string(ENVIRONMENT, value)?,
            },
            state: decode_string(STATE, value)?.try_into()?,
            comments: decode_map(COMMENTS, value)?
                .iter()
                .map(|(id, value)| {
                    value
                        .as_m()
                        .map_err(|_| format!("comment {id} could not be parsed"))
                        .and_then(std::convert::TryInto::try_into)
                })
                .collect::<Result<HashSet<Comment>, String>>()?,
            last_updated: decode_datetime_utc(LAST_UPDATED, value)?,
            display_order: decode_optional_u32(DISPLAY_ORDER, value)?,
        })
    }
}

impl TryFrom<&HashMap<String, AttributeValue>> for Comment {
    type Error = String;

    fn try_from(value: &HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: decode_string(ID, value)?,
            message: decode_string(MESSAGE, value)?,
            created: decode_datetime_utc(CREATED, value)?,
        })
    }
}

/////////////////////////////////////////////////////////////////////////////
// Converting SdkError<_> to Storage Errors
/////////////////////////////////////////////////////////////////////////////

impl From<SdkError<get_item::GetItemError>> for FindError {
    fn from(value: SdkError<get_item::GetItemError>) -> Self {
        Self::Other(aws_sdk_dynamodb::error::DisplayErrorContext(value).to_string())
    }
}

impl From<SdkError<put_item::PutItemError>> for InsertError {
    fn from(value: SdkError<put_item::PutItemError>) -> Self {
        match value.into_service_error() {
            put_item::PutItemError::ConditionalCheckFailedException(exception) => {
                Self::ItemAlreadyExists(
                    aws_sdk_dynamodb::error::DisplayErrorContext(exception).to_string(),
                )
            }
            error => Self::Other(aws_sdk_dynamodb::error::DisplayErrorContext(error).to_string()),
        }
    }
}

impl From<SdkError<update_item::UpdateItemError>> for UpdateError {
    fn from(value: SdkError<update_item::UpdateItemError>) -> Self {
        match value.into_service_error() {
            update_item::UpdateItemError::ConditionalCheckFailedException(exception) => {
                Self::ItemToUpdateNotFound(
                    aws_sdk_dynamodb::error::DisplayErrorContext(exception).to_string(),
                )
            }
            error => Self::Other(aws_sdk_dynamodb::error::DisplayErrorContext(error).to_string()),
        }
    }
}

impl From<SdkError<scan::ScanError>> for FindError {
    fn from(value: SdkError<scan::ScanError>) -> Self {
        Self::Other(aws_sdk_dynamodb::error::DisplayErrorContext(value).to_string())
    }
}

impl From<SdkError<query::QueryError>> for FindError {
    fn from(value: SdkError<query::QueryError>) -> Self {
        Self::Other(aws_sdk_dynamodb::error::DisplayErrorContext(value).to_string())
    }
}

impl From<SdkError<delete_item::DeleteItemError>> for DeleteError {
    fn from(value: SdkError<delete_item::DeleteItemError>) -> Self {
        match value.into_service_error() {
            delete_item::DeleteItemError::ConditionalCheckFailedException(exception) => {
                Self::ItemToDeleteNotFound(
                    aws_sdk_dynamodb::error::DisplayErrorContext(exception).to_string(),
                )
            }

            error => Self::Other(aws_sdk_dynamodb::error::DisplayErrorContext(error).to_string()),
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use chrono::DateTime;
    use itertools::concat;
    use mockall::Any;
    use similar_asserts::assert_eq;
    use testcontainers::clients;
    use testcontainers_modules::dynamodb_local::DynamoDb;

    use crate::types::Gate;

    use super::*;

    #[tokio::test]
    async fn should_insert_and_find_one() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = DynamoDbStorage::new_local(port).await;
        assert_empty(&dynamodb_storage).await;

        let gate = some_gate("some group", "some service", "some environment");

        dynamodb_storage
            .insert(&gate)
            .await
            .expect("storage failed to insert gate");

        // when
        let result = dynamodb_storage.find_one(gate.key.clone()).await;

        // then
        let stored_gate = result
            .expect("storage failed to find gate")
            .expect("gate not found");
        assert_eq!(stored_gate, gate);
    }

    #[tokio::test]
    async fn should_not_insert_if_item_already_exists() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = DynamoDbStorage::new_local(port).await;
        assert_empty(&dynamodb_storage).await;

        let gate = some_gate("some group", "some service", "some environment");

        dynamodb_storage
            .insert(&gate)
            .await
            .expect("storage failed to insert gate");

        // when
        let result = dynamodb_storage.insert(&gate).await;

        // then
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result.expect_err("expected error not found").type_name(),
            InsertError::ItemAlreadyExists(String::default()).type_name()
        );
    }

    #[tokio::test]
    async fn should_not_find_one_if_gate_not_exists() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = DynamoDbStorage::new_local(port).await;
        assert_empty(&dynamodb_storage).await;

        // when
        let result = dynamodb_storage
            .find_one(GateKey {
                group: "some group".to_owned(),
                service: "some service".to_owned(),
                environment: "some environment".to_owned(),
            })
            .await;

        // then
        let stored_gate = result.expect("storage failed to find gate");
        assert_eq!(stored_gate.is_none(), true);
    }

    #[tokio::test]
    async fn should_insert_and_find_all() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = DynamoDbStorage::new_local(port).await;
        assert_empty(&dynamodb_storage).await;

        let gate1 = some_gate("some group", "some service", "some environment");
        let gate2 = some_gate(
            "some other group",
            "some other service",
            "some other environment",
        );

        dynamodb_storage
            .insert(&gate1)
            .await
            .expect("storage failed to insert gate");
        dynamodb_storage
            .insert(&gate2)
            .await
            .expect("storage failed to insert gate");

        // when
        let result = dynamodb_storage.find_all().await;

        // then
        let stored_gates = result.expect("storage failed to find gates");
        assert_eq!(stored_gates.len(), 2);
        assert_eq!(stored_gates, vec![gate1, gate2]);
    }

    #[tokio::test]
    async fn should_insert_and_delete() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = DynamoDbStorage::new_local(port).await;
        assert_empty(&dynamodb_storage).await;

        let gate = some_gate("some group", "some service", "some environment");

        dynamodb_storage
            .insert(&gate)
            .await
            .expect("storage failed to insert gate");

        let stored_gates = dynamodb_storage
            .find_all()
            .await
            .expect("storage failed to find gates");
        assert_eq!(stored_gates.len(), 1);

        // when
        let result = dynamodb_storage.delete(gate.key).await;

        // then
        result.expect("storage failed to delete gate");
        let count = dynamodb_storage
            .find_all()
            .await
            .expect("storage failed to find gates")
            .len();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn should_fail_to_delete_item_if_item_does_not_exist() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = DynamoDbStorage::new_local(port).await;
        assert_empty(&dynamodb_storage).await;

        let gate = some_gate("some group", "some service", "some environment");

        dynamodb_storage
            .insert(&gate)
            .await
            .expect("storage failed to insert gate");

        let stored_gates = dynamodb_storage
            .find_all()
            .await
            .expect("storage failed to find gates");
        assert_eq!(stored_gates.len(), 1);

        // when
        let result = dynamodb_storage
            .delete(GateKey {
                group: "some group".to_owned(),
                service: "some service".to_owned(),
                environment: "some wrong environment".to_owned(),
            })
            .await;

        // then
        assert_eq!(result.is_err(), true);
        let stored_gates = dynamodb_storage
            .find_all()
            .await
            .expect("storage failed to find gates");
        assert_eq!(stored_gates.len(), 1);
        assert_eq!(stored_gates, vec![gate]);
    }

    #[tokio::test]
    async fn should_update_state_and_last_modified() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = DynamoDbStorage::new_local(port).await;
        assert_empty(&dynamodb_storage).await;

        let gate = some_gate("some group", "some service", "some environment");
        assert_eq!(gate.state, GateState::Open);
        assert_eq!(
            gate.last_updated,
            DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                .expect("failed creating date")
        );

        dynamodb_storage
            .insert(&gate)
            .await
            .expect("storage failed to insert gate");

        // when
        let new_last_updated = DateTime::parse_from_rfc3339("2025-04-12T22:10:57+02:00")
            .expect("failed creating date")
            .into();
        let new_state = GateState::Closed;

        let result = dynamodb_storage
            .update_state_and_last_updated(
                GateKey {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                },
                new_state.clone(),
                new_last_updated,
            )
            .await;

        // then
        result.expect("storage failed to update gate state");
        let stored_gates = dynamodb_storage
            .find_all()
            .await
            .expect("storage failed to find gates");
        assert_eq!(stored_gates.len(), 1);
        assert_eq!(
            *stored_gates.first().expect("failed to get stored gate"),
            Gate {
                key: gate.key,
                state: new_state,
                comments: gate.comments,
                last_updated: new_last_updated,
                display_order: gate.display_order,
            }
        );
    }

    #[tokio::test]
    async fn should_fail_to_update_state_and_last_modified_of_item_that_does_not_exist() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = DynamoDbStorage::new_local(port).await;
        assert_empty(&dynamodb_storage).await;

        let gate = some_gate("some group", "some service", "some environment");
        dynamodb_storage
            .insert(&gate)
            .await
            .expect("storage failed to insert gate");

        // when
        let result = dynamodb_storage
            .update_state_and_last_updated(
                GateKey {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some wrong environment".to_owned(),
                },
                GateState::Closed,
                DateTime::parse_from_rfc3339("2025-04-12T22:10:57+02:00")
                    .expect("failed creating date")
                    .into(),
            )
            .await;

        // then
        assert_eq!(result.is_err(), true);
        let stored_gates = dynamodb_storage
            .find_all()
            .await
            .expect("storage failed to find gates");
        assert_eq!(stored_gates.len(), 1);
        assert_eq!(
            *stored_gates.first().expect("failed to get stored gate"),
            gate
        );
    }

    #[tokio::test]
    async fn should_add_new_comment_and_update_last_modified() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = DynamoDbStorage::new_local(port).await;
        assert_empty(&dynamodb_storage).await;

        let gate = some_gate("some group", "some service", "some environment");

        dynamodb_storage
            .insert(&gate)
            .await
            .expect("storage failed to insert gate");

        let now = DateTime::parse_from_rfc3339("2025-04-12T22:10:57+02:00")
            .expect("failed creating date")
            .into();
        let new_comment = Comment {
            id: "NewCommentId".to_owned(),
            message: "Some new comment message".to_owned(),
            created: now,
        };

        // when
        let result = dynamodb_storage
            .update_comment_and_last_updated(gate.key.clone(), new_comment.clone(), now)
            .await;

        // then
        result.expect("storage failed to update gate comment");
        let stored_gates = dynamodb_storage
            .find_all()
            .await
            .expect("storage failed to find gates");
        assert_eq!(stored_gates.len(), 1);
        assert_eq!(
            *stored_gates.first().expect("failed to get stored gate"),
            Gate {
                key: gate.key,
                state: gate.state,
                comments: concat(vec![gate.comments, HashSet::from([new_comment])]),
                last_updated: now,
                display_order: gate.display_order,
            }
        );
    }

    #[tokio::test]
    async fn should_update_existing_comment_and_update_last_modified() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = DynamoDbStorage::new_local(port).await;
        assert_empty(&dynamodb_storage).await;

        let gate = some_gate("some group", "some service", "some environment");

        dynamodb_storage
            .insert(&gate)
            .await
            .expect("storage failed to insert gate");

        let now = DateTime::parse_from_rfc3339("2025-04-12T22:10:57+02:00")
            .expect("failed creating date")
            .into();
        let changed_comment = Comment {
            id: "Comment1".to_owned(),
            message: "Some changed comment message".to_owned(),
            created: now,
        };

        // when
        let result = dynamodb_storage
            .update_comment_and_last_updated(gate.key.clone(), changed_comment.clone(), now)
            .await;

        // then
        result.expect("storage failed to update gate comment");
        let stored_gates = dynamodb_storage
            .find_all()
            .await
            .expect("storage failed to find gates");
        assert_eq!(stored_gates.len(), 1);
        assert_eq!(
            *stored_gates.first().expect("failed to get stored gate"),
            Gate {
                key: gate.key,
                state: gate.state,
                comments: HashSet::from([
                    changed_comment,
                    Comment {
                        id: "Comment2".to_owned(),
                        message: "Some other comment message".to_owned(),
                        created: DateTime::parse_from_rfc3339("2022-04-12T22:10:57+02:00")
                            .expect("failed creating date")
                            .into(),
                    },
                ]),
                last_updated: now,
                display_order: gate.display_order,
            }
        );
    }

    #[tokio::test]
    async fn should_fail_updating_comment_and_update_last_modified_if_item_does_not_exist() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = DynamoDbStorage::new_local(port).await;
        assert_empty(&dynamodb_storage).await;

        let now = DateTime::parse_from_rfc3339("2025-04-12T22:10:57+02:00")
            .expect("failed creating date")
            .into();
        let changed_comment = Comment {
            id: "Comment1".to_owned(),
            message: "Some changed comment message".to_owned(),
            created: now,
        };

        // when
        let result = dynamodb_storage
            .update_comment_and_last_updated(
                GateKey {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                },
                changed_comment.clone(),
                now,
            )
            .await;

        // then
        assert_eq!(result.is_err(), true);
        let stored_gates = dynamodb_storage
            .find_all()
            .await
            .expect("storage failed to find gates");
        assert_eq!(stored_gates.len(), 0);
    }

    #[tokio::test]
    async fn should_delete_comment_by_id_and_update_last_modified() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = DynamoDbStorage::new_local(port).await;
        assert_empty(&dynamodb_storage).await;

        let gate = some_gate("some group", "some service", "some environment");

        dynamodb_storage
            .insert(&gate)
            .await
            .expect("storage failed to insert gate");

        let now = DateTime::parse_from_rfc3339("2025-04-12T22:10:57+02:00")
            .expect("failed creating date")
            .into();

        // when
        let result = dynamodb_storage
            .delete_comment_by_id_and_update_last_updated(
                gate.key.clone(),
                "Comment1".to_owned(),
                now,
            )
            .await;

        // then
        result.expect("storage failed to delete gate comment");
        let stored_gates = dynamodb_storage
            .find_all()
            .await
            .expect("storage failed to find gates");
        assert_eq!(stored_gates.len(), 1);
        assert_eq!(
            *stored_gates.first().expect("failed to get stored gate"),
            Gate {
                key: gate.key,
                state: gate.state,
                comments: HashSet::from([
                    // Comment1 removed
                    Comment {
                        id: "Comment2".to_owned(),
                        message: "Some other comment message".to_owned(),
                        created: DateTime::parse_from_rfc3339("2022-04-12T22:10:57+02:00")
                            .expect("failed creating date")
                            .into(),
                    },
                ]),
                last_updated: now,
                display_order: gate.display_order,
            }
        );
    }

    #[tokio::test]
    async fn should_fail_to_delete_comment_by_id_and_update_last_modified_if_comment_does_not_exist(
    ) {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = DynamoDbStorage::new_local(port).await;
        assert_empty(&dynamodb_storage).await;

        let gate = some_gate("some group", "some service", "some environment");

        dynamodb_storage
            .insert(&gate)
            .await
            .expect("storage failed to insert gate");

        let now = DateTime::parse_from_rfc3339("2025-04-12T22:10:57+02:00")
            .expect("failed creating date")
            .into();

        // when
        let result = dynamodb_storage
            .delete_comment_by_id_and_update_last_updated(
                gate.key.clone(),
                "NonExistentCommentId".to_owned(),
                now,
            )
            .await;

        // then
        assert_eq!(result.is_err(), true);
        let stored_gates = dynamodb_storage
            .find_all()
            .await
            .expect("storage failed to find gates");
        assert_eq!(stored_gates.len(), 1);
        assert_eq!(
            *stored_gates.first().expect("failed to get stored gate"),
            gate
        );
    }

    #[tokio::test]
    async fn should_fail_to_delete_comment_by_id_and_update_last_modified_if_item_does_not_exist() {
        // given
        let docker = clients::Cli::default();

        let dynamodb_container = docker.run(DynamoDb);
        let port = dynamodb_container.get_host_port_ipv4(8000);

        let dynamodb_storage = DynamoDbStorage::new_local(port).await;
        assert_empty(&dynamodb_storage).await;

        let now = DateTime::parse_from_rfc3339("2025-04-12T22:10:57+02:00")
            .expect("failed creating date")
            .into();

        // when
        let result = dynamodb_storage
            .delete_comment_by_id_and_update_last_updated(
                GateKey {
                    group: "some group".to_owned(),
                    service: "some service".to_owned(),
                    environment: "some environment".to_owned(),
                },
                "Comment1".to_owned(),
                now,
            )
            .await;

        // then
        assert_eq!(result.is_err(), true);
        let stored_gates = dynamodb_storage
            .find_all()
            .await
            .expect("storage failed to find gates");
        assert_eq!(stored_gates.len(), 0);
    }

    async fn assert_empty(dynamodb_storage: &DynamoDbStorage) {
        let count = dynamodb_storage
            .find_all()
            .await
            .expect("storage failed to find gates")
            .len();
        assert_eq!(count, 0);
    }

    fn some_gate(group: &str, service: &str, environment: &str) -> Gate {
        Gate {
            key: GateKey {
                group: group.to_owned(),
                service: service.to_owned(),
                environment: environment.to_owned(),
            },
            state: GateState::Open,
            comments: HashSet::from([
                Comment {
                    id: "Comment1".to_owned(),
                    message: "Some comment message".to_owned(),
                    created: DateTime::parse_from_rfc3339("2021-04-12T22:10:57+02:00")
                        .expect("failed creating date")
                        .into(),
                },
                Comment {
                    id: "Comment2".to_owned(),
                    message: "Some other comment message".to_owned(),
                    created: DateTime::parse_from_rfc3339("2022-04-12T22:10:57+02:00")
                        .expect("failed creating date")
                        .into(),
                },
            ]),
            last_updated: DateTime::parse_from_rfc3339("2023-04-12T22:10:57+02:00")
                .expect("failed creating date")
                .into(),
            display_order: Some(2),
        }
    }
}
