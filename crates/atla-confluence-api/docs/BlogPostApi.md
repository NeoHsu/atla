# \BlogPostApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_blog_post**](BlogPostApi.md#create_blog_post) | **POST** /blogposts | Create blog post
[**delete_blog_post**](BlogPostApi.md#delete_blog_post) | **DELETE** /blogposts/{id} | Delete blog post
[**get_blog_post_by_id**](BlogPostApi.md#get_blog_post_by_id) | **GET** /blogposts/{id} | Get blog post by id
[**get_blog_posts**](BlogPostApi.md#get_blog_posts) | **GET** /blogposts | Get blog posts
[**get_blog_posts_in_space**](BlogPostApi.md#get_blog_posts_in_space) | **GET** /spaces/{id}/blogposts | Get blog posts in space
[**get_label_blog_posts**](BlogPostApi.md#get_label_blog_posts) | **GET** /labels/{id}/blogposts | Get blog posts for label
[**update_blog_post**](BlogPostApi.md#update_blog_post) | **PUT** /blogposts/{id} | Update blog post



## create_blog_post

> models::CreateBlogPost200Response create_blog_post(create_blog_post_request, private)
Create blog post

Creates a new blog post in the space specified by the spaceId.  By default this will create the blog post as a non-draft, unless the status is specified as draft. If creating a non-draft, the title must not be empty.  Currently only supports the storage representation specified in the body.representation enums below

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_blog_post_request** | [**CreateBlogPostRequest**](CreateBlogPostRequest.md) |  | [required] |
**private** | Option<**bool**> | The blog post will be private. Only the user who creates this blog post will have permission to view and edit one. |  |[default to false]

### Return type

[**models::CreateBlogPost200Response**](createBlogPost_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_blog_post

> delete_blog_post(id, purge, draft)
Delete blog post

Delete a blog post by id.  By default this will delete blog posts that are non-drafts. To delete a blog post that is a draft, the endpoint must be called on a  draft with the following param `draft=true`. Discarded drafts are not sent to the trash and are permanently deleted.  Deleting a blog post that is not a draft moves the blog post to the trash, where it can be restored later. To permanently delete a blog post (or \"purge\" it), the endpoint must be called on a **trashed** blog post with the following param `purge=true`.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the blog post and its corresponding space. Permission to delete blog posts in the space. Permission to administer the space (if attempting to purge).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post to be deleted. | [required] |
**purge** | Option<**bool**> | If attempting to purge the blog post. |  |[default to false]
**draft** | Option<**bool**> | If attempting to delete a blog post that is a draft. |  |[default to false]

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_blog_post_by_id

> models::CreateBlogPost200Response get_blog_post_by_id(id, body_format, get_draft, status, version, include_labels, include_properties, include_operations, include_likes, include_versions, include_version, include_favorited_by_current_user_status, include_webresources, include_collaborators)
Get blog post by id

Returns a specific blog post.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the blog post and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post to be returned. If you don't know the blog post ID, use Get blog posts and filter the results. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentationSingle**](PrimaryBodyRepresentationSingle.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**get_draft** | Option<**bool**> | Retrieve the draft version of this blog post. |  |[default to false]
**status** | Option<[**Vec<String>**](String.md)> | Filter the blog post being retrieved by its status. |  |
**version** | Option<**i32**> | Allows you to retrieve a previously published version. Specify the previous version's number to retrieve its details. |  |
**include_labels** | Option<**bool**> | Includes labels associated with this blog post in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_properties** | Option<**bool**> | Includes content properties associated with this blog post in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_operations** | Option<**bool**> | Includes operations associated with this blog post in the response, as defined in the `Operation` object. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_likes** | Option<**bool**> | Includes likes associated with this blog post in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_versions** | Option<**bool**> | Includes versions associated with this blog post in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_version** | Option<**bool**> | Includes the current version associated with this blog post in the response. By default this is included and can be omitted by setting the value to `false`. |  |[default to true]
**include_favorited_by_current_user_status** | Option<**bool**> | Includes whether this blog post has been favorited by the current user. |  |[default to false]
**include_webresources** | Option<**bool**> | Includes web resources that can be used to render blog post content on a client. |  |[default to false]
**include_collaborators** | Option<**bool**> | Includes collaborators on the blog post. |  |[default to false]

### Return type

[**models::CreateBlogPost200Response**](createBlogPost_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_blog_posts

> models::MultiEntityResultBlogPost get_blog_posts(id, space_id, sort, status, title, body_format, cursor, limit)
Get blog posts

Returns all blog posts. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Only blog posts that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | Option<[**Vec<i64>**](I64.md)> | Filter the results based on blog post ids. Multiple blog post ids can be specified as a comma-separated list. |  |
**space_id** | Option<[**Vec<i64>**](I64.md)> | Filter the results based on space ids. Multiple space ids can be specified as a comma-separated list. |  |
**sort** | Option<[**BlogPostSortOrder**](BlogPostSortOrder.md)> | Used to sort the result by a particular field. |  |
**status** | Option<[**Vec<String>**](String.md)> | Filter the results to blog posts based on their status. By default, `current` is used. |  |
**title** | Option<**String**> | Filter the results to blog posts based on their title. |  |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of blog posts per result to return. If more results exist, use the `Link` response header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultBlogPost**](MultiEntityResult_BlogPost_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_blog_posts_in_space

> models::MultiEntityResultBlogPost get_blog_posts_in_space(id, sort, status, title, body_format, cursor, limit)
Get blog posts in space

Returns all blog posts in a space. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission) and view the space. Only blog posts that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the space for which blog posts should be returned. | [required] |
**sort** | Option<[**BlogPostSortOrder**](BlogPostSortOrder.md)> | Used to sort the result by a particular field. |  |
**status** | Option<[**Vec<String>**](String.md)> | Filter the results to blog posts based on their status. By default, `current` is used. |  |
**title** | Option<**String**> | Filter the results to blog posts based on their title. |  |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of blog posts per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultBlogPost**](MultiEntityResult_BlogPost_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_label_blog_posts

> models::MultiEntityResultBlogPost get_label_blog_posts(id, space_id, body_format, sort, cursor, limit)
Get blog posts for label

Returns the blogposts of specified label. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the label for which blog posts should be returned. | [required] |
**space_id** | Option<[**Vec<i64>**](I64.md)> | Filter the results based on space ids. Multiple space ids can be specified as a comma-separated list. |  |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**sort** | Option<[**BlogPostSortOrder**](BlogPostSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of blog posts per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultBlogPost**](MultiEntityResult_BlogPost_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_blog_post

> models::CreateBlogPost200Response update_blog_post(id, update_blog_post_request)
Update blog post

Update a blog post by id.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the blog post and its corresponding space. Permission to update blog posts in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post to be updated. If you don't know the blog post ID, use Get Blog Posts and filter the results. | [required] |
**update_blog_post_request** | [**UpdateBlogPostRequest**](UpdateBlogPostRequest.md) |  | [required] |

### Return type

[**models::CreateBlogPost200Response**](createBlogPost_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
