# UpdateSpaceRoleRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **String** | Name of the space role |
**description** | **String** | Description for the space role |
**space_permissions** | **Vec<String>** | The ids of the space permissions associated with the space role. Sample value \"read/space\"; retrieve ids from responses returned by [GET /space-permissions](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-space-permissions/#api-space-permissions-get) endpoint |
**anonymous_reassignment_role_id** | Option<**String**> | If space anonymous access is assigned to the role being modified, the Id of a role to migrate those assignments to can be specified. Anonymous access role assignments left unchanged if unspecified. | [optional]
**guest_reassignment_role_id** | Option<**String**> | If guests are assigned to the role being modified, the Id of a role to migrate those assignments to can be specified. Guest role assignments left unchanged if unspecified. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
