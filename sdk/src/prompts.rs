use crate::client::LangchainClient;
use crate::error::{LangstarError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

// ═══════════════════════════════════════════════════════════════════════
// LC-JSON Serialization Types
// ═══════════════════════════════════════════════════════════════════════

/// LC-JSON wrapper for LangChain object serialization.
///
/// LangChain uses a custom JSON format ("LC-JSON") to serialize objects.
/// This format includes metadata about the object's class and module path.
///
/// # Format
///
/// ```json
/// {
///   "lc": 1,
///   "type": "constructor",
///   "id": ["module", "path", "ClassName"],
///   "kwargs": { ... },
///   "name": "ClassName"
/// }
/// ```
///
/// # Reference
///
/// See `docs/research/398-structured-output-prompts-scout.md` Section 9.2
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LcJson<T> {
    /// LangChain serialization version (always 1)
    pub lc: u8,
    /// Serialization type (always "constructor" for class instances)
    #[serde(rename = "type")]
    pub type_: String,
    /// Module path to the class (e.g., ["langchain_core", "prompts", "structured", "StructuredPrompt"])
    pub id: Vec<String>,
    /// Constructor arguments
    pub kwargs: T,
    /// Class name (redundant with id[-1], present for clarity)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl<T> LcJson<T> {
    /// Create a new LC-JSON wrapper
    pub fn new(id: Vec<String>, kwargs: T) -> Self {
        let name = id.last().cloned();
        Self {
            lc: 1,
            type_: "constructor".to_string(),
            id,
            kwargs,
            name,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Structured Prompt Types
// ═══════════════════════════════════════════════════════════════════════

/// Structured output prompt with JSON schema constraints.
///
/// Combines a prompt template with a JSON schema to constrain LLM outputs.
/// When used with an LLM, the schema ensures the output matches the defined structure.
///
/// # Format
///
/// Serializes to LC-JSON format matching Python's `StructuredPrompt` class.
///
/// # Example
///
/// ```rust
/// use langstar_sdk::prompts::{StructuredPrompt, StructuredOutputKwargs};
/// use serde_json::json;
///
/// let schema = json!({
///     "type": "object",
///     "properties": {
///         "title": {"type": "string"},
///         "rating": {"type": "integer", "minimum": 1, "maximum": 10}
///     },
///     "required": ["title", "rating"]
/// });
///
/// let prompt = StructuredPrompt {
///     input_variables: Some(vec!["movie_name".to_string()]),
///     messages: vec![/* message templates */],
///     schema_: schema,
///     structured_output_kwargs: StructuredOutputKwargs {
///         method: "json_schema".to_string(),
///     },
/// };
/// ```
///
/// # Reference
///
/// - Research: `docs/research/398-structured-output-prompts-scout.md`
/// - Python class: `langchain_core.prompts.structured.StructuredPrompt`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredPrompt {
    /// Input variables extracted from template placeholders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_variables: Option<Vec<String>>,
    /// Message templates (system, human, etc.)
    pub messages: Vec<LcJson<MessagePromptTemplateKwargs>>,
    /// JSON Schema defining the output structure
    pub schema_: Value,
    /// Structured output configuration (method selection)
    pub structured_output_kwargs: StructuredOutputKwargs,
}

impl StructuredPrompt {
    /// Wrap this StructuredPrompt in LC-JSON format for API submission
    pub fn to_lc_json(self) -> LcJson<Self> {
        LcJson::new(
            vec![
                "langchain_core".to_string(),
                "prompts".to_string(),
                "structured".to_string(),
                "StructuredPrompt".to_string(),
            ],
            self,
        )
    }
}

/// Structured output configuration.
///
/// Specifies how the JSON schema should be applied to the LLM output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredOutputKwargs {
    /// Method for applying schema ("json_schema" or "function_calling")
    pub method: String,
}

/// Kwargs for message prompt templates.
///
/// Contains the inner prompt template for a message (system, human, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessagePromptTemplateKwargs {
    /// The prompt template
    pub prompt: LcJson<PromptTemplateKwargs>,
}

/// Kwargs for the base PromptTemplate.
///
/// Contains the template string and configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptTemplateKwargs {
    /// Input variables (placeholders in the template)
    pub input_variables: Vec<String>,
    /// Template string (e.g., "Review the movie: {movie_name}")
    pub template: String,
    /// Template format (e.g., "f-string")
    pub template_format: String,
}

// ═══════════════════════════════════════════════════════════════════════
// Schema Validation
// ═══════════════════════════════════════════════════════════════════════

/// Validate a JSON Schema value.
///
/// This checks if the provided value is a valid JSON Schema by attempting
/// to compile it using the jsonschema crate.
///
/// # Arguments
/// * `schema` - The JSON Schema to validate
///
/// # Returns
/// * `Ok(())` if the schema is valid
/// * `Err(InvalidSchemaError)` if the schema is invalid
///
/// # Example
/// ```rust
/// use langstar_sdk::prompts::validate_json_schema;
/// use serde_json::json;
///
/// let valid_schema = json!({
///     "type": "object",
///     "properties": {
///         "name": {"type": "string"}
///     }
/// });
///
/// assert!(validate_json_schema(&valid_schema).is_ok());
/// ```
pub fn validate_json_schema(schema: &Value) -> Result<()> {
    // Attempt to compile the schema
    jsonschema::JSONSchema::compile(schema).map_err(|e| {
        LangstarError::InvalidSchemaError(format!("Schema validation failed: {}", e))
    })?;

    Ok(())
}

/// Validate a structured output method.
///
/// Valid methods are "json_schema" and "function_calling".
///
/// # Arguments
/// * `method` - The method string to validate
///
/// # Returns
/// * `Ok(())` if the method is valid
/// * `Err(InvalidMethodError)` if the method is invalid
///
/// # Example
/// ```rust
/// use langstar_sdk::prompts::validate_method;
///
/// assert!(validate_method("json_schema").is_ok());
/// assert!(validate_method("function_calling").is_ok());
/// assert!(validate_method("invalid").is_err());
/// ```
pub fn validate_method(method: &str) -> Result<()> {
    match method {
        "json_schema" | "function_calling" => Ok(()),
        _ => Err(LangstarError::InvalidMethodError(method.to_string())),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Prompt Repository Types
// ═══════════════════════════════════════════════════════════════════════

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

    /// Push a structured prompt to the PromptHub with schema validation.
    ///
    /// This method validates the JSON schema before pushing and serializes
    /// the StructuredPrompt to LC-JSON format.
    ///
    /// # Arguments
    /// * `owner` - The owner of the prompt (username or organization)
    /// * `repo` - The prompt repository name
    /// * `structured_prompt` - The structured prompt to push
    /// * `parent_commit` - Optional parent commit hash for updates
    ///
    /// # Returns
    /// The commit response with commit hash
    ///
    /// # Errors
    /// - `InvalidSchemaError` if the JSON schema is invalid
    /// - `InvalidMethodError` if the method is not "json_schema" or "function_calling"
    /// - API errors if the push fails
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, prompts::{StructuredPrompt, StructuredOutputKwargs}};
    /// # use serde_json::json;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    /// let prompts = client.prompts();
    ///
    /// let schema = json!({
    ///     "type": "object",
    ///     "properties": {
    ///         "answer": {"type": "string"}
    ///     },
    ///     "required": ["answer"]
    /// });
    ///
    /// let structured_prompt = StructuredPrompt {
    ///     input_variables: None,
    ///     messages: vec![/* ... */],
    ///     schema_: schema,
    ///     structured_output_kwargs: StructuredOutputKwargs {
    ///         method: "json_schema".to_string(),
    ///     },
    /// };
    ///
    /// let response = prompts.push_structured_prompt(
    ///     "owner",
    ///     "my-prompt",
    ///     structured_prompt,
    ///     None
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn push_structured_prompt(
        &self,
        owner: &str,
        repo: &str,
        structured_prompt: StructuredPrompt,
        parent_commit: Option<String>,
    ) -> Result<CommitResponse> {
        // Validate the schema before pushing
        validate_json_schema(&structured_prompt.schema_)?;

        // Validate the method
        validate_method(&structured_prompt.structured_output_kwargs.method)?;

        // Wrap in LC-JSON format
        let lc_json = structured_prompt.to_lc_json();

        // Serialize to serde_json::Value
        let manifest = serde_json::to_value(&lc_json)?;

        // Create commit request
        let commit_request = CommitRequest {
            manifest,
            parent_commit,
            example_run_ids: None,
        };

        // Push using existing method
        self.push(owner, repo, &commit_request).await
    }

    /// Pull a prompt commit from the PromptHub.
    ///
    /// This retrieves a specific commit from the prompt repository.
    ///
    /// # Arguments
    /// * `owner` - The owner of the prompt (username or organization)
    /// * `repo` - The prompt repository name
    /// * `commit` - The commit hash or tag (e.g., "latest", "main", commit SHA)
    ///
    /// # Returns
    /// The commit manifest as a JSON value
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    /// let prompts = client.prompts();
    ///
    /// let manifest = prompts.pull("owner", "my-prompt", "latest").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn pull(&self, owner: &str, repo: &str, commit: &str) -> Result<Value> {
        let path = format!("/api/v1/commits/{}/{}/{}", owner, repo, commit);
        let request = self.client.langsmith_get(&path)?;

        #[derive(Deserialize)]
        struct CommitManifestResponse {
            manifest: Value,
        }

        let response: CommitManifestResponse = self.client.execute(request).await?;
        Ok(response.manifest)
    }

    /// Pull a structured prompt from the PromptHub and deserialize it.
    ///
    /// This retrieves a commit and attempts to deserialize it as a StructuredPrompt.
    ///
    /// # Arguments
    /// * `owner` - The owner of the prompt (username or organization)
    /// * `repo` - The prompt repository name
    /// * `commit` - The commit hash or tag (e.g., "latest", "main", commit SHA)
    ///
    /// # Returns
    /// The deserialized StructuredPrompt
    ///
    /// # Errors
    /// - `JsonError` if the manifest cannot be deserialized as a StructuredPrompt
    /// - API errors if the pull fails
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    /// let prompts = client.prompts();
    ///
    /// let structured_prompt = prompts.pull_structured_prompt(
    ///     "owner",
    ///     "my-prompt",
    ///     "latest"
    /// ).await?;
    ///
    /// println!("Schema: {}", structured_prompt.schema_);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn pull_structured_prompt(
        &self,
        owner: &str,
        repo: &str,
        commit: &str,
    ) -> Result<StructuredPrompt> {
        let manifest = self.pull(owner, repo, commit).await?;

        // Deserialize from LC-JSON format
        let lc_json: LcJson<StructuredPrompt> = serde_json::from_value(manifest)?;

        Ok(lc_json.kwargs)
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
    use serde_json::json;

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

    // ═══════════════════════════════════════════════════════════════════════
    // LC-JSON and StructuredPrompt Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_lc_json_basic_serialization() {
        let kwargs = json!({"key": "value"});
        let lc_json: LcJson<Value> =
            LcJson::new(vec!["langchain".to_string(), "test".to_string()], kwargs);

        let serialized = serde_json::to_value(&lc_json).unwrap();

        assert_eq!(serialized["lc"], 1);
        assert_eq!(serialized["type"], "constructor");
        assert_eq!(serialized["id"], json!(["langchain", "test"]));
        assert_eq!(serialized["kwargs"], json!({"key": "value"}));
        assert_eq!(serialized["name"], "test");
    }

    #[test]
    fn test_lc_json_round_trip() {
        let original: LcJson<Value> = LcJson::new(
            vec!["module".to_string(), "Class".to_string()],
            json!({"param": "value"}),
        );

        let json_str = serde_json::to_string(&original).unwrap();
        let deserialized: LcJson<Value> = serde_json::from_str(&json_str).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_prompt_template_kwargs_serialization() {
        let kwargs = PromptTemplateKwargs {
            input_variables: vec!["movie_name".to_string()],
            template: "Review the movie: {movie_name}".to_string(),
            template_format: "f-string".to_string(),
        };

        let serialized = serde_json::to_value(&kwargs).unwrap();

        assert_eq!(serialized["input_variables"], json!(["movie_name"]));
        assert_eq!(serialized["template"], "Review the movie: {movie_name}");
        assert_eq!(serialized["template_format"], "f-string");
    }

    #[test]
    fn test_structured_prompt_minimal() {
        let schema = json!({
            "type": "object",
            "properties": {
                "answer": {"type": "string"}
            },
            "required": ["answer"]
        });

        let system_prompt = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "prompt".to_string(),
                "PromptTemplate".to_string(),
            ],
            PromptTemplateKwargs {
                input_variables: vec![],
                template: "You are a helpful assistant.".to_string(),
                template_format: "f-string".to_string(),
            },
        );

        let system_message = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "chat".to_string(),
                "SystemMessagePromptTemplate".to_string(),
            ],
            MessagePromptTemplateKwargs {
                prompt: system_prompt,
            },
        );

        let structured_prompt = StructuredPrompt {
            input_variables: None,
            messages: vec![system_message],
            schema_: schema.clone(),
            structured_output_kwargs: StructuredOutputKwargs {
                method: "json_schema".to_string(),
            },
        };

        let serialized = serde_json::to_value(&structured_prompt).unwrap();

        assert_eq!(serialized["schema_"], schema);
        assert_eq!(
            serialized["structured_output_kwargs"]["method"],
            "json_schema"
        );
        assert!(serialized["messages"].is_array());
    }

    #[test]
    fn test_structured_prompt_with_lc_json_wrapper() {
        let schema = json!({
            "type": "object",
            "title": "MovieReview",
            "properties": {
                "title": {"type": "string"},
                "rating": {"type": "integer", "minimum": 1, "maximum": 10}
            },
            "required": ["title", "rating"]
        });

        let human_prompt_kwargs = PromptTemplateKwargs {
            input_variables: vec!["movie_name".to_string()],
            template: "Review: {movie_name}".to_string(),
            template_format: "f-string".to_string(),
        };

        let human_prompt = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "prompt".to_string(),
                "PromptTemplate".to_string(),
            ],
            human_prompt_kwargs,
        );

        let human_message = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "chat".to_string(),
                "HumanMessagePromptTemplate".to_string(),
            ],
            MessagePromptTemplateKwargs {
                prompt: human_prompt,
            },
        );

        let structured_prompt = StructuredPrompt {
            input_variables: Some(vec!["movie_name".to_string()]),
            messages: vec![human_message],
            schema_: schema.clone(),
            structured_output_kwargs: StructuredOutputKwargs {
                method: "json_schema".to_string(),
            },
        };

        let wrapped = structured_prompt.to_lc_json();

        let serialized = serde_json::to_value(&wrapped).unwrap();

        assert_eq!(serialized["lc"], 1);
        assert_eq!(serialized["type"], "constructor");
        assert_eq!(
            serialized["id"],
            json!([
                "langchain_core",
                "prompts",
                "structured",
                "StructuredPrompt"
            ])
        );
        assert_eq!(serialized["name"], "StructuredPrompt");
        assert_eq!(serialized["kwargs"]["schema_"], schema);
    }

    #[test]
    fn test_structured_prompt_full_round_trip() {
        // Create a complete structured prompt matching the format from the research report
        let schema = json!({
            "description": "A structured movie review.",
            "properties": {
                "title": {
                    "description": "The movie title",
                    "title": "Title",
                    "type": "string"
                },
                "rating": {
                    "description": "Rating from 1-10",
                    "maximum": 10,
                    "minimum": 1,
                    "title": "Rating",
                    "type": "integer"
                },
                "summary": {
                    "description": "Brief summary",
                    "title": "Summary",
                    "type": "string"
                }
            },
            "required": ["title", "rating", "summary"],
            "title": "MovieReview",
            "type": "object"
        });

        // System message
        let system_prompt_kwargs = PromptTemplateKwargs {
            input_variables: vec![],
            template: "You are a movie critic. Provide a structured review.".to_string(),
            template_format: "f-string".to_string(),
        };

        let system_prompt = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "prompt".to_string(),
                "PromptTemplate".to_string(),
            ],
            system_prompt_kwargs,
        );

        let system_message = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "chat".to_string(),
                "SystemMessagePromptTemplate".to_string(),
            ],
            MessagePromptTemplateKwargs {
                prompt: system_prompt,
            },
        );

        // Human message
        let human_prompt_kwargs = PromptTemplateKwargs {
            input_variables: vec!["movie_name".to_string()],
            template: "Review the movie: {movie_name}".to_string(),
            template_format: "f-string".to_string(),
        };

        let human_prompt = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "prompt".to_string(),
                "PromptTemplate".to_string(),
            ],
            human_prompt_kwargs,
        );

        let human_message = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "chat".to_string(),
                "HumanMessagePromptTemplate".to_string(),
            ],
            MessagePromptTemplateKwargs {
                prompt: human_prompt,
            },
        );

        // Structured prompt
        let structured_prompt = StructuredPrompt {
            input_variables: Some(vec!["movie_name".to_string()]),
            messages: vec![system_message, human_message],
            schema_: schema.clone(),
            structured_output_kwargs: StructuredOutputKwargs {
                method: "json_schema".to_string(),
            },
        };

        let wrapped = structured_prompt.to_lc_json();

        // Serialize to JSON
        let json_str = serde_json::to_string_pretty(&wrapped).unwrap();

        // Deserialize back
        let deserialized: LcJson<StructuredPrompt> = serde_json::from_str(&json_str).unwrap();

        // Verify round-trip integrity
        assert_eq!(wrapped, deserialized);
        assert_eq!(deserialized.kwargs.schema_, schema);
        assert_eq!(
            deserialized.kwargs.structured_output_kwargs.method,
            "json_schema"
        );
        assert_eq!(deserialized.kwargs.messages.len(), 2);
    }

    #[test]
    fn test_structured_prompt_matches_python_format() {
        // This test verifies the serialized format matches the Python SDK output
        // from docs/research/398-structured-output-prompts-scout.md Section 9.2
        let schema = json!({
            "type": "object",
            "title": "Response",
            "properties": {
                "answer": {"type": "string"},
                "confidence": {"type": "number"}
            },
            "required": ["answer", "confidence"]
        });

        let system_prompt = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "prompt".to_string(),
                "PromptTemplate".to_string(),
            ],
            PromptTemplateKwargs {
                input_variables: vec![],
                template: "You are a helpful assistant.".to_string(),
                template_format: "f-string".to_string(),
            },
        );

        let system_message = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "chat".to_string(),
                "SystemMessagePromptTemplate".to_string(),
            ],
            MessagePromptTemplateKwargs {
                prompt: system_prompt,
            },
        );

        let structured_prompt = StructuredPrompt {
            input_variables: None,
            messages: vec![system_message],
            schema_: schema.clone(),
            structured_output_kwargs: StructuredOutputKwargs {
                method: "json_schema".to_string(),
            },
        };

        let wrapped = structured_prompt.to_lc_json();
        let serialized = serde_json::to_value(&wrapped).unwrap();

        // Verify top-level structure
        assert_eq!(serialized["lc"], 1);
        assert_eq!(serialized["type"], "constructor");
        assert!(serialized["id"].is_array());
        assert!(serialized["kwargs"].is_object());

        // Verify kwargs structure
        let kwargs = &serialized["kwargs"];
        assert!(kwargs["messages"].is_array());
        assert_eq!(kwargs["schema_"], schema);
        assert_eq!(kwargs["structured_output_kwargs"]["method"], "json_schema");

        // Verify message structure
        let first_message = &kwargs["messages"][0];
        assert_eq!(first_message["lc"], 1);
        assert_eq!(first_message["type"], "constructor");
        assert!(first_message["kwargs"]["prompt"].is_object());
    }

    #[test]
    fn test_function_calling_method() {
        // Test "function_calling" method (alternative to "json_schema")
        let schema = json!({"type": "object"});

        let prompt_template = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "prompt".to_string(),
                "PromptTemplate".to_string(),
            ],
            PromptTemplateKwargs {
                input_variables: vec![],
                template: "Test".to_string(),
                template_format: "f-string".to_string(),
            },
        );

        let message = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "chat".to_string(),
                "SystemMessagePromptTemplate".to_string(),
            ],
            MessagePromptTemplateKwargs {
                prompt: prompt_template,
            },
        );

        let structured_prompt = StructuredPrompt {
            input_variables: None,
            messages: vec![message],
            schema_: schema,
            structured_output_kwargs: StructuredOutputKwargs {
                method: "function_calling".to_string(),
            },
        };

        let serialized = serde_json::to_value(&structured_prompt).unwrap();
        assert_eq!(
            serialized["structured_output_kwargs"]["method"],
            "function_calling"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Schema Validation Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_validate_json_schema_valid() {
        let valid_schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer", "minimum": 0}
            },
            "required": ["name"]
        });

        assert!(validate_json_schema(&valid_schema).is_ok());
    }

    #[test]
    fn test_validate_json_schema_invalid_type() {
        let invalid_schema = json!({
            "type": "invalid_type",
            "properties": {}
        });

        let result = validate_json_schema(&invalid_schema);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LangstarError::InvalidSchemaError(_)
        ));
    }

    #[test]
    fn test_validate_json_schema_malformed() {
        let malformed_schema = json!({
            "properties": {
                "name": {"type": "string"}
            },
            "required": "should_be_array_not_string"
        });

        let result = validate_json_schema(&malformed_schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_method_json_schema() {
        assert!(validate_method("json_schema").is_ok());
    }

    #[test]
    fn test_validate_method_function_calling() {
        assert!(validate_method("function_calling").is_ok());
    }

    #[test]
    fn test_validate_method_invalid() {
        let result = validate_method("invalid_method");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LangstarError::InvalidMethodError(_)
        ));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Push/Pull Integration Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_push_structured_prompt_validates_schema() {
        // Test that push_structured_prompt validates the schema
        // This is a synchronous test that verifies schema validation logic

        let invalid_schema = json!({"type": "invalid_type"});

        let prompt_template = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "prompt".to_string(),
                "PromptTemplate".to_string(),
            ],
            PromptTemplateKwargs {
                input_variables: vec![],
                template: "Test".to_string(),
                template_format: "f-string".to_string(),
            },
        );

        let message = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "chat".to_string(),
                "SystemMessagePromptTemplate".to_string(),
            ],
            MessagePromptTemplateKwargs {
                prompt: prompt_template,
            },
        );

        let structured_prompt = StructuredPrompt {
            input_variables: None,
            messages: vec![message],
            schema_: invalid_schema,
            structured_output_kwargs: StructuredOutputKwargs {
                method: "json_schema".to_string(),
            },
        };

        // Verify that validation would fail
        let validation_result = validate_json_schema(&structured_prompt.schema_);
        assert!(validation_result.is_err());
    }

    #[test]
    fn test_push_structured_prompt_validates_method() {
        // Test that push_structured_prompt validates the method

        let valid_schema = json!({
            "type": "object",
            "properties": {"answer": {"type": "string"}}
        });

        let prompt_template = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "prompt".to_string(),
                "PromptTemplate".to_string(),
            ],
            PromptTemplateKwargs {
                input_variables: vec![],
                template: "Test".to_string(),
                template_format: "f-string".to_string(),
            },
        );

        let message = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "chat".to_string(),
                "SystemMessagePromptTemplate".to_string(),
            ],
            MessagePromptTemplateKwargs {
                prompt: prompt_template,
            },
        );

        let structured_prompt = StructuredPrompt {
            input_variables: None,
            messages: vec![message],
            schema_: valid_schema,
            structured_output_kwargs: StructuredOutputKwargs {
                method: "invalid_method".to_string(),
            },
        };

        // Verify that method validation would fail
        let validation_result = validate_method(&structured_prompt.structured_output_kwargs.method);
        assert!(validation_result.is_err());
    }

    #[test]
    fn test_structured_prompt_serialization_for_api() {
        // Test that StructuredPrompt serializes correctly for API submission
        let schema = json!({
            "type": "object",
            "properties": {
                "answer": {"type": "string"},
                "confidence": {"type": "number", "minimum": 0.0, "maximum": 1.0}
            },
            "required": ["answer"]
        });

        let prompt_template = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "prompt".to_string(),
                "PromptTemplate".to_string(),
            ],
            PromptTemplateKwargs {
                input_variables: vec![],
                template: "You are a helpful assistant.".to_string(),
                template_format: "f-string".to_string(),
            },
        );

        let message = LcJson::new(
            vec![
                "langchain".to_string(),
                "prompts".to_string(),
                "chat".to_string(),
                "SystemMessagePromptTemplate".to_string(),
            ],
            MessagePromptTemplateKwargs {
                prompt: prompt_template,
            },
        );

        let structured_prompt = StructuredPrompt {
            input_variables: None,
            messages: vec![message],
            schema_: schema.clone(),
            structured_output_kwargs: StructuredOutputKwargs {
                method: "json_schema".to_string(),
            },
        };

        // Wrap in LC-JSON
        let lc_json = structured_prompt.to_lc_json();

        // Serialize to JSON
        let manifest = serde_json::to_value(&lc_json).unwrap();

        // Verify structure matches API expectations
        assert_eq!(manifest["lc"], 1);
        assert_eq!(manifest["type"], "constructor");
        assert_eq!(
            manifest["id"],
            json!([
                "langchain_core",
                "prompts",
                "structured",
                "StructuredPrompt"
            ])
        );
        assert_eq!(manifest["kwargs"]["schema_"], schema);
        assert_eq!(
            manifest["kwargs"]["structured_output_kwargs"]["method"],
            "json_schema"
        );
    }

    #[test]
    fn test_pull_structured_prompt_deserialization() {
        // Test that we can deserialize a StructuredPrompt from LC-JSON format
        let schema = json!({
            "type": "object",
            "properties": {
                "title": {"type": "string"},
                "rating": {"type": "integer", "minimum": 1, "maximum": 10}
            },
            "required": ["title", "rating"]
        });

        // Simulate an API response with LC-JSON format
        let api_manifest = json!({
            "lc": 1,
            "type": "constructor",
            "id": ["langchain_core", "prompts", "structured", "StructuredPrompt"],
            "name": "StructuredPrompt",
            "kwargs": {
                "input_variables": ["movie_name"],
                "messages": [
                    {
                        "lc": 1,
                        "type": "constructor",
                        "id": ["langchain", "prompts", "chat", "SystemMessagePromptTemplate"],
                        "kwargs": {
                            "prompt": {
                                "lc": 1,
                                "type": "constructor",
                                "id": ["langchain", "prompts", "prompt", "PromptTemplate"],
                                "kwargs": {
                                    "input_variables": [],
                                    "template": "You are a movie critic.",
                                    "template_format": "f-string"
                                }
                            }
                        }
                    }
                ],
                "schema_": schema.clone(),
                "structured_output_kwargs": {
                    "method": "json_schema"
                }
            }
        });

        // Deserialize
        let lc_json: LcJson<StructuredPrompt> = serde_json::from_value(api_manifest).unwrap();
        let structured_prompt = lc_json.kwargs;

        // Verify deserialization
        assert_eq!(structured_prompt.schema_, schema);
        assert_eq!(
            structured_prompt.structured_output_kwargs.method,
            "json_schema"
        );
        assert_eq!(
            structured_prompt.input_variables,
            Some(vec!["movie_name".to_string()])
        );
        assert_eq!(structured_prompt.messages.len(), 1);
    }
}
