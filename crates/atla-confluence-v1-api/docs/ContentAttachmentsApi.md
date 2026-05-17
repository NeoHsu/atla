# \ContentAttachmentsApi

All URIs are relative to *http://your-domain.atlassian.net*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_attachment**](ContentAttachmentsApi.md#create_attachment) | **POST** /wiki/rest/api/content/{id}/child/attachment | Create attachment
[**create_or_update_attachments**](ContentAttachmentsApi.md#create_or_update_attachments) | **PUT** /wiki/rest/api/content/{id}/child/attachment | Create or update attachment



## create_attachment

> models::ContentArray create_attachment(id, x_atlassian_token, file, minor_edit, status, comment)
Create attachment

Adds an attachment to a piece of content. This method only adds a new attachment. If you want to update an existing attachment, use [Create or update attachments](#api-content-id-child-attachment-put).  Note, you must set a `X-Atlassian-Token: nocheck` header on the request for this method, otherwise it will be blocked. This protects against XSRF attacks, which is necessary as this method accepts multipart/form-data.  The media type 'multipart/form-data' is defined in [RFC 7578](https://www.ietf.org/rfc/rfc7578.txt). Most client libraries have classes that make it easier to implement multipart posts, like the [MultipartEntityBuilder](https://hc.apache.org/httpcomponents-client-5.1.x/current/httpclient5/apidocs/) Java class provided by Apache HTTP Components.  Note, according to [RFC 7578](https://tools.ietf.org/html/rfc7578#section-4.5), in the case where the form data is text, the charset parameter for the \"text/plain\" Content-Type may be used to indicate the character encoding used in that part. In the case of this API endpoint, the `comment` body parameter should be sent with `type=text/plain` and `charset=utf-8` values. This will force the charset to be UTF-8.  Example: This curl command attaches a file ('example.txt') to a container (id='123') with a comment and `minorEdits`=true.  ``` bash curl -D- \\   -u admin:admin \\   -X POST \\   -H 'X-Atlassian-Token: nocheck' \\   -F 'file=@\"example.txt\"' \\   -F 'minorEdit=\"true\"' \\   -F 'comment=\"Example attachment comment\"; type=text/plain; charset=utf-8' \\   https://myhost/wiki/rest/api/content/123/child/attachment ``` **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to update the content.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the content to add the attachment to. | [required] |
**x_atlassian_token** | **String** | Required by Confluence for attachment multipart requests. Use `nocheck`. | [required] |[default to nocheck]
**file** | **std::path::PathBuf** | The relative location and name of the attachment to be added to the content. | [required] |
**minor_edit** | **bool** | If `minorEdits` is set to 'true', no notification email or activity stream will be generated when the attachment is added to the content. | [required] |[default to false]
**status** | Option<**String**> | The status of the content that the attachment is being added to. |  |[default to current]
**comment** | Option<**String**> | The comment for the attachment that is being added. If you specify a comment, then every file must have a comment and the comments must be in the same order as the files. Alternatively, don't specify any comments. |  |

### Return type

[**models::ContentArray**](ContentArray.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_or_update_attachments

> models::ContentArray create_or_update_attachments(id, x_atlassian_token, file, minor_edit, status, comment)
Create or update attachment

Adds an attachment to a piece of content. If the attachment already exists for the content, then the attachment is updated (i.e. a new version of the attachment is created).  Note, you must set a `X-Atlassian-Token: nocheck` header on the request for this method, otherwise it will be blocked. This protects against XSRF attacks, which is necessary as this method accepts multipart/form-data.  The media type 'multipart/form-data' is defined in [RFC 7578](https://www.ietf.org/rfc/rfc7578.txt). Most client libraries have classes that make it easier to implement multipart posts, like the [MultipartEntityBuilder](https://hc.apache.org/httpcomponents-client-5.1.x/current/httpclient5/apidocs/) Java class provided by Apache HTTP Components.  Note, according to [RFC 7578](https://tools.ietf.org/html/rfc7578#section-4.5), in the case where the form data is text, the charset parameter for the \"text/plain\" Content-Type may be used to indicate the character encoding used in that part. In the case of this API endpoint, the `comment` body parameter should be sent with `type=text/plain` and `charset=utf-8` values. This will force the charset to be UTF-8.  Example: This curl command attaches a file ('example.txt') to a piece of content (id='123') with a comment and `minorEdits`=true. If the 'example.txt' file already exists, it will update it with a new version of the attachment.  ``` bash curl -D- \\   -u admin:admin \\   -X PUT \\   -H 'X-Atlassian-Token: nocheck' \\   -F 'file=@\"example.txt\"' \\   -F 'minorEdit=\"true\"' \\   -F 'comment=\"Example attachment comment\"; type=text/plain; charset=utf-8' \\   http://myhost/rest/api/content/123/child/attachment ``` **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to update the content.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | The ID of the content to add the attachment to. | [required] |
**x_atlassian_token** | **String** | Required by Confluence for attachment multipart requests. Use `nocheck`. | [required] |[default to nocheck]
**file** | **std::path::PathBuf** | The relative location and name of the attachment to be added to the content. | [required] |
**minor_edit** | **bool** | If `minorEdits` is set to 'true', no notification email or activity stream will be generated when the attachment is added to the content. | [required] |[default to false]
**status** | Option<**String**> | The status of the content that the attachment is being added to. This should always be set to 'current'. |  |[default to current]
**comment** | Option<**String**> | The comment for the attachment that is being added. If you specify a comment, then every file must have a comment and the comments must be in the same order as the files. Alternatively, don't specify any comments. |  |

### Return type

[**models::ContentArray**](ContentArray.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
