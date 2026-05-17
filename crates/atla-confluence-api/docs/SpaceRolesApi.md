# \SpaceRolesApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_space_role**](SpaceRolesApi.md#create_space_role) | **POST** /space-roles | Create a space role
[**delete_space_role**](SpaceRolesApi.md#delete_space_role) | **DELETE** /space-roles/{id} | Delete a space role
[**get_available_space_roles**](SpaceRolesApi.md#get_available_space_roles) | **GET** /space-roles | Get available space roles
[**get_space_role_assignments**](SpaceRolesApi.md#get_space_role_assignments) | **GET** /spaces/{id}/role-assignments | Get space role assignments
[**get_space_role_mode**](SpaceRolesApi.md#get_space_role_mode) | **GET** /space-role-mode | Get space role mode
[**get_space_roles_by_id**](SpaceRolesApi.md#get_space_roles_by_id) | **GET** /space-roles/{id} | Get space role by ID
[**set_space_role_assignments**](SpaceRolesApi.md#set_space_role_assignments) | **POST** /spaces/{id}/role-assignments | Set space role assignments
[**update_space_role**](SpaceRolesApi.md#update_space_role) | **PUT** /space-roles/{id} | Update a space role



## create_space_role

> models::SpaceRole create_space_role(create_space_role_request)
Create a space role

Create a space role.  Available on tenants with [Role-Based Access Control](https://support.atlassian.com/confluence-cloud/docs/manage-user-roles/).   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: User must be an organization or site admin. Connect and Forge app users are not authorized to access this resource.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_space_role_request** | [**CreateSpaceRoleRequest**](CreateSpaceRoleRequest.md) |  | [required] |

### Return type

[**models::SpaceRole**](SpaceRole.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_space_role

> models::DeleteSpaceRoleResponse delete_space_role(id)
Delete a space role

Delete a space role  Available on tenants with [Role-Based Access Control](https://support.atlassian.com/confluence-cloud/docs/manage-user-roles/).   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: User must be an organization or site admin. Connect and Forge app users are not authorized to access this resource.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Id of the space role | [required] |

### Return type

[**models::DeleteSpaceRoleResponse**](DeleteSpaceRoleResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_available_space_roles

> models::MultiEntityResultSpaceRole get_available_space_roles(space_id, role_type, principal_id, principal_type, cursor, limit)
Get available space roles

Retrieves the available space roles.  Available on tenants with [Role-Based Access Control](https://support.atlassian.com/confluence-cloud/docs/manage-user-roles/).   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site; if requesting a certain space's roles, permission to view the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**space_id** | Option<**String**> | The space ID for which to filter available space roles; if empty, return all available space roles for the tenant. |  |
**role_type** | Option<**String**> | The space role type to filter results by. |  |
**principal_id** | Option<**String**> | The principal ID to filter results by. If specified, a principal-type must also be specified. Paired with a `principal-type` of `ACCESS_CLASS`, valid values include [`anonymous-users`, `jsm-project-admins`, `authenticated-users`, `all-licensed-users`, `all-product-admins`] |  |
**principal_type** | Option<[**PrincipalType**](PrincipalType.md)> | The principal type to filter results by. If specified, a principal-id must also be specified. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of space roles to return. If more results exist, use the `Link` response header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultSpaceRole**](MultiEntityResult_SpaceRole_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_space_role_assignments

> models::MultiEntityResultSpaceRoleAssignment get_space_role_assignments(id, role_id, role_type, principal_id, principal_type, cursor, limit)
Get space role assignments

Retrieves the space role assignments.  Available on tenants with [Role-Based Access Control](https://support.atlassian.com/confluence-cloud/docs/manage-user-roles/).   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i32** | The ID of the space for which to retrieve assignments. | [required] |
**role_id** | Option<**String**> | Filters the returned role assignments to the provided role ID. |  |
**role_type** | Option<**String**> | Filters the returned role assignments to the provided role type. |  |
**principal_id** | Option<**String**> | Filters the returned role assignments to the provided principal id. If specified, a principal-type must also be specified. Paired with a `principal-type` of `ACCESS_CLASS`, valid values include [`anonymous-users`, `jsm-project-admins`, `authenticated-users`, `all-licensed-users`, `all-product-admins`] |  |
**principal_type** | Option<[**PrincipalType**](PrincipalType.md)> | Filters the returned role assignments to the provided principal type. If specified, a principal-id must also be specified. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of space roles to return. If more results exist, use the `Link` response header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultSpaceRoleAssignment**](MultiEntityResult_SpaceRoleAssignment_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_space_role_mode

> models::GetSpaceRoleMode200Response get_space_role_mode()
Get space role mode

Retrieves the space role mode.  Available on tenants with [Role-Based Access Control](https://support.atlassian.com/confluence-cloud/docs/manage-user-roles/).   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission).

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetSpaceRoleMode200Response**](getSpaceRoleMode_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_space_roles_by_id

> models::GetSpaceRolesById200Response get_space_roles_by_id(id)
Get space role by ID

Retrieves the space role by ID.  Available on tenants with [Role-Based Access Control](https://support.atlassian.com/confluence-cloud/docs/manage-user-roles/).   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i32** | The ID of the space role to retrieve. | [required] |

### Return type

[**models::GetSpaceRolesById200Response**](getSpaceRolesById_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_space_role_assignments

> models::MultiEntityResultSpaceRoleAssignment set_space_role_assignments(id, set_space_role_assignments_request_inner)
Set space role assignments

Sets space role assignments as specified in the payload.  Available on tenants with [Role-Based Access Control](https://support.atlassian.com/confluence-cloud/docs/manage-user-roles/).   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to manage roles in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i32** | The ID of the space for which to retrieve assignments. | [required] |
**set_space_role_assignments_request_inner** | [**Vec<models::SetSpaceRoleAssignmentsRequestInner>**](SetSpaceRoleAssignmentsRequestInner.md) |  | [required] |

### Return type

[**models::MultiEntityResultSpaceRoleAssignment**](MultiEntityResult_SpaceRoleAssignment_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_space_role

> models::UpdateSpaceRoleResponse update_space_role(id, update_space_role_request)
Update a space role

Update a space role.  Available on tenants with [Role-Based Access Control](https://support.atlassian.com/confluence-cloud/docs/manage-user-roles/).   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: User must be an organization or site admin. Connect and Forge app users are not authorized to access this resource.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Id of the space role | [required] |
**update_space_role_request** | [**UpdateSpaceRoleRequest**](UpdateSpaceRoleRequest.md) |  | [required] |

### Return type

[**models::UpdateSpaceRoleResponse**](UpdateSpaceRoleResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
