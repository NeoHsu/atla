# CreateBlogPostRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**space_id** | **String** | ID of the space |
**status** | Option<**Status**> | The status of the blog post, specifies if the blog post will be created as a new blog post or a draft (enum: current, draft) | [optional]
**title** | Option<**String**> | Title of the blog post, required if creating non-draft. | [optional]
**body** | Option<[**models::CreateBlogPostRequestBody**](CreateBlogPostRequestBody.md)> |  | [optional]
**created_at** | Option<**String**> | Created date of the blog post in the format of \"yyyy-MM-ddTHH:mm:ss.SSSZ\". | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
