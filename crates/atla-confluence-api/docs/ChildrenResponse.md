# ChildrenResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | ID of the child content. | [optional]
**status** | Option<[**models::OnlyArchivedAndCurrentContentStatus**](OnlyArchivedAndCurrentContentStatus.md)> |  | [optional]
**title** | Option<**String**> | Title of the child content. | [optional]
**r#type** | Option<**String**> | Hierarchical content type (database/embed/folder/page/whiteboard). | [optional]
**space_id** | Option<**String**> | ID of the space the content is in. | [optional]
**child_position** | Option<**i32**> | Numerical value indicating position of the content relative to its siblings (with the same parentId) within the content tree. If the content is sorted by childPosition, it will reflect the default content ordering within the content tree. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
