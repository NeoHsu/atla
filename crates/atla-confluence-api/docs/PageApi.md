# \PageApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_page**](PageApi.md#create_page) | **POST** /pages | Create page
[**delete_page**](PageApi.md#delete_page) | **DELETE** /pages/{id} | Delete page
[**get_label_pages**](PageApi.md#get_label_pages) | **GET** /labels/{id}/pages | Get pages for label
[**get_page_by_id**](PageApi.md#get_page_by_id) | **GET** /pages/{id} | Get page by id
[**get_pages**](PageApi.md#get_pages) | **GET** /pages | Get pages
[**get_pages_in_space**](PageApi.md#get_pages_in_space) | **GET** /spaces/{id}/pages | Get pages in space
[**update_page**](PageApi.md#update_page) | **PUT** /pages/{id} | Update page
[**update_page_title**](PageApi.md#update_page_title) | **PUT** /pages/{id}/title | Update page title



## create_page

> models::CreatePage200Response create_page(create_page_request, embedded, private, root_level)
Create page

Creates a page in the space.  Pages are created as published by default unless specified as a draft in the status field. If creating a published page, the title must be specified.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the corresponding space. Permission to create a page in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_page_request** | [**CreatePageRequest**](CreatePageRequest.md) |  | [required] |
**embedded** | Option<**bool**> | Tag the content as embedded and content will be created in NCS. |  |[default to false]
**private** | Option<**bool**> | The page will be private. Only the user who creates this page will have permission to view and edit one. |  |[default to false]
**root_level** | Option<**bool**> | The page will be created at the root level of the space (outside the space homepage tree). If true, then a  value may not be supplied for the `parentId` body parameter. |  |[default to false]

### Return type

[**models::CreatePage200Response**](createPage_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_page

> delete_page(id, purge, draft)
Delete page

Delete a page by id.  By default this will delete pages that are non-drafts. To delete a page that is a draft, the endpoint must be called on a  draft with the following param `draft=true`. Discarded drafts are not sent to the trash and are permanently deleted.  Deleting a page moves the page to the trash, where it can be restored later. To permanently delete a page (or \"purge\" it), the endpoint must be called on a **trashed** page with the following param `purge=true`.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the page and its corresponding space. Permission to delete pages in the space. Permission to administer the space (if attempting to purge).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page to be deleted. | [required] |
**purge** | Option<**bool**> | If attempting to purge the page. |  |[default to false]
**draft** | Option<**bool**> | If attempting to delete a page that is a draft. |  |[default to false]

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_label_pages

> models::MultiEntityResultPage get_label_pages(id, space_id, body_format, sort, cursor, limit)
Get pages for label

Returns the pages of specified label. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the content of the page and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the label for which pages should be returned. | [required] |
**space_id** | Option<[**Vec<i64>**](I64.md)> | Filter the results based on space ids. Multiple space ids can be specified as a comma-separated list. |  |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**sort** | Option<[**PageSortOrder**](PageSortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of pages per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultPage**](MultiEntityResult_Page_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_by_id

> models::CreatePage200Response get_page_by_id(id, body_format, get_draft, status, version, include_labels, include_properties, include_operations, include_likes, include_versions, include_version, include_favorited_by_current_user_status, include_webresources, include_collaborators, include_direct_children)
Get page by id

Returns a specific page.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the page and its corresponding space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page to be returned. If you don't know the page ID, use Get pages and filter the results. | [required] |
**body_format** | Option<[**PrimaryBodyRepresentationSingle**](PrimaryBodyRepresentationSingle.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**get_draft** | Option<**bool**> | Retrieve the draft version of this page. |  |[default to false]
**status** | Option<[**Vec<String>**](String.md)> | Filter the page being retrieved by its status. |  |
**version** | Option<**i32**> | Allows you to retrieve a previously published version. Specify the previous version's number to retrieve its details. |  |
**include_labels** | Option<**bool**> | Includes labels associated with this page in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_properties** | Option<**bool**> | Includes content properties associated with this page in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_operations** | Option<**bool**> | Includes operations associated with this page in the response, as defined in the `Operation` object. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_likes** | Option<**bool**> | Includes likes associated with this page in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_versions** | Option<**bool**> | Includes versions associated with this page in the response. The number of results will be limited to 50 and sorted in the default sort order.  A `meta` and `_links` property will be present to indicate if more results are available and a link to retrieve the rest of the results. |  |[default to false]
**include_version** | Option<**bool**> | Includes the current version associated with this page in the response. By default this is included and can be omitted by setting the value to `false`. |  |[default to true]
**include_favorited_by_current_user_status** | Option<**bool**> | Includes whether this page has been favorited by the current user. |  |[default to false]
**include_webresources** | Option<**bool**> | Includes web resources that can be used to render page content on a client. |  |[default to false]
**include_collaborators** | Option<**bool**> | Includes collaborators on the page. |  |[default to false]
**include_direct_children** | Option<**bool**> | Includes direct children of the page, as defined in the `ChildrenResponse` object. |  |[default to false]

### Return type

[**models::CreatePage200Response**](createPage_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_pages

> models::MultiEntityResultPage get_pages(id, space_id, sort, status, title, body_format, subtype, cursor, limit)
Get pages

Returns all pages. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Only pages that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | Option<[**Vec<i64>**](I64.md)> | Filter the results based on page ids. Multiple page ids can be specified as a comma-separated list. |  |
**space_id** | Option<[**Vec<i64>**](I64.md)> | Filter the results based on space ids. Multiple space ids can be specified as a comma-separated list. |  |
**sort** | Option<[**PageSortOrder**](PageSortOrder.md)> | Used to sort the result by a particular field. |  |
**status** | Option<[**Vec<String>**](String.md)> | Filter the results to pages based on their status. By default, `current` and `archived` are used. |  |
**title** | Option<**String**> | Filter the results to pages based on their title. |  |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**subtype** | Option<**String**> | Filter the results to pages based on their subtype. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of pages per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultPage**](MultiEntityResult_Page_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_pages_in_space

> models::MultiEntityResultPage get_pages_in_space(id, depth, sort, status, title, body_format, cursor, limit)
Get pages in space

Returns all pages in a space. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission) and 'View' permission for the space. Only pages that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the space for which pages should be returned. | [required] |
**depth** | Option<**String**> | Filter the results to pages at the root level of the space or to all pages in the space. |  |[default to all]
**sort** | Option<[**PageSortOrder**](PageSortOrder.md)> | Used to sort the result by a particular field. |  |
**status** | Option<[**Vec<String>**](String.md)> | Filter the results to pages based on their status. By default, `current` and `archived` are used. |  |
**title** | Option<**String**> | Filter the results to pages based on their title. |  |
**body_format** | Option<[**PrimaryBodyRepresentation**](PrimaryBodyRepresentation.md)> | The content format types to be returned in the `body` field of the response. If available, the representation will be available under a response field of the same name under the `body` field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of pages per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultPage**](MultiEntityResult_Page_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_page

> models::CreatePage200Response update_page(id, update_page_request)
Update page

Update a page by id.  When the \"current\" version is updated, the provided body content is considered as the latest version. This latest body content will be attempted to be merged into the draft version through a content reconciliation algorithm. If two versions are significantly diverged,  the latest provided content may entirely override what was previously in the draft.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the page and its corresponding space. Permission to update pages in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page to be updated. If you don't know the page ID, use Get Pages and filter the results. | [required] |
**update_page_request** | [**UpdatePageRequest**](UpdatePageRequest.md) |  | [required] |

### Return type

[**models::CreatePage200Response**](createPage_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_page_title

> models::CreatePage200Response update_page_title(id, update_page_title_request)
Update page title

Updates the title of a specified page.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the page and its corresponding space. Permission to update pages in the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page to be updated. If you don't know the page ID, use Get Pages and filter the results | [required] |
**update_page_title_request** | [**UpdatePageTitleRequest**](UpdatePageTitleRequest.md) |  | [required] |

### Return type

[**models::CreatePage200Response**](createPage_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
