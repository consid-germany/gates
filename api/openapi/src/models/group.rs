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

/// Group : This is the toplevel of arranging the gates. Gates are bracketed in an environment, environments are part of a service and services are in a group.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Group {
    /// The name of the group. Could be any string, but mainly we recommend an organizational orientation.
    #[serde(rename = "name")]
    pub name: String,
    /// Service is the level between Group an Environment. A service is part of a group and has multiple environments.
    #[serde(rename = "services")]
    pub services: Vec<models::Service>,
}

impl Group {
    /// This is the toplevel of arranging the gates. Gates are bracketed in an environment, environments are part of a service and services are in a group.
    pub fn new(name: String, services: Vec<models::Service>) -> Group {
        Group {
            name,
            services,
        }
    }
}

