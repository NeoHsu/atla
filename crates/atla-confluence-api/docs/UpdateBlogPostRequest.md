# UpdateBlogPostRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Id of the blog post. |
**status** | **Status** | The updated status of the blog post.  Note, if you change the status of a blog post from 'current' to 'draft' and it has an existing draft, the existing draft will be deleted in favor of the updated draft. Additionally, this endpoint can be used to restore a 'trashed' or 'deleted' blog post to 'current' status. For restoration, blog post contents will not be updated and only the blog post status will be changed. (enum: current, draft) |
**title** | **String** | Title of the blog post. |
**space_id** | Option<**String**> | ID of the containing space.  This currently **does not support moving the blog post to a different space**. | [optional]
**body** | [**models::CreateBlogPostRequestBody**](CreateBlogPostRequestBody.md) |  |
**version** | [**models::UpdateBlogPostRequestVersion**](UpdateBlogPostRequestVersion.md) |  |
**created_at** | Option<**String**> | Created date of the blog post in the format of \"yyyy-MM-ddTHH:mm:ss.SSSZ\". | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
