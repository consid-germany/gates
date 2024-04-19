/*
 * Gates API
 *
 * OpenAPI specification for the Gates API
 *
 * The version of the OpenAPI document: 1.12.0
 * 
 * Generated by: https://openapi-generator.tech
 */


use reqwest;

use crate::{apis::ResponseContent, models};
use super::{Error, configuration};


/// struct for typed errors of method [`add_comment`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AddCommentError {
    Status400(String),
    Status403(),
    Status422(),
    Status500(String),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`create_gate`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateGateError {
    Status400(String),
    Status403(),
    Status409(),
    Status422(),
    Status500(String),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`delete_comment`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeleteCommentError {
    Status403(),
    Status500(String),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`delete_gate`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeleteGateError {
    Status403(),
    Status500(String),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_gate`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetGateError {
    Status403(),
    Status500(String),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_gate_state`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetGateStateError {
    Status403(),
    Status500(String),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`list_gates`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListGatesError {
    Status403(),
    Status500(String),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`update_display_order`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UpdateDisplayOrderError {
    Status400(String),
    Status403(),
    Status422(),
    Status500(String),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`update_gate_state`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UpdateGateStateError {
    Status400(String),
    Status403(),
    Status409(),
    Status422(),
    Status500(String),
    UnknownValue(serde_json::Value),
}


/// You can add additional notes as `comment` to a gate. E.g. Providing information why you changed the gate state. You may use the autogenerated id of the comment to specifically remove it using `delete comment`. 
pub async fn add_comment(configuration: &configuration::Configuration, group: &str, service: &str, environment: &str, add_comment_request: models::AddCommentRequest) -> Result<models::Gate, Error<AddCommentError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/gates/{group}/{service}/{environment}/comments", local_var_configuration.base_path, group=crate::apis::urlencode(group), service=crate::apis::urlencode(service), environment=crate::apis::urlencode(environment));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    local_var_req_builder = local_var_req_builder.json(&add_comment_request);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<AddCommentError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// **Create** or **add** a new gate for the combination of group, service and environment. The default state of a new gate is `closed`. 
pub async fn create_gate(configuration: &configuration::Configuration, create_gate_request: models::CreateGateRequest) -> Result<models::Gate, Error<CreateGateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/gates", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    local_var_req_builder = local_var_req_builder.json(&create_gate_request);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<CreateGateError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Use the autogenerated id to remove a specific comment.
pub async fn delete_comment(configuration: &configuration::Configuration, group: &str, service: &str, environment: &str, comment_id: &str) -> Result<(), Error<DeleteCommentError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/gates/{group}/{service}/{environment}/comments/{comment_id}", local_var_configuration.base_path, group=crate::apis::urlencode(group), service=crate::apis::urlencode(service), environment=crate::apis::urlencode(environment), comment_id=crate::apis::urlencode(comment_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::DELETE, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<DeleteCommentError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Remove a specific gate you don´t longer need. **This will delete the gate permanently**.
pub async fn delete_gate(configuration: &configuration::Configuration, group: &str, service: &str, environment: &str) -> Result<(), Error<DeleteGateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/gates/{group}/{service}/{environment}", local_var_configuration.base_path, group=crate::apis::urlencode(group), service=crate::apis::urlencode(service), environment=crate::apis::urlencode(environment));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::DELETE, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<DeleteGateError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Get information about a gate including e.g. state, comments and other data.
pub async fn get_gate(configuration: &configuration::Configuration, group: &str, service: &str, environment: &str) -> Result<models::Gate, Error<GetGateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/gates/{group}/{service}/{environment}", local_var_configuration.base_path, group=crate::apis::urlencode(group), service=crate::apis::urlencode(service), environment=crate::apis::urlencode(environment));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<GetGateError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// This should be used if you want to explicitly know the state of a gate.
pub async fn get_gate_state(configuration: &configuration::Configuration, group: &str, service: &str, environment: &str) -> Result<models::GateStateRep, Error<GetGateStateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/gates/{group}/{service}/{environment}/state", local_var_configuration.base_path, group=crate::apis::urlencode(group), service=crate::apis::urlencode(service), environment=crate::apis::urlencode(environment));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<GetGateStateError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// A list of all available gates, aggregated and ordered by group and service. Will return an empty array if no gates found.
pub async fn list_gates(configuration: &configuration::Configuration, ) -> Result<Vec<models::Group>, Error<ListGatesError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/gates", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<ListGatesError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// This can be used to sort the frontend representation of the gates. Sorting has to be done by the frontend.
pub async fn update_display_order(configuration: &configuration::Configuration, group: &str, service: &str, environment: &str, update_display_order_request: models::UpdateDisplayOrderRequest) -> Result<models::Gate, Error<UpdateDisplayOrderError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/gates/{group}/{service}/{environment}/display-order", local_var_configuration.base_path, group=crate::apis::urlencode(group), service=crate::apis::urlencode(service), environment=crate::apis::urlencode(environment));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::PUT, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    local_var_req_builder = local_var_req_builder.json(&update_display_order_request);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<UpdateDisplayOrderError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// With this operation you can change the `state` of the gate, e.g. switch the gate state from `closed` to `open` and vice versa.
pub async fn update_gate_state(configuration: &configuration::Configuration, group: &str, service: &str, environment: &str, update_gate_state_request: models::UpdateGateStateRequest) -> Result<models::Gate, Error<UpdateGateStateError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/gates/{group}/{service}/{environment}/state", local_var_configuration.base_path, group=crate::apis::urlencode(group), service=crate::apis::urlencode(service), environment=crate::apis::urlencode(environment));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::PUT, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    local_var_req_builder = local_var_req_builder.json(&update_gate_state_request);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<UpdateGateStateError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

