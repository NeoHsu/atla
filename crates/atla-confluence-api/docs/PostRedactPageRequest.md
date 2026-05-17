# PostRedactPageRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **chrono::DateTime<chrono::FixedOffset>** | Timestamp when the content was last updated. |
**clean_history** | Option<**bool**> | Whether to clean up previous versions containing the redaction. When true, historical versions of the content that contain the redacted text will be squashed. | [optional]
**version_number** | Option<**i32**> | Optional version number of the content to redact. When specified, the redaction will target  a specific historical version of the content rather than the current version.  - If omitted or null, the redaction applies to the current (latest) version of the content. - When provided, must be a valid version number that exists for the content.  **Note**: Version numbers start at 1 and increment with each content update.  | [optional]
**body** | Option<[**models::PostRedactPageRequestBody**](PostRedactPageRequestBody.md)> |  | [optional]
**title** | Option<[**models::PostRedactPageRequestBody**](PostRedactPageRequestBody.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
