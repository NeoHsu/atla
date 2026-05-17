# \AttachmentApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_attachment**](AttachmentApi.md#delete_attachment) | **DELETE** /attachments/{id} | Delete attachment
[**get_attachment_by_id**](AttachmentApi.md#get_attachment_by_id) | **GET** /attachments/{id} | Get attachment by id
[**get_attachment_thumbnail_by_id**](AttachmentApi.md#get_attachment_thumbnail_by_id) | **GET** /attachments/{id}/thumbnail/download | Download attachment thumbnail by id
[**get_attachments**](AttachmentApi.md#get_attachments) | **GET** /attachments | Get attachments
[**get_blogpost_attachments**](AttachmentApi.md#get_blogpost_attachments) | **GET** /blogposts/{id}/attachments | Get attachments for blog post
[**get_custom_content_attachments**](AttachmentApi.md#get_custom_content_attachments) | **GET** /custom-content/{id}/attachments | Get attachments for custom content
[**get_label_attachments**](AttachmentApi.md#get_label_attachments) | **GET** /labels/{id}/attachments | Get attachments for label
[**get_page_attachments**](AttachmentApi.md#get_page_attachments) | **GET** /pages/{id}/attachments | Get attachments for page



## delete_attachment

> delete_attachment(id, purge)
Delete attachment

Delete an attachment by id.  Deleting an attachment moves the attachment to the trash, where it can be restored later. To permanently delete an attachment (or \"purge\" it), the endpoint must be called on a **trashed** attachment with the following param `purge=true`.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the container of the attachment. Permission to delete attachments in the space. Permission to administer the space (if attempting to purge).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the attachment to be deleted. | [required] |
**purge** | Option<**bool**> | If attempting to purge the attachment. |  |[default to false]

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_attachment_by_id

> models::GetAttachmentById200Response get_attachment_by_id(id, version, include_labels, include_properties, include_operations, include_versions, include_version, include_collaborators)
Get attachment by id

Returns a specific attachment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the attachment's container.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the attachment to be returned. If you don't know the attachment's ID, use Get attachments for page/blogpost/custom content. | [required] |
**version** | Option<**i32**> | Allows you to retrieve a previously published version. Specify the previous version's number to retrieve its details. |  |
**include_labels** | Option<**bool**> | Includes labels associated with this attachment in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_properties** | Option<**bool**> | Includes content properties associated with this attachment in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_operations** | Option<**bool**> | Includes operations associated with this attachment in the response, as defined in the `Operation` object. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_versions** | Option<**bool**> | Includes versions associated with this attachment in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_version** | Option<**bool**> | Includes the current version associated with this attachment in the response. By default this is included and can be omitted by setting the value to `false`. |  |[default to true]
**include_collaborators** | Option<**bool**> | Includes collaborators on the attachment. |  |[default to false]

### Return type

[**models::GetAttachmentById200Response**](getAttachmentById_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_attachment_thumbnail_by_id

> get_attachment_thumbnail_by_id(id, version, height, width)
Download attachment thumbnail by id

Redirects the client to a URL that serves an attachment thumbnail's binary data.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the attachment's container.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the attachment to be returned. If you don't know the attachment's ID, use Get attachments for page/blogpost/custom content. | [required] |
**version** | Option<**i32**> | Allows you to retrieve a previously published version. Specify the previous version's number to retrieve its details. |  |
**height** | Option<**i32**> | Allows you to define the thumbnail height. |  |
**width** | Option<**i32**> | Allows you to define the thumbnail width. |  |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_attachments

> models::MultiEntityResultAttachment get_attachments(sort, cursor, status, media_type, filename, limit)
Get attachments

Returns all attachments. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the container of the attachment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**sort** | Option<[**AttachmentSortOrder**](AttachmentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**status** | Option<[**Vec<String>**](String.md)> | Filter the results to attachments based on their status. By default, `current` and `archived` are used. |  |
**media_type** | Option<**String**> | Filters on the mediaType of attachments. Only one may be specified. |  |
**filename** | Option<**String**> | Filters on the file-name of attachments. Only one may be specified. |  |
**limit** | Option<**i32**> | Maximum number of attachments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 50]

### Return type

[**models::MultiEntityResultAttachment**](MultiEntityResult_Attachment_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_blogpost_attachments

> models::MultiEntityResultAttachment get_blogpost_attachments(id, sort, cursor, status, media_type, filename, limit)
Get attachments for blog post

Returns the attachments of specific blog post. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the blog post and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post for which attachments should be returned. | [required] |
**sort** | Option<[**AttachmentSortOrder**](AttachmentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**status** | Option<[**Vec<String>**](String.md)> | Filter the results to attachments based on their status. By default, `current` and `archived` are used. |  |
**media_type** | Option<**String**> | Filters on the mediaType of attachments. Only one may be specified. |  |
**filename** | Option<**String**> | Filters on the file-name of attachments. Only one may be specified. |  |
**limit** | Option<**i32**> | Maximum number of attachments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 50]

### Return type

[**models::MultiEntityResultAttachment**](MultiEntityResult_Attachment_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_custom_content_attachments

> models::MultiEntityResultAttachment get_custom_content_attachments(id, sort, cursor, status, media_type, filename, limit)
Get attachments for custom content

Returns the attachments of specific custom content. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the custom content and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the custom content for which attachments should be returned. | [required] |
**sort** | Option<[**AttachmentSortOrder**](AttachmentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**status** | Option<[**Vec<String>**](String.md)> | Filter the results to attachments based on their status. By default, `current` and `archived` are used. |  |
**media_type** | Option<**String**> | Filters on the mediaType of attachments. Only one may be specified. |  |
**filename** | Option<**String**> | Filters on the file-name of attachments. Only one may be specified. |  |
**limit** | Option<**i32**> | Maximum number of attachments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 50]

### Return type

[**models::MultiEntityResultAttachment**](MultiEntityResult_Attachment_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_label_attachments

> models::MultiEntityResultAttachment get_label_attachments(id, sort, cursor, limit)
Get attachments for label

Returns the attachments of specified label. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the attachment and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the label for which attachments should be returned. | [required] |
**sort** | Option<[**AttachmentSortOrder**](AttachmentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of pages per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultAttachment**](MultiEntityResult_Attachment_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_attachments

> models::MultiEntityResultAttachment get_page_attachments(id, sort, cursor, status, media_type, filename, limit)
Get attachments for page

Returns the attachments of specific page. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page for which attachments should be returned. | [required] |
**sort** | Option<[**AttachmentSortOrder**](AttachmentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**status** | Option<[**Vec<String>**](String.md)> | Filter the results to attachments based on their status. By default, `current` and `archived` are used. |  |
**media_type** | Option<**String**> | Filters on the mediaType of attachments. Only one may be specified. |  |
**filename** | Option<**String**> | Filters on the file-name of attachments. Only one may be specified. |  |
**limit** | Option<**i32**> | Maximum number of attachments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 50]

### Return type

[**models::MultiEntityResultAttachment**](MultiEntityResult_Attachment_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
