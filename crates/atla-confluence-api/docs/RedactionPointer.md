# RedactionPointer

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**pointer** | **String** | JSON pointer indicating the exact location within the content structure  where redaction should be applied. Points to the text node containing the content to redact.  |
**from** | Option<**i32**> | Starting character index (zero-based) within the target text where redaction begins.  | [optional]
**to** | Option<**i32**> | Ending character index (zero-based) within the target text where redaction ends (exclusive). Must be greater than or equal to 'from' value.  | [optional]
**reason** | Option<**String**> | Optional human-readable reason for the redaction. Used for audit trails and compliance documentation.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
