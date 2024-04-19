/*
 * Gates API
 *
 * OpenAPI specification for the Gates API
 *
 * The version of the OpenAPI document: 1.12.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddCommentRequest {
    /// This should be a thoughtful description of why this gate is here.
    #[serde(rename = "message")]
    pub message: String,
}

impl AddCommentRequest {
    pub fn new(message: String) -> AddCommentRequest {
        AddCommentRequest {
            message,
        }
    }
}
