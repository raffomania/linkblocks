use reqwest::Client;
use serde_json::json;

pub struct OmnivoreImport {
    client: Client,
    graphql_endpoint_url: String,
}

impl OmnivoreImport {
    pub fn new(api_token: String, graphql_endpoint_url: String) -> Self {
        let client = Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert("Content-Type", "application/json".parse().unwrap());
                headers.insert("Authorization", api_token.parse().unwrap());
                headers
            })
            .build()
            .unwrap();

        Self {
            client,
            graphql_endpoint_url,
        }
    }

    pub async fn get_articles(
        &self,
        limit: Option<i32>,
        cursor: Option<String>,
        format: String,
        query: String,
        include_content: bool,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let request_body = json!({
            "query": "
            query Search($after: String, $first: Int, $query: String, $format: String, $includeContent: Boolean) {
                search(after: $after, first: $first, query: $query, format: $format, includeContent: $includeContent) {
                    ... on SearchSuccess {
                        edges {
                            cursor
                            node {
                                id
                                title
                                slug
                                url
                                pageType
                                contentReader
                                createdAt
                                isArchived
                                readingProgressPercent
                                readingProgressTopPercent
                                readingProgressAnchorIndex
                                author
                                image
                                description
                                publishedAt
                                ownedByViewer
                                originalArticleUrl
                                uploadFileId
                                labels {
                                    id
                                    name
                                    color
                                }
                                pageId
                                shortId
                                quote
                                annotation
                                state
                                siteName
                                subscription
                                readAt
                                savedAt
                                wordsCount
                                recommendations {
                                    id
                                    name
                                    note
                                    user {
                                        userId
                                        name
                                        username
                                        profileImageURL
                                    }
                                    recommendedAt
                                }
                                highlights {
                                    ...HighlightFields
                                }
                            }
                        }
                        pageInfo {
                            hasNextPage
                            hasPreviousPage
                            startCursor
                            endCursor
                            totalCount
                        }
                    }
                    ... on SearchError {
                        errorCodes
                    }
                }
            }
            
            fragment HighlightFields on Highlight {
                id
                type
                shortId
                quote
                prefix
                suffix
                patch
                annotation
                createdByMe
                createdAt
                updatedAt
                sharedAt
                highlightPositionPercent
                highlightPositionAnchorIndex
                labels {
                    id
                    name
                    color
                    createdAt
                }
            }
            ",
            "variables": {
                "first": limit,
                "after": cursor,
                "query": query,
                "format": format,
                "includeContent": include_content,
            }
        });

        let response = self
            .client
            .post(&self.graphql_endpoint_url)
            .json(&request_body)
            .send()
            .await?;

        let response_body: serde_json::Value = response.json().await?;

        if let Some(errors) = response_body.get("errors") {
            return Err(format!("GraphQL Error: {:?}", errors).into());
        }

        Ok(response_body)
    }
}
