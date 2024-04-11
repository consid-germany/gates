# Gate

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**group** | **String** | The group that surrounds the service. | 
**service** | **String** | The service that contains the environment. | 
**environment** | **String** | The environment for the specific gate. | 
**state** | [**models::GateState**](GateState.md) |  | 
**display_order** | Option<**f64**> | The way to describe how you want to arrange your gates. | [optional]
**comments** | [**Vec<models::Comment>**](Comment.md) | The comment describe why you open or closed the gate. | 
**last_updated** | **String** | Changes when a comment or gate is changed. | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


