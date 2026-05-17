# \LikeApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_blog_post_like_count**](LikeApi.md#get_blog_post_like_count) | **GET** /blogposts/{id}/likes/count | Get like count for blog post
[**get_blog_post_like_users**](LikeApi.md#get_blog_post_like_users) | **GET** /blogposts/{id}/likes/users | Get account IDs of likes for blog post
[**get_footer_like_count**](LikeApi.md#get_footer_like_count) | **GET** /footer-comments/{id}/likes/count | Get like count for footer comment
[**get_footer_like_users**](LikeApi.md#get_footer_like_users) | **GET** /footer-comments/{id}/likes/users | Get account IDs of likes for footer comment
[**get_inline_like_count**](LikeApi.md#get_inline_like_count) | **GET** /inline-comments/{id}/likes/count | Get like count for inline comment
[**get_inline_like_users**](LikeApi.md#get_inline_like_users) | **GET** /inline-comments/{id}/likes/users | Get account IDs of likes for inline comment
[**get_page_like_count**](LikeApi.md#get_page_like_count) | **GET** /pages/{id}/likes/count | Get like count for page
[**get_page_like_users**](LikeApi.md#get_page_like_users) | **GET** /pages/{id}/likes/users | Get account IDs of likes for page



## get_blog_post_like_count

> models::Integer get_blog_post_like_count(id)
Get like count for blog post

Returns the count of likes of specific blog post.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the blog post and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post for which like count should be returned. | [required] |

### Return type

[**models::Integer**](Integer.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_blog_post_like_users

> models::MultiEntityResultString get_blog_post_like_users(id, cursor, limit)
Get account IDs of likes for blog post

Returns the account IDs of likes of specific blog post.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the blog post and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post for which account IDs should be returned. | [required] |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of account IDs per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultString**](MultiEntityResult_String_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_footer_like_count

> models::Integer get_footer_like_count(id)
Get like count for footer comment

Returns the count of likes of specific footer comment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page/blogpost and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the footer comment for which like count should be returned. | [required] |

### Return type

[**models::Integer**](Integer.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_footer_like_users

> models::MultiEntityResultString get_footer_like_users(id, cursor, limit)
Get account IDs of likes for footer comment

Returns the account IDs of likes of specific footer comment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page/blogpost and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the footer comment for which like count should be returned. | [required] |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of account IDs per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultString**](MultiEntityResult_String_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_inline_like_count

> models::Integer get_inline_like_count(id)
Get like count for inline comment

Returns the count of likes of specific inline comment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page/blogpost and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the inline comment for which like count should be returned. | [required] |

### Return type

[**models::Integer**](Integer.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_inline_like_users

> models::MultiEntityResultString get_inline_like_users(id, cursor, limit)
Get account IDs of likes for inline comment

Returns the account IDs of likes of specific inline comment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page/blogpost and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the inline comment for which like count should be returned. | [required] |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of account IDs per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultString**](MultiEntityResult_String_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_like_count

> models::Integer get_page_like_count(id)
Get like count for page

Returns the count of likes of specific page.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page for which like count should be returned. | [required] |

### Return type

[**models::Integer**](Integer.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_like_users

> models::MultiEntityResultString get_page_like_users(id, cursor, limit)
Get account IDs of likes for page

Returns the account IDs of likes of specific page.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page for which like count should be returned. | [required] |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of account IDs per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultString**](MultiEntityResult_String_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
