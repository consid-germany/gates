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

/// ActiveHoursPerWeek : The start and end time of each day.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveHoursPerWeek {
    #[serde(rename = "monday", skip_serializing_if = "Option::is_none")]
    pub monday: Option<Box<models::ActiveHours>>,
    #[serde(rename = "tuesday", skip_serializing_if = "Option::is_none")]
    pub tuesday: Option<Box<models::ActiveHours>>,
    #[serde(rename = "wednesday", skip_serializing_if = "Option::is_none")]
    pub wednesday: Option<Box<models::ActiveHours>>,
    #[serde(rename = "thursday", skip_serializing_if = "Option::is_none")]
    pub thursday: Option<Box<models::ActiveHours>>,
    #[serde(rename = "friday", skip_serializing_if = "Option::is_none")]
    pub friday: Option<Box<models::ActiveHours>>,
    #[serde(rename = "saturday", skip_serializing_if = "Option::is_none")]
    pub saturday: Option<Box<models::ActiveHours>>,
    #[serde(rename = "sunday", skip_serializing_if = "Option::is_none")]
    pub sunday: Option<Box<models::ActiveHours>>,
}

impl ActiveHoursPerWeek {
    /// The start and end time of each day.
    pub fn new() -> ActiveHoursPerWeek {
        ActiveHoursPerWeek {
            monday: None,
            tuesday: None,
            wednesday: None,
            thursday: None,
            friday: None,
            saturday: None,
            sunday: None,
        }
    }
}

