# \DatabaseApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_database**](DatabaseApi.md#create_database) | **POST** /databases | Create database
[**delete_database**](DatabaseApi.md#delete_database) | **DELETE** /databases/{id} | Delete database
[**get_database_by_id**](DatabaseApi.md#get_database_by_id) | **GET** /databases/{id} | Get database by id



## create_database

> models::CreateDatabase200Response create_database(create_database_request, private)
Create database

Creates a database in the space.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the corresponding space. Permission to create a database in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_database_request** | [**CreateDatabaseRequest**](CreateDatabaseRequest.md) |  | [required] |
**private** | Option<**bool**> | The database will be private. Only the user who creates this database will have permission to view and edit one. |  |[default to false]

### Return type

[**models::CreateDatabase200Response**](createDatabase_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_database

> delete_database(id)
Delete database

Delete a database by id.  Deleting a database moves the database to the trash, where it can be restored later  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the database and its corresponding space. Permission to delete databases in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the database to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_database_by_id

> models::CreateDatabase200Response get_database_by_id(id, include_collaborators, include_direct_children, include_operations, include_properties)
Get database by id

Returns a specific database.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the database and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the database to be returned | [required] |
**include_collaborators** | Option<**bool**> | Includes collaborators on the database. |  |[default to false]
**include_direct_children** | Option<**bool**> | Includes direct children of the database, as defined in the `ChildrenResponse` object. |  |[default to false]
**include_operations** | Option<**bool**> | Includes operations associated with this database in the response, as defined in the `Operation` object. The number of results will be limited to 50 and sorted in the default sort order. A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_properties** | Option<**bool**> | Includes content properties associated with this database in the response. The number of results will be limited to 50 and sorted in the default sort order. A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]

### Return type

[**models::CreateDatabase200Response**](createDatabase_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
