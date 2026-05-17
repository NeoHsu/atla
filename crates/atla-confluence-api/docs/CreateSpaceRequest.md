# CreateSpaceRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | The name of the space to be created. |
**key** | Option<**String**> | The key for the new space. See [Space Keys](https://support.atlassian.com/confluence-cloud/docs/create-a-space/). If the key property is not provided, the alias property is required to be used instead. | [optional]
**alias** | Option<**String**> | This field will be used as the new identifier for the space in confluence page URLs. If the alias property is not provided, the key property is required to be used instead. Maximum 255 alphanumeric characters in length. | [optional]
**description** | Option<[**models::CreateSpaceRequestDescription**](CreateSpaceRequestDescription.md)> |  | [optional]
**role_assignments** | Option<[**Vec<models::CreateSpaceRequestRoleAssignmentsInner>**](CreateSpaceRequestRoleAssignmentsInner.md)> |  | [optional]
**copy_space_access_configuration** | Option<**i32**> | The id of the space to copy the space access configuration from. | [optional]
**create_private_space** | Option<**bool**> | Whether to create the space as private. | [optional]
**template_key** | Option<**String**> | The key of the template to use. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
