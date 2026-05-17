# \TaskApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_task_by_id**](TaskApi.md#get_task_by_id) | **GET** /tasks/{id} | Get task by id
[**get_tasks**](TaskApi.md#get_tasks) | **GET** /tasks | Get tasks
[**update_task**](TaskApi.md#update_task) | **PUT** /tasks/{id} | Update task



## get_task_by_id

> models::Task get_task_by_id(id, body_format)
Get task by id

Returns a specific task.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the containing page or blog post and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the task to be returned. If you don't know the task ID, use Get tasks and filter the results. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |

### Return type

[**models::Task**](Task.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tasks

> models::MultiEntityResultTask get_tasks(body_format, include_blank_tasks, status, task_id, space_id, page_id, blogpost_id, created_by, assigned_to, completed_by, created_at_from, created_at_to, due_at_from, due_at_to, completed_at_from, completed_at_to, cursor, limit)
Get tasks

Returns all tasks. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Only tasks that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**include_blank_tasks** | Option<**bool**> | Specifies whether to include blank tasks in the response. Defaults to `true`. |  |
**status** | Option<**String**> | Filters on the status of the task. |  |
**task_id** | Option<[**Vec<i64>**](I64.md)> | Filters on task ID. Multiple IDs can be specified. |  |
**space_id** | Option<[**Vec<i64>**](I64.md)> | Filters on the space ID of the task. Multiple IDs can be specified. |  |
**page_id** | Option<[**Vec<i64>**](I64.md)> | Filters on the page ID of the task. Multiple IDs can be specified. Note - page and blog post filters can be used in conjunction. |  |
**blogpost_id** | Option<[**Vec<i64>**](I64.md)> | Filters on the blog post ID of the task. Multiple IDs can be specified. Note - page and blog post filters can be used in conjunction. |  |
**created_by** | Option<[**Vec<String>**](String.md)> | Filters on the Account ID of the user who created this task. Multiple IDs can be specified. |  |
**assigned_to** | Option<[**Vec<String>**](String.md)> | Filters on the Account ID of the user to whom this task is assigned. Multiple IDs can be specified. |  |
**completed_by** | Option<[**Vec<String>**](String.md)> | Filters on the Account ID of the user who completed this task. Multiple IDs can be specified. |  |
**created_at_from** | Option<**i64**> | Filters on start of date-time range of task based on creation date (inclusive). Input is epoch time in milliseconds. |  |
**created_at_to** | Option<**i64**> | Filters on end of date-time range of task based on creation date (inclusive). Input is epoch time in milliseconds. |  |
**due_at_from** | Option<**i64**> | Filters on start of date-time range of task based on due date (inclusive). Input is epoch time in milliseconds. |  |
**due_at_to** | Option<**i64**> | Filters on end of date-time range of task based on due date (inclusive). Input is epoch time in milliseconds. |  |
**completed_at_from** | Option<**i64**> | Filters on start of date-time range of task based on completion date (inclusive). Input is epoch time in milliseconds. |  |
**completed_at_to** | Option<**i64**> | Filters on end of date-time range of task based on completion date (inclusive). Input is epoch time in milliseconds. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of tasks per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultTask**](MultiEntityResult_Task_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_task

> models::Task update_task(id, update_task_request, body_format)
Update task

Update a task by id. This endpoint currently only supports updating task status.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the containing page or blog post and view its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the task to be updated. If you don't know the task ID, use Get tasks and filter the results. | [required] |
**update_task_request** | [**UpdateTaskRequest**](UpdateTaskRequest.md) |  | [required] |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |

### Return type

[**models::Task**](Task.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
