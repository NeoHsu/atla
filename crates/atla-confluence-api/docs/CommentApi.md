# \CommentApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_footer_comment**](CommentApi.md#create_footer_comment) | **POST** /footer-comments | Create footer comment
[**create_inline_comment**](CommentApi.md#create_inline_comment) | **POST** /inline-comments | Create inline comment
[**delete_footer_comment**](CommentApi.md#delete_footer_comment) | **DELETE** /footer-comments/{comment_id} | Delete footer comment
[**delete_inline_comment**](CommentApi.md#delete_inline_comment) | **DELETE** /inline-comments/{comment_id} | Delete inline comment
[**get_attachment_comments**](CommentApi.md#get_attachment_comments) | **GET** /attachments/{id}/footer-comments | Get attachment comments
[**get_blog_post_footer_comments**](CommentApi.md#get_blog_post_footer_comments) | **GET** /blogposts/{id}/footer-comments | Get footer comments for blog post
[**get_blog_post_inline_comments**](CommentApi.md#get_blog_post_inline_comments) | **GET** /blogposts/{id}/inline-comments | Get inline comments for blog post
[**get_custom_content_comments**](CommentApi.md#get_custom_content_comments) | **GET** /custom-content/{id}/footer-comments | Get custom content comments
[**get_footer_comment_by_id**](CommentApi.md#get_footer_comment_by_id) | **GET** /footer-comments/{comment_id} | Get footer comment by id
[**get_footer_comment_children**](CommentApi.md#get_footer_comment_children) | **GET** /footer-comments/{id}/children | Get children footer comments
[**get_footer_comments**](CommentApi.md#get_footer_comments) | **GET** /footer-comments | Get footer comments
[**get_inline_comment_by_id**](CommentApi.md#get_inline_comment_by_id) | **GET** /inline-comments/{comment_id} | Get inline comment by id
[**get_inline_comment_children**](CommentApi.md#get_inline_comment_children) | **GET** /inline-comments/{id}/children | Get children inline comments
[**get_inline_comments**](CommentApi.md#get_inline_comments) | **GET** /inline-comments | Get inline comments
[**get_page_footer_comments**](CommentApi.md#get_page_footer_comments) | **GET** /pages/{id}/footer-comments | Get footer comments for page
[**get_page_inline_comments**](CommentApi.md#get_page_inline_comments) | **GET** /pages/{id}/inline-comments | Get inline comments for page
[**update_footer_comment**](CommentApi.md#update_footer_comment) | **PUT** /footer-comments/{comment_id} | Update footer comment
[**update_inline_comment**](CommentApi.md#update_inline_comment) | **PUT** /inline-comments/{comment_id} | Update inline comment



## create_footer_comment

> models::CreateFooterComment201Response create_footer_comment(create_footer_comment_model)
Create footer comment

Create a footer comment.  The footer comment can be made against several locations:  - at the top level (specifying pageId or blogPostId in the request body) - as a reply (specifying parentCommentId in the request body) - against an attachment (note: this is different than the comments added via the attachment properties page on the UI, which are referred to as version comments) - against a custom content  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page or blogpost and its corresponding space. Permission to create comments in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_footer_comment_model** | [**CreateFooterCommentModel**](CreateFooterCommentModel.md) | The footer comment to be created | [required] |

### Return type

[**models::CreateFooterComment201Response**](createFooterComment_201_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_inline_comment

> models::CreateInlineComment201Response create_inline_comment(create_inline_comment_model)
Create inline comment

Create an inline comment. This can be at the top level (specifying pageId or blogPostId in the request body) or as a reply (specifying parentCommentId in the request body). Note the inlineCommentProperties object in the request body is used to select the text the inline comment should be tied to. This is what determines the text  highlighting when viewing a page in Confluence.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page or blogpost and its corresponding space. Permission to create comments in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_inline_comment_model** | [**CreateInlineCommentModel**](CreateInlineCommentModel.md) | The inline comment to be created | [required] |

### Return type

[**models::CreateInlineComment201Response**](createInlineComment_201_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_footer_comment

> delete_footer_comment(comment_id)
Delete footer comment

Deletes a footer comment. This is a permanent deletion and cannot be reverted.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page or blogpost and its corresponding space. Permission to delete comments in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**comment_id** | **i64** | The ID of the comment to be retrieved. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_inline_comment

> delete_inline_comment(comment_id)
Delete inline comment

Deletes an inline comment. This is a permanent deletion and cannot be reverted.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page or blogpost and its corresponding space. Permission to delete comments in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**comment_id** | **i64** | The ID of the comment to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_attachment_comments

> models::MultiEntityResultAttachmentCommentModel get_attachment_comments(id, body_format, cursor, limit, sort, version)
Get attachment comments

Returns the comments of the specific attachment. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the attachment and its corresponding containers.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the attachment for which comments should be returned. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format type to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of comments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**CommentSortOrder**](CommentSortOrder.md)> | Used to sort the result by a particular field. |  |
**version** | Option<**i64**> | Version number of the attachment to retrieve comments for. If no version provided, retrieves comments for the latest version. |  |

### Return type

[**models::MultiEntityResultAttachmentCommentModel**](MultiEntityResult_AttachmentCommentModel_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_blog_post_footer_comments

> models::MultiEntityResultBlogPostCommentModel get_blog_post_footer_comments(id, body_format, status, sort, cursor, limit)
Get footer comments for blog post

Returns the root footer comments of specific blog post. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the blog post and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post for which footer comments should be returned. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format type to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**status** | Option<[**Vec<String>**](String.md)> | Filter the footer comment being retrieved by its status. |  |
**sort** | Option<[**CommentSortOrder**](CommentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of footer comments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultBlogPostCommentModel**](MultiEntityResult_BlogPostCommentModel_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_blog_post_inline_comments

> models::MultiEntityResultBlogPostInlineCommentModel get_blog_post_inline_comments(id, body_format, status, resolution_status, sort, cursor, limit)
Get inline comments for blog post

Returns the root inline comments of specific blog post. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the blog post and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post for which inline comments should be returned. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format type to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**status** | Option<[**Vec<String>**](String.md)> | Filter the inline comment being retrieved by its status. |  |
**resolution_status** | Option<[**Vec<String>**](String.md)> | Filter the inline comment being retrieved by its resolution status. |  |
**sort** | Option<[**CommentSortOrder**](CommentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of inline comments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultBlogPostInlineCommentModel**](MultiEntityResult_BlogPostInlineCommentModel_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_custom_content_comments

> models::MultiEntityResultCustomContentCommentModel get_custom_content_comments(id, body_format, cursor, limit, sort)
Get custom content comments

Returns the comments of the specific custom content. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the custom content and its corresponding containers.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the custom content for which comments should be returned. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format type to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of comments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**CommentSortOrder**](CommentSortOrder.md)> | Used to sort the result by a particular field. |  |

### Return type

[**models::MultiEntityResultCustomContentCommentModel**](MultiEntityResult_CustomContentCommentModel_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_footer_comment_by_id

> models::CreateFooterComment201Response get_footer_comment_by_id(comment_id, body_format, version, include_properties, include_operations, include_likes, include_versions, include_version)
Get footer comment by id

Retrieves a footer comment by id  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the container and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**comment_id** | **i64** | The ID of the comment to be retrieved. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentationSingle**](PrimaryBodyRepresentationSingle.md)> | The content format type to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**version** | Option<**i32**> | Allows you to retrieve a previously published version. Specify the previous version's number to retrieve its details. |  |
**include_properties** | Option<**bool**> | Includes content properties associated with this footer comment in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_operations** | Option<**bool**> | Includes operations associated with this footer comment in the response, as defined in the `Operation` object. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_likes** | Option<**bool**> | Includes likes associated with this footer comment in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_versions** | Option<**bool**> | Includes versions associated with this footer comment in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_version** | Option<**bool**> | Includes the current version associated with this footer comment in the response. By default this is included and can be omitted by setting the value to `false`. |  |[default to true]

### Return type

[**models::CreateFooterComment201Response**](createFooterComment_201_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_footer_comment_children

> models::MultiEntityResultChildrenCommentModel get_footer_comment_children(id, body_format, sort, cursor, limit)
Get children footer comments

Returns the children footer comments of specific comment. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the parent comment for which footer comment children should be returned. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format type to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**sort** | Option<[**CommentSortOrder**](CommentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of footer comments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultChildrenCommentModel**](MultiEntityResult_ChildrenCommentModel_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_footer_comments

> models::MultiEntityResultFooterCommentModel get_footer_comments(body_format, sort, cursor, limit)
Get footer comments

Returns all footer comments. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the container and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format type to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**sort** | Option<[**CommentSortOrder**](CommentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of footer comments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultFooterCommentModel**](MultiEntityResult_FooterCommentModel_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_inline_comment_by_id

> models::CreateInlineComment201Response get_inline_comment_by_id(comment_id, body_format, version, include_properties, include_operations, include_likes, include_versions, include_version)
Get inline comment by id

Retrieves an inline comment by id  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page or blogpost and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**comment_id** | **i64** | The ID of the comment to be retrieved. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentationSingle**](PrimaryBodyRepresentationSingle.md)> | The content format type to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**version** | Option<**i32**> | Allows you to retrieve a previously published version. Specify the previous version's number to retrieve its details. |  |
**include_properties** | Option<**bool**> | Includes content properties associated with this inline comment in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_operations** | Option<**bool**> | Includes operations associated with this inline comment in the response, as defined in the `Operation` object. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_likes** | Option<**bool**> | Includes likes associated with this inline comment in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_versions** | Option<**bool**> | Includes versions associated with this inline comment in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_version** | Option<**bool**> | Includes the current version associated with this inline comment in the response. By default this is included and can be omitted by setting the value to `false`. |  |[default to true]

### Return type

[**models::CreateInlineComment201Response**](createInlineComment_201_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_inline_comment_children

> models::MultiEntityResultInlineCommentChildrenModel get_inline_comment_children(id, body_format, sort, cursor, limit)
Get children inline comments

Returns the children inline comments of specific comment. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the parent comment for which inline comment children should be returned. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format type to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**sort** | Option<[**CommentSortOrder**](CommentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of footer comments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultInlineCommentChildrenModel**](MultiEntityResult_InlineCommentChildrenModel_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_inline_comments

> models::MultiEntityResultInlineCommentModel get_inline_comments(body_format, sort, cursor, limit)
Get inline comments

Returns all inline comments. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format type to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**sort** | Option<[**CommentSortOrder**](CommentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of footer comments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultInlineCommentModel**](MultiEntityResult_InlineCommentModel_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_footer_comments

> models::MultiEntityResultPageCommentModel get_page_footer_comments(id, body_format, status, sort, cursor, limit)
Get footer comments for page

Returns the root footer comments of specific page. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page for which footer comments should be returned. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format type to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**status** | Option<[**Vec<String>**](String.md)> | Filter the footer comment being retrieved by its status. |  |
**sort** | Option<[**CommentSortOrder**](CommentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of footer comments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultPageCommentModel**](MultiEntityResult_PageCommentModel_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_inline_comments

> models::MultiEntityResultPageInlineCommentModel get_page_inline_comments(id, body_format, status, resolution_status, sort, cursor, limit)
Get inline comments for page

Returns the root inline comments of specific page. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page for which inline comments should be returned. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format type to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**status** | Option<[**Vec<String>**](String.md)> | Filter the inline comment being retrieved by its status. |  |
**resolution_status** | Option<[**Vec<String>**](String.md)> | Filter the inline comment being retrieved by its resolution status. |  |
**sort** | Option<[**CommentSortOrder**](CommentSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of inline comments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultPageInlineCommentModel**](MultiEntityResult_PageInlineCommentModel_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_footer_comment

> models::FooterCommentModel update_footer_comment(comment_id, update_footer_comment_request)
Update footer comment

Update a footer comment. This can be used to update the body text of a comment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page or blogpost and its corresponding space. Permission to create comments in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**comment_id** | **i64** | The ID of the comment to be retrieved. | [required] |
**update_footer_comment_request** | [**UpdateFooterCommentRequest**](UpdateFooterCommentRequest.md) | The footer comment to be created | [required] |

### Return type

[**models::FooterCommentModel**](FooterCommentModel.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_inline_comment

> models::CreateInlineComment201Response update_inline_comment(comment_id, update_inline_comment_model)
Update inline comment

Update an inline comment. This can be used to update the body text of a comment and/or to resolve the comment  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page or blogpost and its corresponding space. Permission to create comments in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**comment_id** | **i64** | The ID of the comment to be retrieved. | [required] |
**update_inline_comment_model** | [**UpdateInlineCommentModel**](UpdateInlineCommentModel.md) | The inline comment to be updated | [required] |

### Return type

[**models::CreateInlineComment201Response**](createInlineComment_201_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
