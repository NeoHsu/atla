# CreateInlineCommentModelInlineCommentProperties

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**text_selection** | Option<**String**> | The text to highlight | [optional]
**text_selection_match_count** | Option<**i32**> | The number of matches for the selected text on the page (should be strictly greater than textSelectionMatchIndex) | [optional]
**text_selection_match_index** | Option<**i32**> | The match index to highlight. This is zero-based. E.g. if you have 3 occurrences of \"hello world\" on a page  and you want to highlight the second occurrence, you should pass 1 for textSelectionMatchIndex and 3 for textSelectionMatchCount. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
