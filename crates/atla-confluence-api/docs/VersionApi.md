# \VersionApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_attachment_version_details**](VersionApi.md#get_attachment_version_details) | **GET** /attachments/{attachment_id}/versions/{version_number} | Get version details for attachment version
[**get_attachment_versions**](VersionApi.md#get_attachment_versions) | **GET** /attachments/{id}/versions | Get attachment versions
[**get_blog_post_version_details**](VersionApi.md#get_blog_post_version_details) | **GET** /blogposts/{blogpost_id}/versions/{version_number} | Get version details for blog post version
[**get_blog_post_versions**](VersionApi.md#get_blog_post_versions) | **GET** /blogposts/{id}/versions | Get blog post versions
[**get_custom_content_version_details**](VersionApi.md#get_custom_content_version_details) | **GET** /custom-content/{custom_content_id}/versions/{version_number} | Get version details for custom content version
[**get_custom_content_versions**](VersionApi.md#get_custom_content_versions) | **GET** /custom-content/{custom_content_id}/versions | Get custom content versions
[**get_footer_comment_version_details**](VersionApi.md#get_footer_comment_version_details) | **GET** /footer-comments/{id}/versions/{version_number} | Get version details for footer comment version
[**get_footer_comment_versions**](VersionApi.md#get_footer_comment_versions) | **GET** /footer-comments/{id}/versions | Get footer comment versions
[**get_inline_comment_version_details**](VersionApi.md#get_inline_comment_version_details) | **GET** /inline-comments/{id}/versions/{version_number} | Get version details for inline comment version
[**get_inline_comment_versions**](VersionApi.md#get_inline_comment_versions) | **GET** /inline-comments/{id}/versions | Get inline comment versions
[**get_page_version_details**](VersionApi.md#get_page_version_details) | **GET** /pages/{page_id}/versions/{version_number} | Get version details for page version
[**get_page_versions**](VersionApi.md#get_page_versions) | **GET** /pages/{id}/versions | Get page versions



## get_attachment_version_details

> models::DetailedVersion get_attachment_version_details(attachment_id, version_number)
Get version details for attachment version

Retrieves version details for the specified attachment and version number.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the attachment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**attachment_id** | **String** | The ID of the attachment for which version details should be returned. | [required] |
**version_number** | **i64** | The version number of the attachment to be returned. | [required] |

### Return type

[**models::DetailedVersion**](DetailedVersion.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_attachment_versions

> models::MultiEntityResultVersion get_attachment_versions(id, cursor, limit, sort)
Get attachment versions

Returns the versions of specific attachment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the attachment and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the attachment to be queried for its versions. If you don't know the attachment ID, use Get attachments and filter the results. | [required] |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of versions per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**VersionSortOrder**](VersionSortOrder.md)> | Used to sort the result by a particular field. |  |

### Return type

[**models::MultiEntityResultVersion**](MultiEntityResult_Version_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_blog_post_version_details

> models::DetailedVersion get_blog_post_version_details(blogpost_id, version_number)
Get version details for blog post version

Retrieves version details for the specified blog post and version number.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the blog post.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**blogpost_id** | **i64** | The ID of the blog post for which version details should be returned. | [required] |
**version_number** | **i64** | The version number of the blog post to be returned. | [required] |

### Return type

[**models::DetailedVersion**](DetailedVersion.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_blog_post_versions

> models::MultiEntityResultVersion1 get_blog_post_versions(id, body_format, cursor, limit, sort)
Get blog post versions

Returns the versions of specific blog post.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the blog post and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post to be queried for its versions. If you don't know the blog post ID, use Get blog posts and filter the results. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of versions per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**VersionSortOrder**](VersionSortOrder.md)> | Used to sort the result by a particular field. |  |

### Return type

[**models::MultiEntityResultVersion1**](MultiEntityResult_Version__1.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_custom_content_version_details

> models::DetailedVersion get_custom_content_version_details(custom_content_id, version_number)
Get version details for custom content version

Retrieves version details for the specified custom content and version number.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the page.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**custom_content_id** | **i64** | The ID of the custom content for which version details should be returned. | [required] |
**version_number** | **i64** | The version number of the custom content to be returned. | [required] |

### Return type

[**models::DetailedVersion**](DetailedVersion.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_custom_content_versions

> models::MultiEntityResultVersion3 get_custom_content_versions(custom_content_id, body_format, cursor, limit, sort)
Get custom content versions

Returns the versions of specific custom content.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the custom content and its corresponding page and space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**custom_content_id** | **i64** | The ID of the custom content to be queried for its versions. If you don't know the custom content ID, use Get custom-content by type and filter the results. | [required] |
**body_format** | Option<[**CustomContentBodyRepresentation**](CustomContentBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field.  Note: If the custom content body type is `storage`, the `storage` and `atlas_doc_format` body formats are able to be returned. If the custom content body type is `raw`, only the `raw` body format is able to be returned. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of versions per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**VersionSortOrder**](VersionSortOrder.md)> | Used to sort the result by a particular field. |  |

### Return type

[**models::MultiEntityResultVersion3**](MultiEntityResult_Version__3.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_footer_comment_version_details

> models::DetailedVersion get_footer_comment_version_details(id, version_number)
Get version details for footer comment version

Retrieves version details for the specified footer comment version.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page or blog post and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the footer comment for which version details should be returned. | [required] |
**version_number** | **i64** | The version number of the footer comment to be returned. | [required] |

### Return type

[**models::DetailedVersion**](DetailedVersion.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_footer_comment_versions

> models::MultiEntityResultVersion4 get_footer_comment_versions(id, body_format, cursor, limit, sort)
Get footer comment versions

Retrieves the versions of the specified footer comment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page or blog post and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the footer comment for which versions should be returned | [required] |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of versions per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**VersionSortOrder**](VersionSortOrder.md)> | Used to sort the result by a particular field. |  |

### Return type

[**models::MultiEntityResultVersion4**](MultiEntityResult_Version__4.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_inline_comment_version_details

> models::DetailedVersion get_inline_comment_version_details(id, version_number)
Get version details for inline comment version

Retrieves version details for the specified inline comment version.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page or blog post and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the inline comment for which version details should be returned. | [required] |
**version_number** | **i64** | The version number of the inline comment to be returned. | [required] |

### Return type

[**models::DetailedVersion**](DetailedVersion.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_inline_comment_versions

> models::MultiEntityResultVersion4 get_inline_comment_versions(id, body_format, cursor, limit, sort)
Get inline comment versions

Retrieves the versions of the specified inline comment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page or blog post and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the inline comment for which versions should be returned | [required] |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of versions per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**VersionSortOrder**](VersionSortOrder.md)> | Used to sort the result by a particular field. |  |

### Return type

[**models::MultiEntityResultVersion4**](MultiEntityResult_Version__4.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_version_details

> models::DetailedVersion get_page_version_details(page_id, version_number)
Get version details for page version

Retrieves version details for the specified page and version number.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the page.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_id** | **i64** | The ID of the page for which version details should be returned. | [required] |
**version_number** | **i64** | The version number of the page to be returned. | [required] |

### Return type

[**models::DetailedVersion**](DetailedVersion.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_versions

> models::MultiEntityResultVersion2 get_page_versions(id, body_format, cursor, limit, sort)
Get page versions

Returns the versions of specific page.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the page and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page to be queried for its versions. If you don't know the page ID, use Get pages and filter the results. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of versions per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**VersionSortOrder**](VersionSortOrder.md)> | Used to sort the result by a particular field. |  |

### Return type

[**models::MultiEntityResultVersion2**](MultiEntityResult_Version__2.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
