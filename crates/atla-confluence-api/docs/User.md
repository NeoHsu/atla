# User

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**display_name** | Option<**String**> | Display name of the user. | [optional]
**time_zone** | Option<**String**> | Time zone of the user. Depending on the user's privacy setting, this may return null. | [optional]
**personal_space_id** | Option<**String**> | Space ID of the user's personal space. Returns null, if no personal space for the user. | [optional]
**is_external_collaborator** | Option<**bool**> | Whether the user is an external collaborator. | [optional]
**account_status** | Option<[**models::AccountStatus**](AccountStatus.md)> |  | [optional]
**account_id** | Option<**String**> | Account ID of the user. | [optional]
**email** | Option<**String**> | The email address of the user. Depending on the user's privacy setting, this may return an empty string. | [optional]
**account_type** | Option<[**models::AccountType**](AccountType.md)> |  | [optional]
**public_name** | Option<**String**> | Public name of the user. | [optional]
**profile_picture** | Option<[**models::Icon**](Icon.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
