# RedactionPointerResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**pointer** | Option<**String**> | JSON pointer indicating where the redaction was applied | [optional]
**from** | Option<**i32**> | Starting character index where redaction was applied | [optional]
**to** | Option<**i32**> | Ending character index where redaction was applied | [optional]
**reason** | Option<**String**> | Reason for the redaction | [optional]
**redaction_id** | Option<**uuid::Uuid**> | Unique identifier for this redaction. Can be used to restore the redacted content later.  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
