# CreateCustomContentRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | **String** | Type of custom content. |
**status** | Option<**Status**> | The status of the custom content. Defaults to `current` when status not provided. (enum: current, draft) | [optional]
**space_id** | Option<**String**> | ID of the containing space. | [optional]
**page_id** | Option<**String**> | ID of the containing page. | [optional]
**blog_post_id** | Option<**String**> | ID of the containing Blog Post. | [optional]
**custom_content_id** | Option<**String**> | ID of the containing custom content. | [optional]
**title** | **String** | Title of the custom content. |
**body** | [**models::CreateCustomContentRequestBody**](CreateCustomContentRequestBody.md) |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
