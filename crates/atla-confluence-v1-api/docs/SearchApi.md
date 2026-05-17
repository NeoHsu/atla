# \SearchApi

All URIs are relative to *http://your-domain.atlassian.net*

Method | HTTP request | Description
------------- | ------------- | -------------
[**search_by_cql**](SearchApi.md#search_by_cql) | **GET** /wiki/rest/api/search | Search content
[**search_user**](SearchApi.md#search_user) | **GET** /wiki/rest/api/search/user | Search users



## search_by_cql

> models::SearchPageResponseSearchResult search_by_cql(cql, cqlcontext, cursor, next, prev, limit, start, include_archived_spaces, exclude_current_spaces, excerpt, site_permission_type_filter, expand)
Search content

Searches for content using the [Confluence Query Language (CQL)](https://developer.atlassian.com/cloud/confluence/advanced-searching-using-cql/).  **Note that CQL input queries submitted through the `/wiki/rest/api/search` endpoint no longer support user-specific fields like `user`, `user.fullname`, `user.accountid`, and `user.userkey`.**  See this [deprecation notice](https://developer.atlassian.com/cloud/confluence/deprecation-notice-search-api/) for more details.  Example initial call: ``` /wiki/rest/api/search?cql=type=page&limit=25 ```  Example response: ``` {   \"results\": [     { ... },     { ... },     ...     { ... }   ],   \"limit\": 25,   \"size\": 25,   ...   \"_links\": {     \"base\": \"<url>\",     \"context\": \"<url>\",     \"next\": \"/rest/api/search?cql=type=page&limit=25&cursor=raNDoMsTRiNg\",     \"self\": \"<url>\"   } } ```  When additional results are available, returns `next` and `prev` URLs to retrieve them in subsequent calls. The URLs each contain a cursor that points to the appropriate set of results. Use `limit` to specify the number of results returned in each call.  Example subsequent call (taken from example response): ``` /wiki/rest/api/search?cql=type=page&limit=25&cursor=raNDoMsTRiNg ``` The response to this will have a `prev` URL similar to the `next` in the example response.  If the expand query parameter is used with the `body.export_view` and/or `body.styled_view` properties, then the query limit parameter will be restricted to a maximum value of 25.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to view the entities. Note, only entities that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**cql** | **String** | The CQL query to be used for the search. See [Advanced Searching using CQL](https://developer.atlassian.com/cloud/confluence/advanced-searching-using-cql/) for instructions on how to build a CQL query. | [required] |
**cqlcontext** | Option<**String**> | The space, content, and content status to execute the search against.  - `spaceKey` Key of the space to search against. Optional. - `contentId` ID of the content to search against. Optional. Must be in the space specified by `spaceKey`. - `contentStatuses` Content statuses to search against. Optional.  Specify these values in an object. For example, `cqlcontext={%22spaceKey%22:%22TEST%22, %22contentId%22:%22123%22}` |  |
**cursor** | Option<**String**> | Pointer to a set of search results, returned as part of the `next` or `prev` URL from the previous search call. |  |
**next** | Option<**bool**> |  |  |[default to false]
**prev** | Option<**bool**> |  |  |[default to false]
**limit** | Option<**i32**> | The maximum number of content objects to return per page. Note, this may be restricted by fixed system limits. |  |[default to 25]
**start** | Option<**i32**> | The start point of the collection to return |  |[default to 0]
**include_archived_spaces** | Option<**bool**> | Whether to include content from archived spaces in the results. |  |[default to false]
**exclude_current_spaces** | Option<**bool**> | Whether to exclude current spaces and only show archived spaces. |  |[default to false]
**excerpt** | Option<**String**> | The excerpt strategy to apply to the result |  |[default to highlight]
**site_permission_type_filter** | Option<**String**> | Filters users by permission type. Use `none` to default to licensed users, `externalCollaborator` for external/guest users, and `all` to include all permission types. |  |[default to none]
**expand** | Option<[**Vec<String>**](String.md)> |  |  |

### Return type

[**models::SearchPageResponseSearchResult**](SearchPageResponseSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## search_user

> models::SearchPageResponseSearchResult search_user(cql, start, limit, expand, site_permission_type_filter)
Search users

Searches for users using user-specific queries from the [Confluence Query Language (CQL)](https://developer.atlassian.com/cloud/confluence/advanced-searching-using-cql/).  Note that CQL input queries submitted through the `/wiki/rest/api/search/user` endpoint only support user-specific fields like `user`, `user.fullname`, `user.accountid`, and `user.userkey`.  Note that some user fields may be set to null depending on the user's privacy settings. These are: email, profilePicture, displayName, and timeZone.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**cql** | **String** | The CQL query to be used for the search. See [Advanced Searching using CQL](https://developer.atlassian.com/cloud/confluence/advanced-searching-using-cql/) for instructions on how to build a CQL query.  Example queries:           cql=type=user will return up to 10k users           cql=user=\"1234\" will return user with accountId \"1234\"           You can also use IN, NOT IN, != operators           cql=user IN (\"12\", \"34\") will return users with accountids \"12\" and \"34\"           cql=user.fullname~jo will return users with nickname/full name starting with \"jo\"           cql=user.accountid=\"123\" will return user with accountId \"123\" | [required] |
**start** | Option<**i32**> | The starting index of the returned users. |  |[default to 0]
**limit** | Option<**i32**> | The maximum number of user objects to return per page. Note, this may be restricted by fixed system limits. |  |[default to 25]
**expand** | Option<[**Vec<String>**](String.md)> | A multi-value parameter indicating which properties of the user to expand.  - `operations` returns the operations for the user, which are used when setting permissions. - `personalSpace` returns the personal space of the user. |  |
**site_permission_type_filter** | Option<**String**> | Filters users by permission type. Use `none` to default to licensed users, `externalCollaborator` for external/guest users, and `all` to include all permission types. |  |[default to none]

### Return type

[**models::SearchPageResponseSearchResult**](SearchPageResponseSearchResult.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
