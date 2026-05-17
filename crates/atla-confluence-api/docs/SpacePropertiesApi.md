# \SpacePropertiesApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_space_property**](SpacePropertiesApi.md#create_space_property) | **POST** /spaces/{space_id}/properties | Create space property in space
[**delete_space_property_by_id**](SpacePropertiesApi.md#delete_space_property_by_id) | **DELETE** /spaces/{space_id}/properties/{property_id} | Delete space property by id
[**get_space_properties**](SpacePropertiesApi.md#get_space_properties) | **GET** /spaces/{space_id}/properties | Get space properties in space
[**get_space_property_by_id**](SpacePropertiesApi.md#get_space_property_by_id) | **GET** /spaces/{space_id}/properties/{property_id} | Get space property by id
[**update_space_property_by_id**](SpacePropertiesApi.md#update_space_property_by_id) | **PUT** /spaces/{space_id}/properties/{property_id} | Update space property by id



## create_space_property

> models::SpaceProperty create_space_property(space_id, space_property_create_request)
Create space property in space

Creates a new space property.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission) and 'Admin' permission for the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**space_id** | **i64** | The ID of the space for which space properties should be returned. | [required] |
**space_property_create_request** | [**SpacePropertyCreateRequest**](SpacePropertyCreateRequest.md) | The space property to be created | [required] |

### Return type

[**models::SpaceProperty**](SpaceProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_space_property_by_id

> delete_space_property_by_id(space_id, property_id)
Delete space property by id

Deletes a space property by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission) and 'Admin' permission for the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**space_id** | **i64** | The ID of the space the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_space_properties

> models::MultiEntityResultSpaceProperty get_space_properties(space_id, key, cursor, limit)
Get space properties in space

Returns all properties for the given space. Space properties are a key-value storage associated with a space. The limit parameter specifies the maximum number of results returned in a single response. Use the `link` response header to paginate through additional results.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission) and 'View' permission for the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**space_id** | **i64** | The ID of the space for which space properties should be returned. | [required] |
**key** | Option<**String**> | The key of the space property to retrieve. This should be used when a user knows the key of their property, but needs to retrieve the id for use in other methods. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of pages per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultSpaceProperty**](MultiEntityResult_SpaceProperty_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_space_property_by_id

> models::SpaceProperty get_space_property_by_id(space_id, property_id)
Get space property by id

Retrieve a space property by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission) and 'View' permission for the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**space_id** | **i64** | The ID of the space the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be retrieved. | [required] |

### Return type

[**models::SpaceProperty**](SpaceProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_space_property_by_id

> models::SpaceProperty update_space_property_by_id(space_id, property_id, space_property_update_request)
Update space property by id

Update a space property by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission) and 'Admin' permission for the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**space_id** | **i64** | The ID of the space the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be updated. | [required] |
**space_property_update_request** | [**SpacePropertyUpdateRequest**](SpacePropertyUpdateRequest.md) | The space property to be updated. | [required] |

### Return type

[**models::SpaceProperty**](SpaceProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
