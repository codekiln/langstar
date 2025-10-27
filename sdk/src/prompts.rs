use crate::client::{LangchainClient, ListResponse};
use crate::error::Result;
use serde::{Deserialize, Serialize};

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
    pub async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Prompt>> {
        let limit = limit.unwrap_or(20);
        let offset = offset.unwrap_or(0);

        let path = format!("/api/v1/repos/?limit={}&offset={}", limit, offset);
        let request = self.client.langsmith_get(&path)?;

        // LangSmith API returns a paginated response with a "repos" field
        #[derive(Deserialize)]
        struct ListReposResponse {
            repos: Vec<Prompt>,
        }

        let response: ListReposResponse = self.client.execute(request).await?;
        Ok(response.repos)
    }

    /// Get a specific prompt by handle
    ///
    /// # Arguments
    /// * `handle` - The prompt handle (e.g., "owner/prompt-name")
    pub async fn get(&self, handle: &str) -> Result<Prompt> {
        let path = format!("/api/v1/repos/{}", handle);
        let request = self.client.langsmith_get(&path)?;
        let prompt: Prompt = self.client.execute(request).await?;
        Ok(prompt)
    }

    /// Search for prompts
    ///
    /// # Arguments
    /// * `query` - Search query string
    /// * `limit` - Maximum number of results (default: 20)
    pub async fn search(&self, query: &str, limit: Option<u32>) -> Result<Vec<Prompt>> {
        let limit = limit.unwrap_or(20);
        let path = format!("/api/v1/repos/?query={}&limit={}", query, limit);
        let request = self.client.langsmith_get(&path)?;
        let response: Vec<Prompt> = self.client.execute(request).await?;
        Ok(response)
    }

    /// Create or update a prompt in the PromptHub
    ///
    /// # Arguments
    /// * `repo_handle` - The handle for the prompt (e.g., "owner/prompt-name")
    /// * `prompt_data` - The prompt data to push
    pub async fn push(&self, repo_handle: &str, prompt_data: &PromptData) -> Result<Prompt> {
        let path = format!("/api/v1/repos/{}", repo_handle);
        // Use PUT for creating/updating prompts
        let request = self.client.langsmith_put(&path)?
            .json(prompt_data);
        let prompt: Prompt = self.client.execute(request).await?;
        Ok(prompt)
    }
}

/// Data for creating/updating a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn prompts(&self) -> PromptClient {
        PromptClient::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthConfig;

    #[test]
    fn test_prompt_client_creation() {
        let auth = AuthConfig::new(Some("test".to_string()), None);
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
