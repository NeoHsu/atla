# \AdminKeyApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**disable_admin_key**](AdminKeyApi.md#disable_admin_key) | **DELETE** /admin-key | Disable Admin Key
[**enable_admin_key**](AdminKeyApi.md#enable_admin_key) | **POST** /admin-key | Enable Admin Key
[**get_admin_key**](AdminKeyApi.md#get_admin_key) | **GET** /admin-key | Get Admin Key



## disable_admin_key

> disable_admin_key()
Disable Admin Key

Disables admin key access for the calling user within the site.  **[Permissions](https://support.atlassian.com/user-management/docs/give-users-admin-permissions/#Centralized-user-management-content) required**: User must be an organization or site admin.

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## enable_admin_key

> models::AdminKeyResponse enable_admin_key(enable_admin_key_request)
Enable Admin Key

Enables admin key access for the calling user within the site. If an admin key already exists for the user, a new one will be issued with an updated expiration time.  **Note:** The `durationInMinutes` field within the request body is optional. If the request body is empty or if the `durationInMinutes` is set to 0 minutes, a new admin key will be issued to the calling user with a default duration of 10 minutes.  **[Permissions](https://support.atlassian.com/user-management/docs/give-users-admin-permissions/#Centralized-user-management-content) required**: User must be an organization or site admin.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**enable_admin_key_request** | Option<[**EnableAdminKeyRequest**](EnableAdminKeyRequest.md)> |  |  |

### Return type

[**models::AdminKeyResponse**](AdminKeyResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_admin_key

> models::AdminKeyResponse get_admin_key()
Get Admin Key

Returns information about the admin key if one is currently enabled for the calling user within the site.  **[Permissions](https://support.atlassian.com/user-management/docs/give-users-admin-permissions/#Centralized-user-management-content) required**: User must be an organization or site admin.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::AdminKeyResponse**](AdminKeyResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
