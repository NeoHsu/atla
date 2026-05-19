# \ContentLabelsApi

All URIs are relative to *http://your-domain.atlassian.net*

Method | HTTP request | Description
------------- | ------------- | -------------
[**add_labels_to_content**](ContentLabelsApi.md#add_labels_to_content) | **POST** /wiki/rest/api/content/{id}/label | Add labels to content
[**remove_label_from_content_using_query_parameter**](ContentLabelsApi.md#remove_label_from_content_using_query_parameter) | **DELETE** /wiki/rest/api/content/{id}/label | Remove label from content using query parameter



## add_labels_to_content

> models::LabelArray add_labels_to_content(id, body)
Add labels to content

Adds labels to a piece of content. Does not modify the existing labels.  Notes:  - Labels can also be added when creating content ([Create content](#api-content-post)). - Labels can be updated when updating content ([Update content](#api-content-id-put)). This will delete the existing labels and replace them with the labels in the request.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to update the content.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the content that will have labels added to it. | [required] |
**body** | [**AddLabelsToContentRequest**](AddLabelsToContentRequest.md) | The labels to add to the content. | [required] |

### Return type

[**models::LabelArray**](LabelArray.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## remove_label_from_content_using_query_parameter

> remove_label_from_content_using_query_parameter(id, name)
Remove label from content using query parameter

Removes a label from a piece of content. Labels can't be deleted from archived content. This is similar to [Remove label from content](#api-content-id-label-label-delete) except that the label name is specified via a query parameter.  Use this method if the label name has \"/\" characters, as [Remove label from content using query parameter](#api-content-id-label-delete) does not accept \"/\" characters for the label name.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to update the content.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the content that the label will be removed from. | [required] |
**name** | **String** | The name of the label to be removed. | [required] |

### Return type

 (empty response body)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
