# \ClassificationLevelApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_space_default_classification_level**](ClassificationLevelApi.md#delete_space_default_classification_level) | **DELETE** /spaces/{id}/classification-level/default | Delete space default classification level
[**get_blog_post_classification_level**](ClassificationLevelApi.md#get_blog_post_classification_level) | **GET** /blogposts/{id}/classification-level | Get blog post classification level
[**get_classification_levels**](ClassificationLevelApi.md#get_classification_levels) | **GET** /classification-levels | Get list of classification levels
[**get_database_classification_level**](ClassificationLevelApi.md#get_database_classification_level) | **GET** /databases/{id}/classification-level | Get database classification level
[**get_page_classification_level**](ClassificationLevelApi.md#get_page_classification_level) | **GET** /pages/{id}/classification-level | Get page classification level
[**get_space_default_classification_level**](ClassificationLevelApi.md#get_space_default_classification_level) | **GET** /spaces/{id}/classification-level/default | Get space default classification level
[**get_whiteboard_classification_level**](ClassificationLevelApi.md#get_whiteboard_classification_level) | **GET** /whiteboards/{id}/classification-level | Get whiteboard classification level
[**post_blog_post_classification_level**](ClassificationLevelApi.md#post_blog_post_classification_level) | **POST** /blogposts/{id}/classification-level/reset | Reset blog post classification level
[**post_database_classification_level**](ClassificationLevelApi.md#post_database_classification_level) | **POST** /databases/{id}/classification-level/reset | Reset database classification level
[**post_page_classification_level**](ClassificationLevelApi.md#post_page_classification_level) | **POST** /pages/{id}/classification-level/reset | Reset page classification level
[**post_whiteboard_classification_level**](ClassificationLevelApi.md#post_whiteboard_classification_level) | **POST** /whiteboards/{id}/classification-level/reset | Reset whiteboard classification level
[**put_blog_post_classification_level**](ClassificationLevelApi.md#put_blog_post_classification_level) | **PUT** /blogposts/{id}/classification-level | Update blog post classification level
[**put_database_classification_level**](ClassificationLevelApi.md#put_database_classification_level) | **PUT** /databases/{id}/classification-level | Update database classification level
[**put_page_classification_level**](ClassificationLevelApi.md#put_page_classification_level) | **PUT** /pages/{id}/classification-level | Update page classification level
[**put_space_default_classification_level**](ClassificationLevelApi.md#put_space_default_classification_level) | **PUT** /spaces/{id}/classification-level/default | Update space default classification level
[**put_whiteboard_classification_level**](ClassificationLevelApi.md#put_whiteboard_classification_level) | **PUT** /whiteboards/{id}/classification-level | Update whiteboard classification level



## delete_space_default_classification_level

> delete_space_default_classification_level(id)
Delete space default classification level

Returns the [default classification level](https://support.atlassian.com/security-and-access-policies/docs/what-is-a-default-classification-level/)  for a specific space.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and 'Admin' permission for the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the space for which default classification level should be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_blog_post_classification_level

> models::ClassificationLevel get_blog_post_classification_level(id, status)
Get blog post classification level

Returns the [classification level](https://developer.atlassian.com/cloud/admin/dlp/rest/intro/#Classification%20level) for a specific blog post.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and permission to view the blog post. 'Permission to edit the blog post is required if trying to view classification level for a draft.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post for which classification level should be returned. | [required] |
**status** | Option<**String**> | Status of blog post from which classification level will fetched. |  |[default to current]

### Return type

[**models::ClassificationLevel**](ClassificationLevel.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_classification_levels

> Vec<models::ClassificationLevel> get_classification_levels()
Get list of classification levels

Returns a list of [classification levels](https://developer.atlassian.com/cloud/admin/dlp/rest/intro/#Classification%20level)  available.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission).

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::ClassificationLevel>**](ClassificationLevel.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_database_classification_level

> models::ClassificationLevel get_database_classification_level(id)
Get database classification level

Returns the [classification level](https://developer.atlassian.com/cloud/admin/dlp/rest/intro/#Classification%20level) for a specific database.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and permission to view the database.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the database for which classification level should be returned. | [required] |

### Return type

[**models::ClassificationLevel**](ClassificationLevel.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_classification_level

> models::ClassificationLevel get_page_classification_level(id, status)
Get page classification level

Returns the [classification level](https://developer.atlassian.com/cloud/admin/dlp/rest/intro/#Classification%20level) for a specific page.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and permission to view the page. 'Permission to edit the page is required if trying to view classification level for a draft.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page for which classification level should be returned. | [required] |
**status** | Option<**String**> | Status of page from which classification level will fetched. |  |[default to current]

### Return type

[**models::ClassificationLevel**](ClassificationLevel.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_space_default_classification_level

> models::ClassificationLevel get_space_default_classification_level(id)
Get space default classification level

Returns the [default classification level](https://support.atlassian.com/security-and-access-policies/docs/what-is-a-default-classification-level/)  for a specific space.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and permission to view the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the space for which default classification level should be returned. | [required] |

### Return type

[**models::ClassificationLevel**](ClassificationLevel.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_whiteboard_classification_level

> models::ClassificationLevel get_whiteboard_classification_level(id)
Get whiteboard classification level

Returns the [classification level](https://developer.atlassian.com/cloud/admin/dlp/rest/intro/#Classification%20level) for a specific whiteboard.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and permission to view the whiteboard.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the whiteboard for which classification level should be returned. | [required] |

### Return type

[**models::ClassificationLevel**](ClassificationLevel.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_blog_post_classification_level

> post_blog_post_classification_level(id, post_page_classification_level_request)
Reset blog post classification level

Resets the [classification level](https://developer.atlassian.com/cloud/admin/dlp/rest/intro/#Classification%20level) for a specific blog post for the space   [default classification level](https://support.atlassian.com/security-and-access-policies/docs/what-is-a-default-classification-level/).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and permission to view the blog post.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post for which classification level should be updated. | [required] |
**post_page_classification_level_request** | [**PostPageClassificationLevelRequest**](PostPageClassificationLevelRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_database_classification_level

> post_database_classification_level(id, post_whiteboard_classification_level_request)
Reset database classification level

Resets the [classification level](https://developer.atlassian.com/cloud/admin/dlp/rest/intro/#Classification%20level) for a specific database for the space  [default classification level](https://support.atlassian.com/security-and-access-policies/docs/what-is-a-default-classification-level/).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and permission to view the database.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the database for which classification level should be updated. | [required] |
**post_whiteboard_classification_level_request** | [**PostWhiteboardClassificationLevelRequest**](PostWhiteboardClassificationLevelRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_page_classification_level

> post_page_classification_level(id, post_page_classification_level_request)
Reset page classification level

Resets the [classification level](https://developer.atlassian.com/cloud/admin/dlp/rest/intro/#Classification%20level) for a specific page for the space  [default classification level](https://support.atlassian.com/security-and-access-policies/docs/what-is-a-default-classification-level/).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and permission to view the page.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page for which classification level should be updated. | [required] |
**post_page_classification_level_request** | [**PostPageClassificationLevelRequest**](PostPageClassificationLevelRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_whiteboard_classification_level

> post_whiteboard_classification_level(id, post_whiteboard_classification_level_request)
Reset whiteboard classification level

Resets the [classification level](https://developer.atlassian.com/cloud/admin/dlp/rest/intro/#Classification%20level) for a specific whiteboard for the space  [default classification level](https://support.atlassian.com/security-and-access-policies/docs/what-is-a-default-classification-level/).  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and permission to view the whiteboard.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the whiteboard for which classification level should be updated. | [required] |
**post_whiteboard_classification_level_request** | [**PostWhiteboardClassificationLevelRequest**](PostWhiteboardClassificationLevelRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_blog_post_classification_level

> put_blog_post_classification_level(id, put_page_classification_level_request)
Update blog post classification level

Updates the [classification level](https://developer.atlassian.com/cloud/admin/dlp/rest/intro/#Classification%20level) for a specific blog post.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and permission to edit the blog post.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post for which classification level should be updated. | [required] |
**put_page_classification_level_request** | [**PutPageClassificationLevelRequest**](PutPageClassificationLevelRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_database_classification_level

> put_database_classification_level(id, put_whiteboard_classification_level_request)
Update database classification level

Updates the [classification level](https://developer.atlassian.com/cloud/admin/dlp/rest/intro/#Classification%20level) for a specific database.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and permission to edit the database.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the database for which classification level should be updated. | [required] |
**put_whiteboard_classification_level_request** | [**PutWhiteboardClassificationLevelRequest**](PutWhiteboardClassificationLevelRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_page_classification_level

> put_page_classification_level(id, put_page_classification_level_request)
Update page classification level

Updates the [classification level](https://developer.atlassian.com/cloud/admin/dlp/rest/intro/#Classification%20level) for a specific page.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and permission to edit the page.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page for which classification level should be updated. | [required] |
**put_page_classification_level_request** | [**PutPageClassificationLevelRequest**](PutPageClassificationLevelRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_space_default_classification_level

> put_space_default_classification_level(id, put_space_default_classification_level_request)
Update space default classification level

Update the [default classification level](https://support.atlassian.com/security-and-access-policies/docs/what-is-a-default-classification-level/)  for a specific space.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and 'Admin' permission for the space.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the space for which default classification level should be updated. | [required] |
**put_space_default_classification_level_request** | [**PutSpaceDefaultClassificationLevelRequest**](PutSpaceDefaultClassificationLevelRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_whiteboard_classification_level

> put_whiteboard_classification_level(id, put_whiteboard_classification_level_request)
Update whiteboard classification level

Updates the [classification level](https://developer.atlassian.com/cloud/admin/dlp/rest/intro/#Classification%20level) for a specific whiteboard.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: 'Permission to access the Confluence site ('Can use' global permission) and permission to edit the whiteboard.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the whiteboard for which classification level should be updated. | [required] |
**put_whiteboard_classification_level_request** | [**PutWhiteboardClassificationLevelRequest**](PutWhiteboardClassificationLevelRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
