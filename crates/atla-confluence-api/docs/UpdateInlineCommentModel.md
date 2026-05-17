# UpdateInlineCommentModel

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**version** | Option<[**models::UpdateFooterCommentModelVersion**](UpdateFooterCommentModelVersion.md)> |  | [optional]
**body** | Option<[**models::CreateFooterCommentModelBody**](CreateFooterCommentModelBody.md)> |  | [optional]
**resolved** | Option<**bool**> | Resolved state of the comment. Set to true to resolve the comment, set to false to reopen it. If matching the existing state (i.e. true -> resolved or false -> open/reopened) , no change will occur. A dangling comment cannot be updated. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
