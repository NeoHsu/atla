# \UserApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**check_access_by_email**](UserApi.md#check_access_by_email) | **POST** /user/access/check-access-by-email | Check site access for a list of emails
[**create_bulk_user_lookup**](UserApi.md#create_bulk_user_lookup) | **POST** /users-bulk | Create bulk user lookup using ids
[**invite_by_email**](UserApi.md#invite_by_email) | **POST** /user/access/invite-by-email | Invite a list of emails to the site



## check_access_by_email

> models::CheckAccessByEmail200Response check_access_by_email(check_access_by_email_request)
Check site access for a list of emails

Returns the list of emails from the input list that do not have access to site.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**check_access_by_email_request** | [**CheckAccessByEmailRequest**](CheckAccessByEmailRequest.md) |  | [required] |

### Return type

[**models::CheckAccessByEmail200Response**](checkAccessByEmail_200_response.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_bulk_user_lookup

> models::MultiEntityResultUser create_bulk_user_lookup(create_bulk_user_lookup_request)
Create bulk user lookup using ids

Returns user details for the ids provided in the request body.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). The user must be able to view user profiles in the Confluence site.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_bulk_user_lookup_request** | [**CreateBulkUserLookupRequest**](CreateBulkUserLookupRequest.md) |  | [required] |

### Return type

[**models::MultiEntityResultUser**](MultiEntityResult_User_.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## invite_by_email

> invite_by_email(check_access_by_email_request)
Invite a list of emails to the site

Invite a list of emails to the site.  Ignores all invalid emails and no action is taken for the emails that already have access to the site.  <b>NOTE:</b> This API is asynchronous and may take some time to complete.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**check_access_by_email_request** | [**CheckAccessByEmailRequest**](CheckAccessByEmailRequest.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
