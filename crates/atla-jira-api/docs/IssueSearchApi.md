# \IssueSearchApi

All URIs are relative to *https://your-domain.atlassian.net*

Method | HTTP request | Description
------------- | ------------- | -------------
[**search_and_reconsile_issues_using_jql**](IssueSearchApi.md#search_and_reconsile_issues_using_jql) | **GET** /rest/api/3/search/jql | Search for issues using JQL enhanced search (GET)



## search_and_reconsile_issues_using_jql

> models::SearchAndReconcileResults search_and_reconsile_issues_using_jql(jql, next_page_token, max_results, fields)
Search for issues using JQL enhanced search (GET)

Searches for issues using [JQL](https://confluence.atlassian.com/x/egORLQ). Recent updates might not be immediately visible in the returned search results. If you need [read-after-write](https://developer.atlassian.com/cloud/jira/platform/search-and-reconcile/) consistency, you can utilize the `reconcileIssues` parameter to ensure stronger consistency assurances. This operation can be accessed anonymously.  If the JQL query expression is too large to be encoded as a query parameter, use the [POST](#api-rest-api-3-search-post) version of this resource.  **[Permissions](#permissions) required:** Issues are included in the response where the user has:   *  *Browse projects* [project permission](https://confluence.atlassian.com/x/yodKLg) for the project containing the issue.  *  If [issue-level security](https://confluence.atlassian.com/x/J4lKLg) is configured, issue-level security permission to view the issue.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**jql** | Option<**String**> |  |  |
**next_page_token** | Option<**String**> |  |  |
**max_results** | Option<**i32**> |  |  |
**fields** | Option<[**Vec<String>**](String.md)> |  |  |

### Return type

[**models::SearchAndReconcileResults**](SearchAndReconcileResults.md)

### Authorization

[OAuth2](../README.md#OAuth2), [basicAuth](../README.md#basicAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
