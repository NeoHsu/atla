# \SpaceApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_space**](SpaceApi.md#create_space) | **POST** /spaces | Create space
[**get_space_by_id**](SpaceApi.md#get_space_by_id) | **GET** /spaces/{id} | Get space by id
[**get_spaces**](SpaceApi.md#get_spaces) | **GET** /spaces | Get spaces



## create_space

> models::CreateSpace201Response create_space(create_space_request)
Create space

Creates a Space as specified in the payload.  Available on tenants with [Role-Based Access Control](https://support.atlassian.com/confluence-cloud/docs/manage-user-roles/).   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to create spaces.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_space_request** | [**CreateSpaceRequest**](CreateSpaceRequest.md) |  | [required] |

### Return type

[**models::CreateSpace201Response**](createSpace_201_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_space_by_id

> models::GetSpaceById200Response get_space_by_id(id, description_format, include_icon, include_operations, include_properties, include_permissions, include_role_assignments, include_labels)
Get space by id

Returns a specific space.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the space to be returned. | [required] |
**description_format** | Option<[**SpaceDescriptionBodyRepresentation**](SpaceDescriptionBodyRepresentation.md)> | The content format type to be returned in the `description` field of the response. If available, the representation will be available under a response field of the same name under the `description` field. |  |
**include_icon** | Option<**bool**> | If the icon for the space should be fetched or not. |  |[default to false]
**include_operations** | Option<**bool**> | Includes operations associated with this space in the response, as defined in the `Operation` object. The number of results will be limited to 50 and sorted in the default sort order. A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_properties** | Option<**bool**> | Includes space properties associated with this space in the response. The number of results will be limited to 50 and sorted in the default sort order. A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_permissions** | Option<**bool**> | Includes space permissions associated with this space in the response. The number of results will be limited to 50 and sorted in the default sort order. A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_role_assignments** | Option<**bool**> | Includes role assignments associated with this space in the response. This parameter is only accepted for EAP sites. The number of results will be limited to 50 and sorted in the default sort order. A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_labels** | Option<**bool**> | Includes labels associated with this space in the response. The number of results will be limited to 50 and sorted in the default sort order. A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]

### Return type

[**models::GetSpaceById200Response**](getSpaceById_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_spaces

> models::MultiEntityResultSpace get_spaces(ids, keys, r#type, status, labels, favorited_by, not_favorited_by, sort, description_format, include_icon, cursor, limit)
Get spaces

Returns all spaces. The results will be sorted by id ascending. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Only spaces that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ids** | Option<[**Vec<i64>**](I64.md)> | Filter the results to spaces based on their IDs. Multiple IDs can be specified as a comma-separated list. |  |
**keys** | Option<[**Vec<String>**](String.md)> | Filter the results to spaces based on their keys. Multiple keys can be specified as a comma-separated list. |  |
**r#type** | Option<**String**> | Filter the results to spaces based on their type. |  |
**status** | Option<**String**> | Filter the results to spaces based on their status. |  |
**labels** | Option<[**Vec<String>**](String.md)> | Filter the results to spaces based on their labels. Multiple labels can be specified as a comma-separated list. |  |
**favorited_by** | Option<**String**> | Filter the results to spaces favorited by the user with the specified account ID. |  |
**not_favorited_by** | Option<**String**> | Filter the results to spaces NOT favorited by the user with the specified account ID. |  |
**sort** | Option<[**SpaceSortOrder**](SpaceSortOrder.md)> | Used to sort the result by a particular field. |  |
**description_format** | Option<[**SpaceDescriptionBodyRepresentation**](SpaceDescriptionBodyRepresentation.md)> | The content format type to be returned in the `description` field of the response. If available, the representation will be available under a response field of the same name under the `description` field. |  |
**include_icon** | Option<**bool**> | If the icon for the space should be fetched or not. |  |[default to false]
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of spaces per result to return. If more results exist, use the `Link` response header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultSpace**](MultiEntityResult_Space_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
