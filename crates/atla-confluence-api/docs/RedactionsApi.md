# \RedactionsApi

All URIs are relative to *https://no-default/wiki/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**post_redact_blog**](RedactionsApi.md#post_redact_blog) | **POST** /blogposts/{id}/redact | Redact Content in a Confluence Blog Post
[**post_redact_page**](RedactionsApi.md#post_redact_page) | **POST** /pages/{id}/redact | Redact Content in a Confluence Page



## post_redact_blog

> models::RedactionResponse post_redact_blog(id, post_redact_page_request)
Redact Content in a Confluence Blog Post

Redacts sensitive content in a Confluence blog post by replacing specified text ranges with redaction markers.  Each redaction in the response includes a unique UUID for restoration (except code block redactions).  The response metadata items maintain the same order as the input redaction pointers, and completely  overlapping redactions are merged into a single redaction with one UUID.  **Note**: This endpoint requires **Atlassian Guard Premium**.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the blog post to redact content from. | [required] |
**post_redact_page_request** | Option<[**PostRedactPageRequest**](PostRedactPageRequest.md)> |  |  |

### Return type

[**models::RedactionResponse**](RedactionResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_redact_page

> models::RedactionResponse post_redact_page(id, post_redact_page_request)
Redact Content in a Confluence Page

Redacts sensitive content in a Confluence page by replacing specified text ranges with redaction markers.  Each redaction in the response includes a unique UUID for restoration (except code block redactions).  The response metadata items maintain the same order as the input redaction pointers, and completely  overlapping redactions are merged into a single redaction with one UUID.  **Note**: This endpoint requires **Atlassian Guard Premium**.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **i64** | The ID of the page to redact content from. | [required] |
**post_redact_page_request** | Option<[**PostRedactPageRequest**](PostRedactPageRequest.md)> |  |  |

### Return type

[**models::RedactionResponse**](RedactionResponse.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
