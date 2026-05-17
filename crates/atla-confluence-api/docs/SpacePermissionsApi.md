# \SpacePermissionsApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_available_space_permissions**](SpacePermissionsApi.md#get_available_space_permissions) | **GET** /space-permissions | Get available space permissions
[**get_space_permissions_assignments**](SpacePermissionsApi.md#get_space_permissions_assignments) | **GET** /spaces/{id}/permissions | Get space permissions assignments



## get_available_space_permissions

> models::MultiEntityResultSpacePermission get_available_space_permissions(cursor, limit)
Get available space permissions

Retrieves the available space permissions.  Available on tenants with [Role-Based Access Control](https://support.atlassian.com/confluence-cloud/docs/manage-user-roles/).   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of space permissions to return. If more results exist, use the `Link` response header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultSpacePermission**](MultiEntityResult_SpacePermission_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_space_permissions_assignments

> models::MultiEntityResultSpacePermissionAssignment get_space_permissions_assignments(id, cursor, limit)
Get space permissions assignments

Returns space permission assignments for a specific space.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the space to be returned. | [required] |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of assignments to return. If more results exist, use the `Link` response header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultSpacePermissionAssignment**](MultiEntityResult_SpacePermissionAssignment_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
