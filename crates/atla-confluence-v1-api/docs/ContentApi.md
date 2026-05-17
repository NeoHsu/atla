# \ContentApi

All URIs are relative to *http://your-domain.atlassian.net*

Method | HTTP request | Description
------------- | ------------- | -------------
[**search_content_by_cql**](ContentApi.md#search_content_by_cql) | **GET** /wiki/rest/api/content/search | Search content by CQL



## search_content_by_cql

> models::ContentArray search_content_by_cql(cql, cqlcontext, expand, cursor, limit)
Search content by CQL

Returns the list of content that matches a Confluence Query Language (CQL) query. For information on CQL, see: [Advanced searching using CQL](https://developer.atlassian.com/cloud/confluence/advanced-searching-using-cql/).  Example initial call: ``` /wiki/rest/api/content/search?cql=type=page&limit=25 ```  Example response: ``` {   \"results\": [     { ... },     { ... },     ...     { ... }   ],   \"limit\": 25,   \"size\": 25,   ...   \"_links\": {     \"base\": \"<url>\",     \"context\": \"<url>\",     \"next\": \"/rest/api/content/search?cql=type=page&limit=25&cursor=raNDoMsTRiNg\",     \"self\": \"<url>\"   } } ```  When additional results are available, returns `next` and `prev` URLs to retrieve them in subsequent calls. The URLs each contain a cursor that points to the appropriate set of results. Use `limit` to specify the number of results returned in each call. Example subsequent call (taken from example response): ``` /wiki/rest/api/content/search?cql=type=page&limit=25&cursor=raNDoMsTRiNg ``` The response to this will have a `prev` URL similar to the `next` in the example response.  If the expand query parameter is used with the `body.export_view` and/or `body.styled_view` properties, then the query limit parameter will be restricted to a maximum value of 25.  **[Permissions](https://confluence.atlassian.com/x/_AozKw) required**: Permission to access the Confluence site ('Can use' global permission). Only content that the user has permission to view will be returned.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**cql** | **String** | The CQL string that is used to find the requested content. | [required] |
**cqlcontext** | Option<**String**> | The space, content, and content status to execute the search against. Specify this as an object with the following properties:  - `spaceKey` Key of the space to search against. Optional. - `contentId` ID of the content to search against. Optional. Must be in the space spacified by `spaceKey`. - `contentStatuses` Content statuses to search against. Optional. |  |
**expand** | Option<[**Vec<String>**](String.md)> | A multi-value parameter indicating which properties of the content to expand.  - `childTypes.all` returns whether the content has attachments, comments, or child pages/whiteboards. Use this if you only need to check whether the content has children of a particular type. - `childTypes.attachment` returns whether the content has attachments. - `childTypes.comment` returns whether the content has comments. - `childTypes.page` returns whether the content has child pages. - `childTypes.whiteboard` returns whether the content has child whiteboards. - `childTypes.database` returns whether the content has child databases. - `childTypes.embed` returns whether the content has child embeds (smartlinks). - `childTypes.folder` returns whether the content has child folders. - `container` returns the space that the content is in. This is the same as the information returned by [Get space](#api-space-spaceKey-get). - `metadata.currentuser` returns information about the current user in relation to the content, including when they last viewed it, modified it, contributed to it, or added it as a favorite. - `metadata.properties` returns content properties that have been set via the Confluence REST API. - `metadata.labels` returns the labels that have been added to the content. - `metadata.frontend` this property is only used by Atlassian. - `operations` returns the operations for the content, which are used when setting permissions. - `children.page` returns pages that are descendants at the level immediately below the content. - `children.whiteboard` returns whiteboards that are descendants at the level immediately below the content. - `children.database` returns databases that are descendants at the level immediately below the content. - `children.embed` returns embeds (smartlinks) that are descendants at the level immediately below the content. - `children.folder` returns folders that are descendants at the level immediately below the content. - `children.attachment` returns all attachments for the content. - `children.comment` returns all comments on the content. - `restrictions.read.restrictions.user` returns the users that have permission to read the content. - `restrictions.read.restrictions.group` returns the groups that have permission to read the content. Note that this may return deleted groups, because deleting a group doesn't remove associated restrictions. - `restrictions.update.restrictions.user` returns the users that have permission to update the content. - `restrictions.update.restrictions.group` returns the groups that have permission to update the content. Note that this may return deleted groups because deleting a group doesn't remove associated restrictions. - `history` returns the history of the content, including the date it was created. - `history.lastUpdated` returns information about the most recent update of the content, including who updated it and when it was updated. - `history.previousVersion` returns information about the update prior to the current content update. - `history.contributors` returns all of the users who have contributed to the content. - `history.nextVersion` returns information about the update after to the current content update. - `ancestors` returns the parent content, if the content is a page or whiteboard. - `body` returns the body of the content in different formats, including the editor format, view format, and export format. - `body.storage` returns the body of content in storage format. - `body.view` returns the body of content in view format. - `version` returns information about the most recent update of the content, including who updated it and when it was updated. - `descendants.page` returns pages that are descendants at any level below the content. - `descendants.whiteboard` returns whiteboards that are descendants at any level below the content. - `descendants.database` returns databases that are descendants at any level below the content. - `descendants.embed` returns embeds (smartlinks) that are descendants at any level below the content. - `descendants.folder` returns folders that are descendants at any level below the content. - `descendants.attachment` returns all attachments for the content, same as `children.attachment`. - `descendants.comment` returns all comments on the content, same as `children.comment`. - `space` returns the space that the content is in. This is the same as the information returned by [Get space](#api-space-spaceKey-get).  In addition, the following comment-specific expansions can be used: - `extensions.inlineProperties` returns inline comment-specific properties. - `extensions.resolution` returns the resolution status of each comment. |  |
**cursor** | Option<**String**> | Pointer to a set of search results, returned as part of the `next` or `prev` URL from the previous search call. |  |
**limit** | Option<**i32**> | The maximum number of content objects to return per page. Note, this may be restricted by fixed system limits. |  |[default to 25]

### Return type

[**models::ContentArray**](ContentArray.md)

### Authorization

[basicAuth](../README.md#basicAuth), [oAuthDefinitions](../README.md#oAuthDefinitions)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
