# UpdateBlogPostRequestVersion

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**number** | Option<**i32**> | The new version number of the updated blog post.  Set this to the current version number plus one, unless you are updating the status to 'draft' which requires a version number of 1.  If you don't know the current version number, use Get blog post by id. | [optional]
**message** | Option<**String**> | An optional message to be stored with the version. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
