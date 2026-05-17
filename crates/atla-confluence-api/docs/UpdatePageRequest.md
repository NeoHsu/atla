# UpdatePageRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Id of the page. |
**status** | **Status** | The updated status of the page.  Note, if you change the status of a page from 'current' to 'draft' and it has an existing draft, the existing draft will be deleted in favor of the updated draft. Additionally, this endpoint can be used to restore a 'trashed' or 'deleted' page to 'current' status. For restoration, page contents will not be updated and only the page status will be changed. (enum: current, draft) |
**title** | **String** | Title of the page. |
**space_id** | Option<**serde_json::Value**> | ID of the containing space.  This currently **does not support moving the page to a different space**. | [optional]
**parent_id** | Option<**serde_json::Value**> | ID of the parent content.  This allows the page to be moved under a different parent within the same space. | [optional]
**owner_id** | Option<**serde_json::Value**> | Account ID of the page owner.  This allows page ownership to be transferred to another user. | [optional]
**body** | [**models::CreatePageRequestBody**](CreatePageRequestBody.md) |  |
**version** | [**models::UpdatePageRequestVersion**](UpdatePageRequestVersion.md) |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
