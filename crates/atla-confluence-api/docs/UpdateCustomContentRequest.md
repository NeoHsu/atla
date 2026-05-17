# UpdateCustomContentRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Id of custom content. |
**r#type** | **String** | Type of custom content. |
**status** | **Status** | The status of the custom content. (enum: current) |
**space_id** | Option<**String**> | ID of the containing space (must be the same as the spaceId of the space the custom content was created in). | [optional]
**page_id** | Option<**String**> | ID of the containing page. | [optional]
**blog_post_id** | Option<**String**> | ID of the containing Blog Post. | [optional]
**custom_content_id** | Option<**String**> | ID of the containing custom content. | [optional]
**title** | **String** | Title of the custom content. |
**body** | [**models::CreateCustomContentRequestBody**](CreateCustomContentRequestBody.md) |  |
**version** | [**models::UpdateCustomContentRequestVersion**](UpdateCustomContentRequestVersion.md) |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
