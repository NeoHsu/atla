# BlogPostSingle

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | ID of the blog post. | [optional]
**status** | Option<[**models::BlogPostContentStatus**](BlogPostContentStatus.md)> |  | [optional]
**title** | Option<**String**> | Title of the blog post. | [optional]
**space_id** | Option<**String**> | ID of the space the blog post is in. | [optional]
**author_id** | Option<**String**> | The account ID of the user who created this blog post originally. | [optional]
**created_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Date and time when the blog post was created. In format \"YYYY-MM-DDTHH:mm:ss.sssZ\". | [optional]
**version** | Option<[**models::Version**](Version.md)> |  | [optional]
**body** | Option<[**models::BodySingle**](BodySingle.md)> |  | [optional]
**labels** | Option<[**models::AttachmentSingleLabels**](AttachmentSingleLabels.md)> |  | [optional]
**properties** | Option<[**models::AttachmentSingleProperties**](AttachmentSingleProperties.md)> |  | [optional]
**operations** | Option<[**models::AttachmentSingleOperations**](AttachmentSingleOperations.md)> |  | [optional]
**likes** | Option<[**models::BlogPostSingleLikes**](BlogPostSingleLikes.md)> |  | [optional]
**versions** | Option<[**models::AttachmentSingleVersions**](AttachmentSingleVersions.md)> |  | [optional]
**is_favorited_by_current_user** | Option<**bool**> | Whether the blog post has been favorited by the current user. | [optional]
**_links** | Option<[**models::AbstractPageLinks**](AbstractPageLinks.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
