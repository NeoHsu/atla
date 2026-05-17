# \CustomContentApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_custom_content**](CustomContentApi.md#create_custom_content) | **POST** /custom-content | Create custom content
[**delete_custom_content**](CustomContentApi.md#delete_custom_content) | **DELETE** /custom-content/{id} | Delete custom content
[**get_custom_content_by_id**](CustomContentApi.md#get_custom_content_by_id) | **GET** /custom-content/{id} | Get custom content by id
[**get_custom_content_by_type**](CustomContentApi.md#get_custom_content_by_type) | **GET** /custom-content | Get custom content by type
[**get_custom_content_by_type_in_blog_post**](CustomContentApi.md#get_custom_content_by_type_in_blog_post) | **GET** /blogposts/{id}/custom-content | Get custom content by type in blog post
[**get_custom_content_by_type_in_page**](CustomContentApi.md#get_custom_content_by_type_in_page) | **GET** /pages/{id}/custom-content | Get custom content by type in page
[**get_custom_content_by_type_in_space**](CustomContentApi.md#get_custom_content_by_type_in_space) | **GET** /spaces/{id}/custom-content | Get custom content by type in space
[**update_custom_content**](CustomContentApi.md#update_custom_content) | **PUT** /custom-content/{id} | Update custom content



## create_custom_content

> models::CreateCustomContent201Response create_custom_content(create_custom_content_request)
Create custom content

Creates a new custom content in the given space, page, blogpost or other custom content.  Only one of `spaceId`, `pageId`, `blogPostId`, or `customContentId` is required in the request body. **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page or blogpost and its corresponding space. Permission to create custom content in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_custom_content_request** | [**CreateCustomContentRequest**](CreateCustomContentRequest.md) |  | [required] |

### Return type

[**models::CreateCustomContent201Response**](createCustomContent_201_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_custom_content

> delete_custom_content(id, purge)
Delete custom content

Delete a custom content by id.  Deleting a custom content will either move it to the trash or permanently delete it (purge it), depending on the apiSupport. To permanently delete a **trashed** custom content, the endpoint must be called with the following param `purge=true`.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page or blogpost and its corresponding space. Permission to delete custom content in the space. Permission to administer the space (if attempting to purge).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the custom content to be deleted. | [required] |
**purge** | Option<**bool**> | If attempting to purge the custom content. |  |[default to false]

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_custom_content_by_id

> models::CreateCustomContent201Response get_custom_content_by_id(id, body_format, version, include_labels, include_properties, include_operations, include_versions, include_version, include_collaborators)
Get custom content by id

Returns a specific piece of custom content.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the custom content, the container of the custom content, and the corresponding space (if different from the container).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the custom content to be returned. If you don't know the custom content ID, use Get Custom Content by Type and filter the results. | [required] |
**body_format** | Option<[**CustomContentBodyRepresentationSingle**](CustomContentBodyRepresentationSingle.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field.  Note: If the custom content body type is `storage`, the `storage` and `atlas_doc_format` body formats are able to be returned. If the custom content body type is `raw`, only the `raw` body format is able to be returned. |  |
**version** | Option<**i32**> | Allows you to retrieve a previously published version. Specify the previous version's number to retrieve its details. |  |
**include_labels** | Option<**bool**> | Includes labels associated with this custom content in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_properties** | Option<**bool**> | Includes content properties associated with this custom content in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_operations** | Option<**bool**> | Includes operations associated with this custom content in the response, as defined in the `Operation` object. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_versions** | Option<**bool**> | Includes versions associated with this custom content in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_version** | Option<**bool**> | Includes the current version associated with this custom content in the response. By default this is included and can be omitted by setting the value to `false`. |  |[default to true]
**include_collaborators** | Option<**bool**> | Includes collaborators on the custom content. |  |[default to false]

### Return type

[**models::CreateCustomContent201Response**](createCustomContent_201_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_custom_content_by_type

> models::MultiEntityResultCustomContent get_custom_content_by_type(r#type, id, space_id, sort, cursor, limit, body_format)
Get custom content by type

Returns all custom content for a given type. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the custom content, the container of the custom content, and the corresponding space (if different from the container).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**r#type** | **String** | The type of custom content being requested. See: https://developer.atlassian.com/cloud/confluence/custom-content/ for additional details on custom content. | [required] |
**id** | Option<[**Vec<i64>**](I64.md)> | Filter the results based on custom content ids. Multiple custom content ids can be specified as a comma-separated list. |  |
**space_id** | Option<[**Vec<i64>**](I64.md)> | Filter the results based on space ids. Multiple space ids can be specified as a comma-separated list. |  |
**sort** | Option<[**CustomContentSortOrder**](CustomContentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of pages per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**body_format** | Option<[**CustomContentBodyRepresentation**](CustomContentBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field.  Note: If the custom content body type is `storage`, the `storage` and `atlas_doc_format` body formats are able to be returned. If the custom content body type is `raw`, only the `raw` body format is able to be returned. |  |

### Return type

[**models::MultiEntityResultCustomContent**](MultiEntityResult_CustomContent_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_custom_content_by_type_in_blog_post

> models::MultiEntityResultCustomContent get_custom_content_by_type_in_blog_post(id, r#type, sort, cursor, limit, body_format)
Get custom content by type in blog post

Returns all custom content for a given type within a given blogpost. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the custom content, the container of the custom content (blog post), and the corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post for which custom content should be returned. | [required] |
**r#type** | **String** | The type of custom content being requested. See: https://developer.atlassian.com/cloud/confluence/custom-content/ for additional details on custom content. | [required] |
**sort** | Option<[**CustomContentSortOrder**](CustomContentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of pages per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**body_format** | Option<[**CustomContentBodyRepresentation**](CustomContentBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field.  Note: If the custom content body type is `storage`, the `storage` and `atlas_doc_format` body formats are able to be returned. If the custom content body type is `raw`, only the `raw` body format is able to be returned. |  |

### Return type

[**models::MultiEntityResultCustomContent**](MultiEntityResult_CustomContent_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_custom_content_by_type_in_page

> models::MultiEntityResultCustomContent get_custom_content_by_type_in_page(id, r#type, sort, cursor, limit, body_format)
Get custom content by type in page

Returns all custom content for a given type within a given page. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the custom content, the container of the custom content (page), and the corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page for which custom content should be returned. | [required] |
**r#type** | **String** | The type of custom content being requested. See: https://developer.atlassian.com/cloud/confluence/custom-content/ for additional details on custom content. | [required] |
**sort** | Option<[**CustomContentSortOrder**](CustomContentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of pages per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**body_format** | Option<[**CustomContentBodyRepresentation**](CustomContentBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field.  Note: If the custom content body type is `storage`, the `storage` and `atlas_doc_format` body formats are able to be returned. If the custom content body type is `raw`, only the `raw` body format is able to be returned. |  |

### Return type

[**models::MultiEntityResultCustomContent**](MultiEntityResult_CustomContent_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_custom_content_by_type_in_space

> models::MultiEntityResultCustomContent get_custom_content_by_type_in_space(id, r#type, cursor, limit, body_format)
Get custom content by type in space

Returns all custom content for a given type within a given space. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the custom content and the corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the space for which custom content should be returned. | [required] |
**r#type** | **String** | The type of custom content being requested. See: https://developer.atlassian.com/cloud/confluence/custom-content/ for additional details on custom content. | [required] |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of pages per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**body_format** | Option<[**CustomContentBodyRepresentation**](CustomContentBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field.  Note: If the custom content body type is `storage`, the `storage` and `atlas_doc_format` body formats are able to be returned. If the custom content body type is `raw`, only the `raw` body format is able to be returned. |  |

### Return type

[**models::MultiEntityResultCustomContent**](MultiEntityResult_CustomContent_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_custom_content

> models::CreateCustomContent201Response update_custom_content(id, update_custom_content_request)
Update custom content

Update a custom content by id. At most one of `spaceId`, `pageId`, `blogPostId`, or `customContentId` is allowed in the request body. Note that if `spaceId` is specified, it must be the same as the `spaceId` used for creating the custom content as moving custom content to a different space is not supported.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page or blogpost and its corresponding space. Permission to update custom content in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the custom content to be updated. If you don't know the custom content ID, use Get Custom Content by Type and filter the results. | [required] |
**update_custom_content_request** | [**UpdateCustomContentRequest**](UpdateCustomContentRequest.md) |  | [required] |

### Return type

[**models::CreateCustomContent201Response**](createCustomContent_201_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
