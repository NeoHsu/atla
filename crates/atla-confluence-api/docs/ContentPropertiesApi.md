# \ContentPropertiesApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_attachment_property**](ContentPropertiesApi.md#create_attachment_property) | **POST** /attachments/{attachment_id}/properties | Create content property for attachment
[**create_blogpost_property**](ContentPropertiesApi.md#create_blogpost_property) | **POST** /blogposts/{blogpost_id}/properties | Create content property for blog post
[**create_comment_property**](ContentPropertiesApi.md#create_comment_property) | **POST** /comments/{comment_id}/properties | Create content property for comment
[**create_custom_content_property**](ContentPropertiesApi.md#create_custom_content_property) | **POST** /custom-content/{custom_content_id}/properties | Create content property for custom content
[**create_database_property**](ContentPropertiesApi.md#create_database_property) | **POST** /databases/{id}/properties | Create content property for database
[**create_folder_property**](ContentPropertiesApi.md#create_folder_property) | **POST** /folders/{id}/properties | Create content property for folder
[**create_page_property**](ContentPropertiesApi.md#create_page_property) | **POST** /pages/{page_id}/properties | Create content property for page
[**create_smart_link_property**](ContentPropertiesApi.md#create_smart_link_property) | **POST** /embeds/{id}/properties | Create content property for Smart Link in the content tree
[**create_whiteboard_property**](ContentPropertiesApi.md#create_whiteboard_property) | **POST** /whiteboards/{id}/properties | Create content property for whiteboard
[**delete_attachment_property_by_id**](ContentPropertiesApi.md#delete_attachment_property_by_id) | **DELETE** /attachments/{attachment_id}/properties/{property_id} | Delete content property for attachment by id
[**delete_blogpost_property_by_id**](ContentPropertiesApi.md#delete_blogpost_property_by_id) | **DELETE** /blogposts/{blogpost_id}/properties/{property_id} | Delete content property for blogpost by id
[**delete_comment_property_by_id**](ContentPropertiesApi.md#delete_comment_property_by_id) | **DELETE** /comments/{comment_id}/properties/{property_id} | Delete content property for comment by id
[**delete_custom_content_property_by_id**](ContentPropertiesApi.md#delete_custom_content_property_by_id) | **DELETE** /custom-content/{custom_content_id}/properties/{property_id} | Delete content property for custom content by id
[**delete_database_property_by_id**](ContentPropertiesApi.md#delete_database_property_by_id) | **DELETE** /databases/{database_id}/properties/{property_id} | Delete content property for database by id
[**delete_folder_property_by_id**](ContentPropertiesApi.md#delete_folder_property_by_id) | **DELETE** /folders/{folder_id}/properties/{property_id} | Delete content property for folder by id
[**delete_page_property_by_id**](ContentPropertiesApi.md#delete_page_property_by_id) | **DELETE** /pages/{page_id}/properties/{property_id} | Delete content property for page by id
[**delete_smart_link_property_by_id**](ContentPropertiesApi.md#delete_smart_link_property_by_id) | **DELETE** /embeds/{embed_id}/properties/{property_id} | Delete content property for Smart Link in the content tree by id
[**delete_whiteboard_property_by_id**](ContentPropertiesApi.md#delete_whiteboard_property_by_id) | **DELETE** /whiteboards/{whiteboard_id}/properties/{property_id} | Delete content property for whiteboard by id
[**get_attachment_content_properties**](ContentPropertiesApi.md#get_attachment_content_properties) | **GET** /attachments/{attachment_id}/properties | Get content properties for attachment
[**get_attachment_content_properties_by_id**](ContentPropertiesApi.md#get_attachment_content_properties_by_id) | **GET** /attachments/{attachment_id}/properties/{property_id} | Get content property for attachment by id
[**get_blogpost_content_properties**](ContentPropertiesApi.md#get_blogpost_content_properties) | **GET** /blogposts/{blogpost_id}/properties | Get content properties for blog post
[**get_blogpost_content_properties_by_id**](ContentPropertiesApi.md#get_blogpost_content_properties_by_id) | **GET** /blogposts/{blogpost_id}/properties/{property_id} | Get content property for blog post by id
[**get_comment_content_properties**](ContentPropertiesApi.md#get_comment_content_properties) | **GET** /comments/{comment_id}/properties | Get content properties for comment
[**get_comment_content_properties_by_id**](ContentPropertiesApi.md#get_comment_content_properties_by_id) | **GET** /comments/{comment_id}/properties/{property_id} | Get content property for comment by id
[**get_custom_content_content_properties**](ContentPropertiesApi.md#get_custom_content_content_properties) | **GET** /custom-content/{custom_content_id}/properties | Get content properties for custom content
[**get_custom_content_content_properties_by_id**](ContentPropertiesApi.md#get_custom_content_content_properties_by_id) | **GET** /custom-content/{custom_content_id}/properties/{property_id} | Get content property for custom content by id
[**get_database_content_properties**](ContentPropertiesApi.md#get_database_content_properties) | **GET** /databases/{id}/properties | Get content properties for database
[**get_database_content_properties_by_id**](ContentPropertiesApi.md#get_database_content_properties_by_id) | **GET** /databases/{database_id}/properties/{property_id} | Get content property for database by id
[**get_folder_content_properties**](ContentPropertiesApi.md#get_folder_content_properties) | **GET** /folders/{id}/properties | Get content properties for folder
[**get_folder_content_properties_by_id**](ContentPropertiesApi.md#get_folder_content_properties_by_id) | **GET** /folders/{folder_id}/properties/{property_id} | Get content property for folder by id
[**get_page_content_properties**](ContentPropertiesApi.md#get_page_content_properties) | **GET** /pages/{page_id}/properties | Get content properties for page
[**get_page_content_properties_by_id**](ContentPropertiesApi.md#get_page_content_properties_by_id) | **GET** /pages/{page_id}/properties/{property_id} | Get content property for page by id
[**get_smart_link_content_properties**](ContentPropertiesApi.md#get_smart_link_content_properties) | **GET** /embeds/{id}/properties | Get content properties for Smart Link in the content tree
[**get_smart_link_content_properties_by_id**](ContentPropertiesApi.md#get_smart_link_content_properties_by_id) | **GET** /embeds/{embed_id}/properties/{property_id} | Get content property for Smart Link in the content tree by id
[**get_whiteboard_content_properties**](ContentPropertiesApi.md#get_whiteboard_content_properties) | **GET** /whiteboards/{id}/properties | Get content properties for whiteboard
[**get_whiteboard_content_properties_by_id**](ContentPropertiesApi.md#get_whiteboard_content_properties_by_id) | **GET** /whiteboards/{whiteboard_id}/properties/{property_id} | Get content property for whiteboard by id
[**update_attachment_property_by_id**](ContentPropertiesApi.md#update_attachment_property_by_id) | **PUT** /attachments/{attachment_id}/properties/{property_id} | Update content property for attachment by id
[**update_blogpost_property_by_id**](ContentPropertiesApi.md#update_blogpost_property_by_id) | **PUT** /blogposts/{blogpost_id}/properties/{property_id} | Update content property for blog post by id
[**update_comment_property_by_id**](ContentPropertiesApi.md#update_comment_property_by_id) | **PUT** /comments/{comment_id}/properties/{property_id} | Update content property for comment by id
[**update_custom_content_property_by_id**](ContentPropertiesApi.md#update_custom_content_property_by_id) | **PUT** /custom-content/{custom_content_id}/properties/{property_id} | Update content property for custom content by id
[**update_database_property_by_id**](ContentPropertiesApi.md#update_database_property_by_id) | **PUT** /databases/{database_id}/properties/{property_id} | Update content property for database by id
[**update_folder_property_by_id**](ContentPropertiesApi.md#update_folder_property_by_id) | **PUT** /folders/{folder_id}/properties/{property_id} | Update content property for folder by id
[**update_page_property_by_id**](ContentPropertiesApi.md#update_page_property_by_id) | **PUT** /pages/{page_id}/properties/{property_id} | Update content property for page by id
[**update_smart_link_property_by_id**](ContentPropertiesApi.md#update_smart_link_property_by_id) | **PUT** /embeds/{embed_id}/properties/{property_id} | Update content property for Smart Link in the content tree by id
[**update_whiteboard_property_by_id**](ContentPropertiesApi.md#update_whiteboard_property_by_id) | **PUT** /whiteboards/{whiteboard_id}/properties/{property_id} | Update content property for whiteboard by id



## create_attachment_property

> models::ContentProperty create_attachment_property(attachment_id, content_property_create_request)
Create content property for attachment

Creates a new content property for an attachment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to update the attachment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**attachment_id** | **String** | The ID of the attachment to create a property for. | [required] |
**content_property_create_request** | [**ContentPropertyCreateRequest**](ContentPropertyCreateRequest.md) | The content property to be created | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_blogpost_property

> models::ContentProperty create_blogpost_property(blogpost_id, content_property_create_request)
Create content property for blog post

Creates a new property for a blogpost.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to update the blog post.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**blogpost_id** | **i64** | The ID of the blog post to create a property for. | [required] |
**content_property_create_request** | [**ContentPropertyCreateRequest**](ContentPropertyCreateRequest.md) | The content property to be created | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_comment_property

> models::ContentProperty create_comment_property(comment_id, content_property_create_request)
Create content property for comment

Creates a new content property for a comment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to update the comment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**comment_id** | **i64** | The ID of the comment to create a property for. | [required] |
**content_property_create_request** | [**ContentPropertyCreateRequest**](ContentPropertyCreateRequest.md) | The content property to be created | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_custom_content_property

> models::ContentProperty create_custom_content_property(custom_content_id, content_property_create_request)
Create content property for custom content

Creates a new content property for a piece of custom content.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to update the custom content.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**custom_content_id** | **i64** | The ID of the custom content to create a property for. | [required] |
**content_property_create_request** | [**ContentPropertyCreateRequest**](ContentPropertyCreateRequest.md) | The content property to be created | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_database_property

> models::ContentProperty create_database_property(id, content_property_create_request)
Create content property for database

Creates a new content property for a database.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to update the database.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the database to create a property for. | [required] |
**content_property_create_request** | [**ContentPropertyCreateRequest**](ContentPropertyCreateRequest.md) | The content property to be created | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_folder_property

> models::ContentProperty create_folder_property(id, content_property_create_request)
Create content property for folder

Creates a new content property for a folder.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to update the folder.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the folder to create a property for. | [required] |
**content_property_create_request** | [**ContentPropertyCreateRequest**](ContentPropertyCreateRequest.md) | The content property to be created | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_page_property

> models::ContentProperty create_page_property(page_id, content_property_create_request)
Create content property for page

Creates a new content property for a page.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to update the page.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_id** | **i64** | The ID of the page to create a property for. | [required] |
**content_property_create_request** | [**ContentPropertyCreateRequest**](ContentPropertyCreateRequest.md) | The content property to be created | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_smart_link_property

> models::ContentProperty create_smart_link_property(id, content_property_create_request)
Create content property for Smart Link in the content tree

Creates a new content property for a Smart Link in the content tree.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to update the Smart Link in the content tree.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the Smart Link in the content tree to create a property for. | [required] |
**content_property_create_request** | [**ContentPropertyCreateRequest**](ContentPropertyCreateRequest.md) | The content property to be created | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_whiteboard_property

> models::ContentProperty create_whiteboard_property(id, content_property_create_request)
Create content property for whiteboard

Creates a new content property for a whiteboard.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to update the whiteboard.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the whiteboard to create a property for. | [required] |
**content_property_create_request** | [**ContentPropertyCreateRequest**](ContentPropertyCreateRequest.md) | The content property to be created | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_attachment_property_by_id

> delete_attachment_property_by_id(attachment_id, property_id)
Delete content property for attachment by id

Deletes a content property for an attachment by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to attachment the page.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**attachment_id** | **String** | The ID of the attachment the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_blogpost_property_by_id

> delete_blogpost_property_by_id(blogpost_id, property_id)
Delete content property for blogpost by id

Deletes a content property for a blogpost by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the blog post.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**blogpost_id** | **i64** | The ID of the blog post the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_comment_property_by_id

> delete_comment_property_by_id(comment_id, property_id)
Delete content property for comment by id

Deletes a content property for a comment by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the comment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**comment_id** | **i64** | The ID of the comment the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_custom_content_property_by_id

> delete_custom_content_property_by_id(custom_content_id, property_id)
Delete content property for custom content by id

Deletes a content property for a piece of custom content by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the custom content.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**custom_content_id** | **i64** | The ID of the custom content the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_database_property_by_id

> delete_database_property_by_id(database_id, property_id)
Delete content property for database by id

Deletes a content property for a database by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the database.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **i64** | The ID of the database the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_folder_property_by_id

> delete_folder_property_by_id(folder_id, property_id)
Delete content property for folder by id

Deletes a content property for a folder by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the folder.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**folder_id** | **i64** | The ID of the folder the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_page_property_by_id

> delete_page_property_by_id(page_id, property_id)
Delete content property for page by id

Deletes a content property for a page by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the page.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_id** | **i64** | The ID of the page the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_smart_link_property_by_id

> delete_smart_link_property_by_id(embed_id, property_id)
Delete content property for Smart Link in the content tree by id

Deletes a content property for a Smart Link in the content tree by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the Smart Link in the content tree.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**embed_id** | **i64** | The ID of the Smart Link in the content tree the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_whiteboard_property_by_id

> delete_whiteboard_property_by_id(whiteboard_id, property_id)
Delete content property for whiteboard by id

Deletes a content property for a whiteboard by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the whiteboard.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**whiteboard_id** | **i64** | The ID of the whiteboard the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be deleted. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_attachment_content_properties

> models::MultiEntityResultContentProperty get_attachment_content_properties(attachment_id, key, sort, cursor, limit)
Get content properties for attachment

Retrieves all Content Properties tied to a specified attachment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the attachment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**attachment_id** | **String** | The ID of the attachment for which content properties should be returned. | [required] |
**key** | Option<**String**> | Filters the response to return a specific content property with matching key (case sensitive). |  |
**sort** | Option<[**ContentPropertySortOrder**](ContentPropertySortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of attachments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultContentProperty**](MultiEntityResult_ContentProperty_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_attachment_content_properties_by_id

> models::ContentProperty get_attachment_content_properties_by_id(attachment_id, property_id)
Get content property for attachment by id

Retrieves a specific Content Property by ID that is attached to a specified attachment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the attachment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**attachment_id** | **String** | The ID of the attachment for which content properties should be returned. | [required] |
**property_id** | **i64** | The ID of the content property to be returned | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_blogpost_content_properties

> models::MultiEntityResultContentProperty get_blogpost_content_properties(blogpost_id, key, sort, cursor, limit)
Get content properties for blog post

Retrieves all Content Properties tied to a specified blog post.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the blog post.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**blogpost_id** | **i64** | The ID of the blog post for which content properties should be returned. | [required] |
**key** | Option<**String**> | Filters the response to return a specific content property with matching key (case sensitive). |  |
**sort** | Option<[**ContentPropertySortOrder**](ContentPropertySortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of attachments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultContentProperty**](MultiEntityResult_ContentProperty_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_blogpost_content_properties_by_id

> models::ContentProperty get_blogpost_content_properties_by_id(blogpost_id, property_id)
Get content property for blog post by id

Retrieves a specific Content Property by ID that is attached to a specified blog post.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the blog post.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**blogpost_id** | **i64** | The ID of the blog post for which content properties should be returned. | [required] |
**property_id** | **i64** | The ID of the property being requested | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_comment_content_properties

> models::MultiEntityResultContentProperty get_comment_content_properties(comment_id, key, sort, cursor, limit)
Get content properties for comment

Retrieves Content Properties attached to a specified comment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the comment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**comment_id** | **i64** | The ID of the comment for which content properties should be returned. | [required] |
**key** | Option<**String**> | Filters the response to return a specific content property with matching key (case sensitive). |  |
**sort** | Option<[**ContentPropertySortOrder**](ContentPropertySortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of attachments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultContentProperty**](MultiEntityResult_ContentProperty_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_comment_content_properties_by_id

> models::ContentProperty get_comment_content_properties_by_id(comment_id, property_id)
Get content property for comment by id

Retrieves a specific Content Property by ID that is attached to a specified comment.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the comment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**comment_id** | **i64** | The ID of the comment for which content properties should be returned. | [required] |
**property_id** | **i64** | The ID of the content property being requested. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_custom_content_content_properties

> models::MultiEntityResultContentProperty get_custom_content_content_properties(custom_content_id, key, sort, cursor, limit)
Get content properties for custom content

Retrieves Content Properties tied to a specified custom content.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the custom content.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**custom_content_id** | **i64** | The ID of the custom content for which content properties should be returned. | [required] |
**key** | Option<**String**> | Filters the response to return a specific content property with matching key (case sensitive). |  |
**sort** | Option<[**ContentPropertySortOrder**](ContentPropertySortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of attachments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultContentProperty**](MultiEntityResult_ContentProperty_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_custom_content_content_properties_by_id

> models::ContentProperty get_custom_content_content_properties_by_id(custom_content_id, property_id)
Get content property for custom content by id

Retrieves a specific Content Property by ID that is attached to a specified custom content.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the page.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**custom_content_id** | **i64** | The ID of the custom content for which content properties should be returned. | [required] |
**property_id** | **i64** | The ID of the content property being requested. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_database_content_properties

> models::MultiEntityResultContentProperty get_database_content_properties(id, key, sort, cursor, limit)
Get content properties for database

Retrieves Content Properties tied to a specified database.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the database.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the database for which content properties should be returned. | [required] |
**key** | Option<**String**> | Filters the response to return a specific content property with matching key (case sensitive). |  |
**sort** | Option<[**ContentPropertySortOrder**](ContentPropertySortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of attachments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultContentProperty**](MultiEntityResult_ContentProperty_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_database_content_properties_by_id

> models::ContentProperty get_database_content_properties_by_id(database_id, property_id)
Get content property for database by id

Retrieves a specific Content Property by ID that is attached to a specified database.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the database.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **i64** | The ID of the database for which content properties should be returned. | [required] |
**property_id** | **i64** | The ID of the content property being requested. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_folder_content_properties

> models::MultiEntityResultContentProperty get_folder_content_properties(id, key, sort, cursor, limit)
Get content properties for folder

Retrieves Content Properties tied to a specified folder.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the folder.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the folder for which content properties should be returned. | [required] |
**key** | Option<**String**> | Filters the response to return a specific content property with matching key (case sensitive). |  |
**sort** | Option<[**ContentPropertySortOrder**](ContentPropertySortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of attachments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultContentProperty**](MultiEntityResult_ContentProperty_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_folder_content_properties_by_id

> models::ContentProperty get_folder_content_properties_by_id(folder_id, property_id)
Get content property for folder by id

Retrieves a specific Content Property by ID that is attached to a specified folder.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the folder.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**folder_id** | **i64** | The ID of the folder for which content properties should be returned. | [required] |
**property_id** | **i64** | The ID of the content property being requested. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_content_properties

> models::MultiEntityResultContentProperty get_page_content_properties(page_id, key, sort, cursor, limit)
Get content properties for page

Retrieves Content Properties tied to a specified page.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the page.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_id** | **i64** | The ID of the page for which content properties should be returned. | [required] |
**key** | Option<**String**> | Filters the response to return a specific content property with matching key (case sensitive). |  |
**sort** | Option<[**ContentPropertySortOrder**](ContentPropertySortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of attachments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultContentProperty**](MultiEntityResult_ContentProperty_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_page_content_properties_by_id

> models::ContentProperty get_page_content_properties_by_id(page_id, property_id)
Get content property for page by id

Retrieves a specific Content Property by ID that is attached to a specified page.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the page.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_id** | **i64** | The ID of the page for which content properties should be returned. | [required] |
**property_id** | **i64** | The ID of the content property being requested. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_smart_link_content_properties

> models::MultiEntityResultContentProperty get_smart_link_content_properties(id, key, sort, cursor, limit)
Get content properties for Smart Link in the content tree

Retrieves Content Properties tied to a specified Smart Link in the content tree.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the Smart Link in the content tree.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the Smart Link in the content tree for which content properties should be returned. | [required] |
**key** | Option<**String**> | Filters the response to return a specific content property with matching key (case sensitive). |  |
**sort** | Option<[**ContentPropertySortOrder**](ContentPropertySortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of Smart Links per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultContentProperty**](MultiEntityResult_ContentProperty_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_smart_link_content_properties_by_id

> models::ContentProperty get_smart_link_content_properties_by_id(embed_id, property_id)
Get content property for Smart Link in the content tree by id

Retrieves a specific Content Property by ID that is attached to a specified Smart Link in the content tree.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the Smart Link in the content tree.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**embed_id** | **i64** | The ID of the Smart Link in the content tree for which content properties should be returned. | [required] |
**property_id** | **i64** | The ID of the content property being requested. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_whiteboard_content_properties

> models::MultiEntityResultContentProperty get_whiteboard_content_properties(id, key, sort, cursor, limit)
Get content properties for whiteboard

Retrieves Content Properties tied to a specified whiteboard.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the whiteboard.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the whiteboard for which content properties should be returned. | [required] |
**key** | Option<**String**> | Filters the response to return a specific content property with matching key (case sensitive). |  |
**sort** | Option<[**ContentPropertySortOrder**](ContentPropertySortOrder.md)> | Used to sort the result by a particular field. |  |
**cursor** | Option<**String**> | Used for pagination, this opaque cursor will be returned in the `next` URL in the `Link` response header. Use the relative URL in the `Link` header to retrieve the `next` set of results. |  |
**limit** | Option<**i32**> | Maximum number of attachments per result to return. If more results exist, use the `Link` header to retrieve a relative URL that will return the next set of results. |  |[default to 25]

### Return type

[**models::MultiEntityResultContentProperty**](MultiEntityResult_ContentProperty_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_whiteboard_content_properties_by_id

> models::ContentProperty get_whiteboard_content_properties_by_id(whiteboard_id, property_id)
Get content property for whiteboard by id

Retrieves a specific Content Property by ID that is attached to a specified whiteboard.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the whiteboard.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**whiteboard_id** | **i64** | The ID of the whiteboard for which content properties should be returned. | [required] |
**property_id** | **i64** | The ID of the content property being requested. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_attachment_property_by_id

> models::ContentProperty update_attachment_property_by_id(attachment_id, property_id, content_property_update_request)
Update content property for attachment by id

Update a content property for attachment by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the attachment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**attachment_id** | **String** | The ID of the attachment the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be updated. | [required] |
**content_property_update_request** | [**ContentPropertyUpdateRequest**](ContentPropertyUpdateRequest.md) | The content property to be updated. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_blogpost_property_by_id

> models::ContentProperty update_blogpost_property_by_id(blogpost_id, property_id, content_property_update_request)
Update content property for blog post by id

Update a content property for blog post by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the blog post.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**blogpost_id** | **i64** | The ID of the blog post the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be updated. | [required] |
**content_property_update_request** | [**ContentPropertyUpdateRequest**](ContentPropertyUpdateRequest.md) | The content property to be updated. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_comment_property_by_id

> models::ContentProperty update_comment_property_by_id(comment_id, property_id, content_property_update_request)
Update content property for comment by id

Update a content property for a comment by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the comment.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**comment_id** | **i64** | The ID of the comment the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be updated. | [required] |
**content_property_update_request** | [**ContentPropertyUpdateRequest**](ContentPropertyUpdateRequest.md) | The content property to be updated. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_custom_content_property_by_id

> models::ContentProperty update_custom_content_property_by_id(custom_content_id, property_id, content_property_update_request)
Update content property for custom content by id

Update a content property for a piece of custom content by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the custom content.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**custom_content_id** | **i64** | The ID of the custom content the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be updated. | [required] |
**content_property_update_request** | [**ContentPropertyUpdateRequest**](ContentPropertyUpdateRequest.md) | The content property to be updated. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_database_property_by_id

> models::ContentProperty update_database_property_by_id(database_id, property_id, content_property_update_request)
Update content property for database by id

Update a content property for a database by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the database.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**database_id** | **i64** | The ID of the database the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be updated. | [required] |
**content_property_update_request** | [**ContentPropertyUpdateRequest**](ContentPropertyUpdateRequest.md) | The content property to be updated. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_folder_property_by_id

> models::ContentProperty update_folder_property_by_id(folder_id, property_id, content_property_update_request)
Update content property for folder by id

Update a content property for a folder by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the folder.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**folder_id** | **i64** | The ID of the folder the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be updated. | [required] |
**content_property_update_request** | [**ContentPropertyUpdateRequest**](ContentPropertyUpdateRequest.md) | The content property to be updated. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_page_property_by_id

> models::ContentProperty update_page_property_by_id(page_id, property_id, content_property_update_request)
Update content property for page by id

Update a content property for a page by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the page.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page_id** | **i64** | The ID of the page the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be updated. | [required] |
**content_property_update_request** | [**ContentPropertyUpdateRequest**](ContentPropertyUpdateRequest.md) | The content property to be updated. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_smart_link_property_by_id

> models::ContentProperty update_smart_link_property_by_id(embed_id, property_id, content_property_update_request)
Update content property for Smart Link in the content tree by id

Update a content property for a Smart Link in the content tree by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the Smart Link in the content tree.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**embed_id** | **i64** | The ID of the Smart Link in the content tree the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be updated. | [required] |
**content_property_update_request** | [**ContentPropertyUpdateRequest**](ContentPropertyUpdateRequest.md) | The content property to be updated. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_whiteboard_property_by_id

> models::ContentProperty update_whiteboard_property_by_id(whiteboard_id, property_id, content_property_update_request)
Update content property for whiteboard by id

Update a content property for a whiteboard by its id.   **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to edit the whiteboard.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**whiteboard_id** | **i64** | The ID of the whiteboard the property belongs to. | [required] |
**property_id** | **i64** | The ID of the property to be updated. | [required] |
**content_property_update_request** | [**ContentPropertyUpdateRequest**](ContentPropertyUpdateRequest.md) | The content property to be updated. | [required] |

### Return type

[**models::ContentProperty**](ContentProperty.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
