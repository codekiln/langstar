use crate::client::LangchainClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Visibility filter for prompts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Only public prompts
    Public,
    /// Only private prompts
    Private,
    /// All prompts (public and private)
    Any,
}

/// A prompt from the LangSmith Prompt Hub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// Unique identifier for the prompt
    pub id: String,
    /// Name of the prompt
    pub repo_handle: String,
    /// Description of the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Number of likes
    #[serde(default)]
    pub num_likes: u32,
    /// Number of downloads
    #[serde(default)]
    pub num_downloads: u32,
    /// Prompt content/template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest: Option<serde_json::Value>,
    /// When the prompt was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// When the prompt was last updated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// Is this prompt public
    #[serde(default)]
    pub is_public: bool,
}

/// Client for interacting with LangSmith Prompts API
pub struct PromptClient<'a> {
    client: &'a LangchainClient,
}

impl<'a> PromptClient<'a> {
    /// Create a new PromptClient
    pub fn new(client: &'a LangchainClient) -> Self {
        Self { client }
    }

    /// List all prompts
    ///
    /// # Arguments
    /// * `limit` - Maximum number of prompts to return (default: 20)
    /// * `offset` - Number of prompts to skip (default: 0)
    /// * `visibility` - Filter by visibility (Public, Private, or Any). Defaults to Any.
    pub async fn list(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        visibility: Option<Visibility>,
    ) -> Result<Vec<Prompt>> {
        let limit = limit.unwrap_or(20);
        let offset = offset.unwrap_or(0);
        let visibility = visibility.unwrap_or(Visibility::Any);

        let path = format!("/api/v1/repos/?limit={}&offset={}", limit, offset);
        let request = self.client.langsmith_get(&path)?;

        // LangSmith API returns a paginated response with a "repos" field
        #[derive(Deserialize)]
        struct ListReposResponse {
            repos: Vec<Prompt>,
        }

        let response: ListReposResponse = self.client.execute(request).await?;

        // Filter by visibility if specified
        let filtered = match visibility {
            Visibility::Public => response.repos.into_iter().filter(|p| p.is_public).collect(),
            Visibility::Private => response
                .repos
                .into_iter()
                .filter(|p| !p.is_public)
                .collect(),
            Visibility::Any => response.repos,
        };

        Ok(filtered)
    }

    /// Get a specific prompt by handle
    ///
    /// # Arguments
    /// * `handle` - The prompt handle (e.g., "owner/prompt-name")
    pub async fn get(&self, handle: &str) -> Result<Prompt> {
        let path = format!("/api/v1/repos/{}", handle);
        let request = self.client.langsmith_get(&path)?;

        // The API wraps the prompt in a "repo" field
        #[derive(Deserialize)]
        struct PromptResponse {
            repo: Prompt,
        }

        let response: PromptResponse = self.client.execute(request).await?;
        Ok(response.repo)
    }

    /// Search for prompts
    ///
    /// # Arguments
    /// * `query` - Search query string
    /// * `limit` - Maximum number of results (default: 20)
    /// * `visibility` - Filter by visibility (Public, Private, or Any). Defaults to Any.
    pub async fn search(
        &self,
        query: &str,
        limit: Option<u32>,
        visibility: Option<Visibility>,
    ) -> Result<Vec<Prompt>> {
        let limit = limit.unwrap_or(20);
        let visibility = visibility.unwrap_or(Visibility::Any);

        let path = format!("/api/v1/repos/?query={}&limit={}", query, limit);
        let request = self.client.langsmith_get(&path)?;

        // LangSmith API returns a paginated response with a "repos" field (same as list)
        #[derive(Deserialize)]
        struct SearchReposResponse {
            repos: Vec<Prompt>,
        }

        let response: SearchReposResponse = self.client.execute(request).await?;

        // Filter by visibility if specified
        let filtered = match visibility {
            Visibility::Public => response.repos.into_iter().filter(|p| p.is_public).collect(),
            Visibility::Private => response
                .repos
                .into_iter()
                .filter(|p| !p.is_public)
                .collect(),
            Visibility::Any => response.repos,
        };

        Ok(filtered)
    }

    /// Create a new prompt repository
    ///
    /// # Arguments
    /// * `repo_handle` - The handle for the repository (e.g., "owner/repo-name")
    /// * `description` - Optional description
    /// * `readme` - Optional readme content
    /// * `is_public` - Whether the repository is public (default: false)
    /// * `tags` - Optional tags
    pub async fn create_repo(
        &self,
        repo_handle: &str,
        description: Option<String>,
        readme: Option<String>,
        is_public: bool,
        tags: Option<Vec<String>>,
    ) -> Result<Prompt> {
        let path = "/api/v1/repos";

        #[derive(Serialize)]
        struct CreateRepoRequest {
            repo_handle: String,
            description: Option<String>,
            readme: Option<String>,
            is_public: bool,
            tags: Option<Vec<String>>,
        }

        let request_body = CreateRepoRequest {
            repo_handle: repo_handle.to_string(),
            description,
            readme,
            is_public,
            tags,
        };

        let request = self.client.langsmith_post(path)?.json(&request_body);

        #[derive(Deserialize)]
        struct CreateRepoResponse {
            repo: Prompt,
        }

        let response: CreateRepoResponse = self.client.execute(request).await?;
        Ok(response.repo)
    }

    /// Create or update a prompt in the PromptHub
    ///
    /// This creates a new commit for the prompt. The correct endpoint is
    /// `/api/v1/commits/{owner}/{repo}` not `/api/v1/repos/{owner}/{repo}`.
    ///
    /// # Arguments
    /// * `owner` - The owner of the prompt (username or organization)
    /// * `repo` - The prompt repository name
    /// * `commit_request` - The commit data to push
    pub async fn push(
        &self,
        owner: &str,
        repo: &str,
        commit_request: &CommitRequest,
    ) -> Result<CommitResponse> {
        let path = format!("/api/v1/commits/{}/{}", owner, repo);
        // Use POST to create a new commit
        let request = self.client.langsmith_post(&path)?.json(commit_request);
        let response: CommitResponse = self.client.execute(request).await?;
        Ok(response)
    }
}

/// Request to create a commit (upload/update a prompt)
///
/// Corresponds to the LangSmith API CreateRepoCommitRequest schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRequest {
    /// The prompt manifest/template (required)
    pub manifest: serde_json::Value,
    /// Parent commit hash (optional, for updates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_commit: Option<String>,
    /// Example run IDs to associate with this commit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example_run_ids: Option<Vec<String>>,
}

/// Response from creating a commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitResponse {
    /// The commit data
    pub commit: CommitData,
}

/// Commit data within the response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitData {
    /// Commit hash
    pub commit_hash: String,
    /// URL to the commit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Data for creating/updating a prompt (deprecated, use CommitRequest)
///
/// This type is kept for backward compatibility but CommitRequest should be
/// used for new code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(since = "0.1.0", note = "Use CommitRequest instead")]
pub struct PromptData {
    /// Description of the prompt
    pub description: Option<String>,
    /// Prompt readme/documentation
    pub readme: Option<String>,
    /// Tags for the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Is this prompt public
    #[serde(default)]
    pub is_public: bool,
    /// The prompt manifest/template
    pub manifest: serde_json::Value,
}

impl LangchainClient {
    /// Get a PromptClient for interacting with prompts
    pub fn prompts(&self) -> PromptClient<'_> {
        PromptClient::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthConfig;

    #[test]
    fn test_prompt_client_creation() {
        let auth = AuthConfig::new(Some("test".to_string()), None, None, None);
        let client = LangchainClient::new(auth).unwrap();
        let _prompt_client = client.prompts();
    }

    #[test]
    fn test_prompt_serialization() {
        let prompt = Prompt {
            id: "test-id".to_string(),
            repo_handle: "owner/prompt".to_string(),
            description: Some("Test prompt".to_string()),
            num_likes: 42,
            num_downloads: 100,
            manifest: None,
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
            updated_at: Some("2024-01-01T00:00:00Z".to_string()),
            is_public: true,
        };

        let json = serde_json::to_string(&prompt).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("owner/prompt"));
    }
}
