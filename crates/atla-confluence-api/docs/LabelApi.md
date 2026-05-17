# \LabelApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_attachment_labels**](LabelApi.md#get_attachment_labels) | **GET** /attachments/{id}/labels | Get labels for attachment
[**get_blog_post_labels**](LabelApi.md#get_blog_post_labels) | **GET** /blogposts/{id}/labels | Get labels for blog post
[**get_custom_content_labels**](LabelApi.md#get_custom_content_labels) | **GET** /custom-content/{id}/labels | Get labels for custom content
[**get_labels**](LabelApi.md#get_labels) | **GET** /labels | Get labels
[**get_page_labels**](LabelApi.md#get_page_labels) | **GET** /pages/{id}/labels | Get labels for page
[**get_space_content_labels**](LabelApi.md#get_space_content_labels) | **GET** /spaces/{id}/content/labels | Get labels for space content
[**get_space_labels**](LabelApi.md#get_space_labels) | **GET** /spaces/{id}/labels | Get labels for space



## get_attachment_labels

> models::MultiEntityResultLabel get_attachment_labels(id, prefix, sort, cursor, limit)
Get labels for attachment

Returns the labels of specific attachment. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the parent content of the attachment and its corresponding space. Only labels that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the attachment for which labels should be returned. | [required] |
**prefix** | Option<**String**> | Filter the results to labels based on their prefix. |  |
**sort** | Option<[**Vec<models::LabelSortOrder>**](Models__LabelSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of labels per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultLabel**](MultiEntityResult_Label_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_blog_post_labels

> models::MultiEntityResultLabel get_blog_post_labels(id, prefix, sort, cursor, limit)
Get labels for blog post

Returns the labels of specific blog post. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the blog post and its corresponding space. Only labels that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post for which labels should be returned. | [required] |
**prefix** | Option<**String**> | Filter the results to labels based on their prefix. |  |
**sort** | Option<[**Vec<models::LabelSortOrder>**](Models__LabelSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of labels per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultLabel**](MultiEntityResult_Label_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_custom_content_labels

> models::MultiEntityResultLabel get_custom_content_labels(id, prefix, sort, cursor, limit)
Get labels for custom content

Returns the labels for a specific piece of custom content. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the custom content and its corresponding space. Only labels that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the custom content for which labels should be returned. | [required] |
**prefix** | Option<**String**> | Filter the results to labels based on their prefix. |  |
**sort** | Option<[**Vec<models::LabelSortOrder>**](Models__LabelSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of labels per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultLabel**](MultiEntityResult_Label_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_labels

> models::MultiEntityResultLabel get_labels(label_id, prefix, cursor, sort, limit)
Get labels

Returns all labels. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Only labels that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**label_id** | Option<[**Vec<i64>**](I64.md)> | Filters on label ID. Multiple IDs can be specified as a comma-separated list. |  |
**prefix** | Option<[**Vec<String>**](String.md)> | Filters on label prefix. Multiple IDs can be specified as a comma-separated list. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**sort** | Option<[**Vec<models::LabelSortOrder>**](Models__LabelSortOrder.md)> | Used to sort the result by a particular field. |  |
**limit** | Option<**i32**> | Maximum number of labels per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultLabel**](MultiEntityResult_Label_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_labels

> models::MultiEntityResultLabel get_page_labels(id, prefix, sort, cursor, limit)
Get labels for page

Returns the labels of specific page. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page and its corresponding space. Only labels that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page for which labels should be returned. | [required] |
**prefix** | Option<**String**> | Filter the results to labels based on their prefix. |  |
**sort** | Option<[**Vec<models::LabelSortOrder>**](Models__LabelSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of labels per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultLabel**](MultiEntityResult_Label_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_space_content_labels

> models::MultiEntityResultLabel get_space_content_labels(id, prefix, sort, cursor, limit)
Get labels for space content

Returns the labels of space content (pages, blogposts etc). The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the space. Only labels that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the space for which labels should be returned. | [required] |
**prefix** | Option<**String**> | Filter the results to labels based on their prefix. |  |[default to my, team]
**sort** | Option<[**Vec<models::LabelSortOrder>**](Models__LabelSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of labels per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultLabel**](MultiEntityResult_Label_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_space_labels

> models::MultiEntityResultLabel get_space_labels(id, prefix, sort, cursor, limit)
Get labels for space

Returns the labels of specific space. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the space. Only labels that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the space for which labels should be returned. | [required] |
**prefix** | Option<**String**> | Filter the results to labels based on their prefix. |  |[default to my, team]
**sort** | Option<[**Vec<models::LabelSortOrder>**](Models__LabelSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of labels per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultLabel**](MultiEntityResult_Label_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
