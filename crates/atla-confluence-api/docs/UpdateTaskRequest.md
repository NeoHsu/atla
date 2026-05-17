# UpdateTaskRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | Option<**String**> | ID of the task. | [optional]
**local_id** | Option<**String**> | Local ID of the task. This ID is local to the corresponding page or blog post. | [optional]
**space_id** | Option<**String**> | ID of the space the task is in. | [optional]
**page_id** | Option<**String**> | ID of the page the task is in. | [optional]
**blog_post_id** | Option<**String**> | ID of the blog post the task is in. | [optional]
**status** | **Status** | Status of the task. (enum: complete, incomplete) |
**created_by** | Option<**String**> | Account ID of the user who created this task. | [optional]
**assigned_to** | Option<**String**> | Account ID of the user to whom this task is assigned. | [optional]
**completed_by** | Option<**String**> | Account ID of the user who completed this task. | [optional]
**created_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Date and time when the task was created. In format \"YYYY-MM-DDTHH:mm:ss.sssZ\". | [optional]
**updated_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Date and time when the task was updated. In format \"YYYY-MM-DDTHH:mm:ss.sssZ\". | [optional]
**due_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Date and time when the task is due. In format \"YYYY-MM-DDTHH:mm:ss.sssZ\". | [optional]
**completed_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Date and time when the task was completed. In format \"YYYY-MM-DDTHH:mm:ss.sssZ\". | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
