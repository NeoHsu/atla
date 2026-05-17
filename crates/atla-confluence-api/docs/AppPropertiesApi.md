# \AppPropertiesApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_forge_app_property**](AppPropertiesApi.md#delete_forge_app_property) | **DELETE** /app/properties/{propertyKey} | Deletes a Forge app property.
[**get_forge_app_properties**](AppPropertiesApi.md#get_forge_app_properties) | **GET** /app/properties | Get Forge app properties.
[**get_forge_app_property**](AppPropertiesApi.md#get_forge_app_property) | **GET** /app/properties/{propertyKey} | Get a Forge app property by key.
[**put_forge_app_property**](AppPropertiesApi.md#put_forge_app_property) | **PUT** /app/properties/{propertyKey} | Create or update a Forge app property.



## delete_forge_app_property

> delete_forge_app_property(property_key)
Deletes a Forge app property.

Deletes a Forge app property. This API can only be accessed using **[asApp()](https://developer.atlassian.com/platform/forge/apis-reference/fetch-api-product.requestconfluence/#method-signature)** requests from Forge.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**property_key** | **String** | The key of the property | [required] |

### Return type

 (empty response body)

### Authorization

[oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_forge_app_properties

> models::MultiEntityResultAppProperty get_forge_app_properties(cursor, limit)
Get Forge app properties.

Gets Forge app properties. This API can only be accessed using **[asApp()](https://developer.atlassian.com/platform/forge/apis-reference/fetch-api-product.requestconfluence/#method-signature)** requests from Forge.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**cursor** | Option<**String**> | Used for pagination, this opaque cursor represents the last returned property key. It will be included in the response body as the next link. Use this key to request the next set of results. |  |
**limit** | Option<**i32**> | Maximum number of app properties per result to return. If more results exist, use the last returned property key from the Link field in the response body as a cursor to retrieve the next set of results. |  |[default to 50]

### Return type

[**models::MultiEntityResultAppProperty**](MultiEntityResult_AppProperty_.md)

### Authorization

[oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_forge_app_property

> models::GetForgeAppProperty200Response get_forge_app_property(property_key)
Get a Forge app property by key.

Gets a Forge app property by property key. This API can only be accessed using **[asApp()](https://developer.atlassian.com/platform/forge/apis-reference/fetch-api-product.requestconfluence/#method-signature)** requests from Forge.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**property_key** | **String** | The key of the property | [required] |

### Return type

[**models::GetForgeAppProperty200Response**](getForgeAppProperty_200_response.md)

### Authorization

[oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## put_forge_app_property

> put_forge_app_property(property_key, body)
Create or update a Forge app property.

Creates or updates a Forge app property. This API can only be accessed using **[asApp()](https://developer.atlassian.com/platform/forge/apis-reference/fetch-api-product.requestconfluence/#method-signature)** requests from Forge.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**property_key** | **String** | The key of the property | [required] |
**body** | **serde_json::Value** |  | [required] |

### Return type

 (empty response body)

### Authorization

[oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
