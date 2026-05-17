# \ChildrenApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_child_custom_content**](ChildrenApi.md#get_child_custom_content) | **GET** /custom-content/{id}/children | Get child custom content
[**get_child_pages**](ChildrenApi.md#get_child_pages) | **GET** /pages/{id}/children | Get child pages
[**get_database_direct_children**](ChildrenApi.md#get_database_direct_children) | **GET** /databases/{id}/direct-children | Get direct children of a database
[**get_folder_direct_children**](ChildrenApi.md#get_folder_direct_children) | **GET** /folders/{id}/direct-children | Get direct children of a folder
[**get_page_direct_children**](ChildrenApi.md#get_page_direct_children) | **GET** /pages/{id}/direct-children | Get direct children of a page
[**get_smart_link_direct_children**](ChildrenApi.md#get_smart_link_direct_children) | **GET** /embeds/{id}/direct-children | Get direct children of a Smart Link
[**get_whiteboard_direct_children**](ChildrenApi.md#get_whiteboard_direct_children) | **GET** /whiteboards/{id}/direct-children | Get direct children of a whiteboard



## get_child_custom_content

> models::MultiEntityResultChildCustomContent get_child_custom_content(id, cursor, limit, sort)
Get child custom content

Returns all child custom content for given custom content id. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Only custom content that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the parent custom content. If you don't know the custom content ID, use Get custom-content and filter the results. | [required] |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of pages per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**Vec<models::ChildCustomContentSortOrder>**](Models__ChildCustomContentSortOrder.md)> | Used to sort the result by a particular field. |  |

### Return type

[**models::MultiEntityResultChildCustomContent**](MultiEntityResult_ChildCustomContent_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_child_pages

> models::MultiEntityResultChildPage get_child_pages(id, cursor, limit, sort)
Get child pages

Returns all child pages for given page id. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Only pages that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the parent page. If you don't know the page ID, use Get pages and filter the results. | [required] |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of pages per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**Vec<models::ChildPageSortOrder>**](Models__ChildPageSortOrder.md)> | Used to sort the result by a particular field. |  |

### Return type

[**models::MultiEntityResultChildPage**](MultiEntityResult_ChildPage_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_database_direct_children

> models::MultiEntityResultChildrenResponse get_database_direct_children(id, cursor, limit, sort)
Get direct children of a database

Returns all children for given database id in the content tree. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  The following types of content will be returned: - Database - Embed - Folder - Page - Whiteboard  This endpoint returns minimal information about each child. To fetch more details, use a related endpoint based on the content type, such as:  - [Get database by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-database/#api-databases-id-get) - [Get embed by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-smart-link/#api-embeds-id-get) - [Get folder by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-folder/#api-folders-id-get) - [Get page by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/#api-pages-id-get) - [Get whiteboard by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-whiteboard/#api-whiteboards-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Only content that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the parent database. | [required] |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of items per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**Vec<models::ContentSortOrder>**](Models__ContentSortOrder.md)> | Used to sort the result by a particular field. |  |

### Return type

[**models::MultiEntityResultChildrenResponse**](MultiEntityResult_ChildrenResponse_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_folder_direct_children

> models::MultiEntityResultChildrenResponse get_folder_direct_children(id, cursor, limit, sort)
Get direct children of a folder

Returns all children for given folder id in the content tree. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  The following types of content will be returned: - Database - Embed - Folder - Page - Whiteboard  This endpoint returns minimal information about each child. To fetch more details, use a related endpoint based on the content type, such as:  - [Get database by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-database/#api-databases-id-get) - [Get embed by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-smart-link/#api-embeds-id-get) - [Get folder by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-folder/#api-folders-id-get) - [Get page by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/#api-pages-id-get) - [Get whiteboard by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-whiteboard/#api-whiteboards-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Only content that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the parent folder. | [required] |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of items per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**Vec<models::ContentSortOrder>**](Models__ContentSortOrder.md)> | Used to sort the result by a particular field. |  |

### Return type

[**models::MultiEntityResultChildrenResponse**](MultiEntityResult_ChildrenResponse_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_direct_children

> models::MultiEntityResultChildrenResponse get_page_direct_children(id, cursor, limit, sort)
Get direct children of a page

Returns all children for given page id in the content tree. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  The following types of content will be returned: - Database - Embed - Folder - Page - Whiteboard  This endpoint returns minimal information about each child. To fetch more details, use a related endpoint based on the content type, such as:  - [Get database by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-database/#api-databases-id-get) - [Get embed by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-smart-link/#api-embeds-id-get) - [Get folder by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-folder/#api-folders-id-get) - [Get page by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/#api-pages-id-get) - [Get whiteboard by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-whiteboard/#api-whiteboards-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Only content that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the parent page. | [required] |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of items per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**Vec<models::ContentSortOrder>**](Models__ContentSortOrder.md)> | Used to sort the result by a particular field. |  |

### Return type

[**models::MultiEntityResultChildrenResponse**](MultiEntityResult_ChildrenResponse_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_smart_link_direct_children

> models::MultiEntityResultChildrenResponse get_smart_link_direct_children(id, cursor, limit, sort)
Get direct children of a Smart Link

Returns all children for given smart link id in the content tree. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  The following types of content will be returned: - Database - Embed - Folder - Page - Whiteboard  This endpoint returns minimal information about each child. To fetch more details, use a related endpoint based on the content type, such as:  - [Get database by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-database/#api-databases-id-get) - [Get embed by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-smart-link/#api-embeds-id-get) - [Get folder by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-folder/#api-folders-id-get) - [Get page by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/#api-pages-id-get) - [Get whiteboard by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-whiteboard/#api-whiteboards-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Only content that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the parent smart link. | [required] |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of items per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**Vec<models::ContentSortOrder>**](Models__ContentSortOrder.md)> | Used to sort the result by a particular field. |  |

### Return type

[**models::MultiEntityResultChildrenResponse**](MultiEntityResult_ChildrenResponse_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_whiteboard_direct_children

> models::MultiEntityResultChildrenResponse get_whiteboard_direct_children(id, cursor, limit, sort)
Get direct children of a whiteboard

Returns all children for given whiteboard id in the content tree. The number of results is limited by the `limit` parameter and additional results (if available) will be available through the `next` URL present in the `Link` response header.  The following types of content will be returned: - Database - Embed - Folder - Page - Whiteboard  This endpoint returns minimal information about each child. To fetch more details, use a related endpoint based on the content type, such as:  - [Get database by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-database/#api-databases-id-get) - [Get embed by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-smart-link/#api-embeds-id-get) - [Get folder by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-folder/#api-folders-id-get) - [Get page by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/#api-pages-id-get) - [Get whiteboard by id](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-whiteboard/#api-whiteboards-id-get).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Only content that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the parent whiteboard. | [required] |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of items per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]
**sort** | Option<[**Vec<models::ContentSortOrder>**](Models__ContentSortOrder.md)> | Used to sort the result by a particular field. |  |

### Return type

[**models::MultiEntityResultChildrenResponse**](MultiEntityResult_ChildrenResponse_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
