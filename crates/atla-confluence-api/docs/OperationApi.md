# \OperationApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_attachment_operations**](OperationApi.md#get_attachment_operations) | **GET** /attachments/{id}/operations | Get permitted operations for attachment
[**get_blog_post_operations**](OperationApi.md#get_blog_post_operations) | **GET** /blogposts/{id}/operations | Get permitted operations for blog post
[**get_custom_content_operations**](OperationApi.md#get_custom_content_operations) | **GET** /custom-content/{id}/operations | Get permitted operations for custom content
[**get_database_operations**](OperationApi.md#get_database_operations) | **GET** /databases/{id}/operations | Get permitted operations for a database
[**get_folder_operations**](OperationApi.md#get_folder_operations) | **GET** /folders/{id}/operations | Get permitted operations for a folder
[**get_footer_comment_operations**](OperationApi.md#get_footer_comment_operations) | **GET** /footer-comments/{id}/operations | Get permitted operations for footer comment
[**get_inline_comment_operations**](OperationApi.md#get_inline_comment_operations) | **GET** /inline-comments/{id}/operations | Get permitted operations for inline comment
[**get_page_operations**](OperationApi.md#get_page_operations) | **GET** /pages/{id}/operations | Get permitted operations for page
[**get_smart_link_operations**](OperationApi.md#get_smart_link_operations) | **GET** /embeds/{id}/operations | Get permitted operations for a Smart Link in the content tree
[**get_space_operations**](OperationApi.md#get_space_operations) | **GET** /spaces/{id}/operations | Get permitted operations for space
[**get_whiteboard_operations**](OperationApi.md#get_whiteboard_operations) | **GET** /whiteboards/{id}/operations | Get permitted operations for a whiteboard



## get_attachment_operations

> models::PermittedOperationsResponse get_attachment_operations(id)
Get permitted operations for attachment

Returns the permitted operations on specific attachment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the parent content of the attachment and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the attachment for which operations should be returned. | [required] |

### Return type

[**models::PermittedOperationsResponse**](PermittedOperationsResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_blog_post_operations

> models::PermittedOperationsResponse get_blog_post_operations(id)
Get permitted operations for blog post

Returns the permitted operations on specific blog post.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the parent content of the blog post and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post for which operations should be returned. | [required] |

### Return type

[**models::PermittedOperationsResponse**](PermittedOperationsResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_custom_content_operations

> models::PermittedOperationsResponse get_custom_content_operations(id)
Get permitted operations for custom content

Returns the permitted operations on specific custom content.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the parent content of the custom content and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the custom content for which operations should be returned. | [required] |

### Return type

[**models::PermittedOperationsResponse**](PermittedOperationsResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_database_operations

> models::PermittedOperationsResponse get_database_operations(id)
Get permitted operations for a database

Returns the permitted operations on specific database.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the database and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the database for which operations should be returned. | [required] |

### Return type

[**models::PermittedOperationsResponse**](PermittedOperationsResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_folder_operations

> models::PermittedOperationsResponse get_folder_operations(id)
Get permitted operations for a folder

Returns the permitted operations on specific folder.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the folder and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the folder for which operations should be returned. | [required] |

### Return type

[**models::PermittedOperationsResponse**](PermittedOperationsResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_footer_comment_operations

> models::PermittedOperationsResponse get_footer_comment_operations(id)
Get permitted operations for footer comment

Returns the permitted operations on specific footer comment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the parent content of the footer comment and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the footer comment for which operations should be returned. | [required] |

### Return type

[**models::PermittedOperationsResponse**](PermittedOperationsResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_inline_comment_operations

> models::PermittedOperationsResponse get_inline_comment_operations(id)
Get permitted operations for inline comment

Returns the permitted operations on specific inline comment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the parent content of the inline comment and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the inline comment for which operations should be returned. | [required] |

### Return type

[**models::PermittedOperationsResponse**](PermittedOperationsResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_operations

> models::PermittedOperationsResponse get_page_operations(id)
Get permitted operations for page

Returns the permitted operations on specific page.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the parent content of the page and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page for which operations should be returned. | [required] |

### Return type

[**models::PermittedOperationsResponse**](PermittedOperationsResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_smart_link_operations

> models::PermittedOperationsResponse get_smart_link_operations(id)
Get permitted operations for a Smart Link in the content tree

Returns the permitted operations on specific Smart Link in the content tree.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the Smart Link in the content tree and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the Smart Link in the content tree for which operations should be returned. | [required] |

### Return type

[**models::PermittedOperationsResponse**](PermittedOperationsResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_space_operations

> models::PermittedOperationsResponse get_space_operations(id)
Get permitted operations for space

Returns the permitted operations on specific space.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the space for which operations should be returned. | [required] |

### Return type

[**models::PermittedOperationsResponse**](PermittedOperationsResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_whiteboard_operations

> models::PermittedOperationsResponse get_whiteboard_operations(id)
Get permitted operations for a whiteboard

Returns the permitted operations on specific whiteboard.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the whiteboard and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the whiteboard for which operations should be returned. | [required] |

### Return type

[**models::PermittedOperationsResponse**](PermittedOperationsResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
