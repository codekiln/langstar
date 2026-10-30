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
    /// Name of the prompt (without owner prefix)
    pub repo_handle: String,
    /// Full handle including owner (e.g., "langchain-ai/rag-prompt")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    /// Owner of the prompt (e.g., "langchain-ai"), or None for private prompts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Description of the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The prompt's readme, the long text beside its description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
    /// The prompt's own tags, which name the prompt rather than one of its commits
    #[serde(default)]
    pub tags: Vec<String>,
    /// The ID of the workspace that holds the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
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

        // Build query string with is_public parameter for server-side filtering
        // See: reference/api-specs/langsmith/prompt-endpoints.json:161-178
        let mut path = format!("/api/v1/repos/?limit={}&offset={}", limit, offset);

        match visibility {
            Visibility::Public => path.push_str("&is_public=true"),
            Visibility::Private => path.push_str("&is_public=false"),
            Visibility::Any => {} // Don't add is_public parameter - return all
        }

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
    /// * `handle` - The prompt handle (e.g., "owner/prompt-name" or just "prompt-name")
    ///
    /// # Note
    /// If the handle doesn't contain an owner prefix (no `/`), this method
    /// prepends `-/` which represents the current user's private prompts.
    /// The LangSmith API returns `repo_handle` without the owner prefix,
    /// but expects `GET /api/v1/repos/{owner}/{repo}` with owner in the path.
    pub async fn get(&self, handle: &str) -> Result<Prompt> {
        // If handle doesn't contain '/', prepend '-/' for private prompts
        // The '-' owner represents the current user's private prompts
        let full_handle = if handle.contains('/') {
            handle.to_string()
        } else {
            format!("-/{}", handle)
        };
        let path = format!("/api/v1/repos/{}", full_handle);
        let request = self.client.langsmith_get(&path)?;

        // The API wraps the prompt in a "repo" field
        #[derive(Deserialize)]
        struct PromptResponse {
            repo: Prompt,
        }

        let response: PromptResponse = self.client.execute(request).await?;
        Ok(response.repo)
    }

    /// Get a specific prompt by UUID
    ///
    /// # Arguments
    /// * `id` - The prompt UUID
    ///
    /// # Note
    /// This method searches through the prompt list to find a prompt with the given ID.
    /// It uses pagination to search through all prompts until the matching ID is found.
    /// It's less efficient than `get()` but necessary since the API doesn't provide
    /// a direct UUID-based endpoint.
    pub async fn get_by_id(&self, id: &str) -> Result<Prompt> {
        // Search through all prompts using pagination to find one with matching ID
        let page_size = 100;
        let mut offset = 0;

        loop {
            let prompts = self.list(Some(page_size), Some(offset), None).await?;

            // If we get no results, we've exhausted all prompts
            if prompts.is_empty() {
                break;
            }

            // Check if any prompt in this page matches the ID
            if let Some(prompt) = prompts.iter().find(|p| p.id == id) {
                return Ok(prompt.clone());
            }

            // Move to next page
            offset += page_size;
        }

        // Prompt not found after searching all pages
        Err(crate::error::LangstarError::ApiError {
            status: 404,
            message: format!("Prompt with ID '{}' not found", id),
        })
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

        // Build query string with is_public parameter for server-side filtering
        // See: reference/api-specs/langsmith/prompt-endpoints.json:161-178
        let mut path = format!(
            "/api/v1/repos/?query={}&limit={}",
            urlencoding::encode(query),
            limit
        );

        match visibility {
            Visibility::Public => path.push_str("&is_public=true"),
            Visibility::Private => path.push_str("&is_public=false"),
            Visibility::Any => {} // Don't add is_public parameter - return all
        }

        let request = self.client.langsmith_get(&path)?;

        // LangSmith API returns a paginated response with a "repos" field (same as list)
        #[derive(Deserialize)]
        struct SearchReposResponse {
            repos: Vec<Prompt>,
        }

        let response: SearchReposResponse = self.client.execute(request).await?;

        Ok(response.repos)
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
            description: None,
            parent_commit,
            example_run_ids: None,
        };

        // Push using existing method
        self.push(owner, repo, &commit_request).await
    }

    /// Get commit metadata including hash and manifest.
    ///
    /// This retrieves the full commit response including the commit hash,
    /// which is needed when creating child commits with parent references.
    ///
    /// # Arguments
    /// * `owner` - The owner of the prompt (username or organization)
    /// * `repo` - The prompt repository name
    /// * `commit` - The commit hash or tag (e.g., "latest", "main", commit SHA)
    ///
    /// # Returns
    /// The full commit response including commit_hash and manifest
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    /// let prompts = client.prompts();
    ///
    /// let commit_info = prompts.get_commit("owner", "my-prompt", "latest").await?;
    /// println!("Latest commit hash: {}", commit_info.commit_hash);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_commit(
        &self,
        owner: &str,
        repo: &str,
        commit: &str,
    ) -> Result<CommitManifestResponse> {
        let path = format!("/api/v1/commits/{}/{}/{}", owner, repo, commit);
        let request = self.client.langsmith_get(&path)?;
        let response: CommitManifestResponse = self.client.execute(request).await?;
        Ok(response)
    }

    /// Get a commit with the chat model inside its manifest.
    ///
    /// [`get_commit`](Self::get_commit) returns the prompt on its own. This call sends
    /// `include=model`, and LangSmith then wraps the prompt and the chat model in one
    /// `RunnableSequence`: `manifest.kwargs.first` is the prompt and `manifest.kwargs.last`
    /// is a `RunnableBinding` whose `kwargs.bound` is the chat model constructor. A reader
    /// after the model's class name, such as `ChatOpenAI`, takes the last element of that
    /// constructor's `id`. The binding's own `id` ends in `RunnableBinding`.
    /// `reference/research/807-openapi-validation.md` records those reads against the
    /// live API.
    ///
    /// `model_config` and `model_provider` come back either way, so the manifest is what
    /// says whether the read asked for the model. It is also the only place a commit
    /// create can carry a model to another workspace.
    ///
    /// The older `include_model=true`, which the LangSmith Python SDK sends, returns the
    /// same document, and the spec marks it deprecated.
    ///
    /// # Arguments
    /// * `owner` - The owner of the prompt (username or organization)
    /// * `repo` - The prompt repository name
    /// * `commit` - The commit hash or tag (e.g., "latest", "main", commit SHA)
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let commit = client
    ///     .prompts()
    ///     .get_commit_with_model("owner", "my-prompt", "latest")
    ///     .await?;
    /// if let Some(provider) = &commit.model_provider {
    ///     println!("Model class: {}", provider);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_commit_with_model(
        &self,
        owner: &str,
        repo: &str,
        commit: &str,
    ) -> Result<CommitManifestResponse> {
        // A tag name or hash goes into the path, so a reserved character in it would
        // otherwise cut the request short and drop `include=model`.
        let path = format!(
            "/api/v1/commits/{}/{}/{}?include=model",
            owner,
            repo,
            urlencoding::encode(commit)
        );
        let request = self.client.langsmith_get(&path)?;
        let response: CommitManifestResponse = self.client.execute(request).await?;
        Ok(response)
    }

    /// List a prompt's commits, newest first.
    ///
    /// Each entry carries the commit's `id` and its hash. The commit read returns only
    /// the hash, and the tag endpoints take the `id`, so a caller that wants to tag a
    /// commit it knows by hash finds the ID here.
    ///
    /// # Arguments
    /// * `owner` - The owner of the prompt (username or organization)
    /// * `repo` - The prompt repository name
    /// * `limit` - How many commits to return. The API returns 20 by default and at most 100.
    /// * `offset` - How many commits to skip (default: 0)
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let commits = client
    ///     .prompts()
    ///     .list_commits("owner", "my-prompt", Some(10), None)
    ///     .await?;
    /// for commit in &commits.commits {
    ///     println!("{} {}", commit.id, commit.commit_hash);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_commits(
        &self,
        owner: &str,
        repo: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<CommitList> {
        let mut path = format!("/api/v1/commits/{}/{}", owner, repo);
        let mut separator = '?';
        if let Some(limit) = limit {
            path.push_str(&format!("{}limit={}", separator, limit));
            separator = '&';
        }
        if let Some(offset) = offset {
            path.push_str(&format!("{}offset={}", separator, offset));
        }
        let request = self.client.langsmith_get(&path)?;
        let response: CommitList = self.client.execute(request).await?;
        Ok(response)
    }

    /// List a prompt's commit tags.
    ///
    /// # Arguments
    /// * `owner` - The owner of the prompt (username or organization)
    /// * `repo` - The prompt repository name
    pub async fn list_tags(&self, owner: &str, repo: &str) -> Result<Vec<RepoTag>> {
        let path = format!("/api/v1/repos/{}/{}/tags", owner, repo);
        let request = self.client.langsmith_get(&path)?;
        let tags: Vec<RepoTag> = self.client.execute(request).await?;
        Ok(tags)
    }

    /// Put a new tag on a commit.
    ///
    /// # Arguments
    /// * `owner` - The owner of the prompt (username or organization)
    /// * `repo` - The prompt repository name
    /// * `tag_name` - The tag's name, such as `prod`
    /// * `commit_id` - The commit's ID, which [`list_commits`](Self::list_commits) and a
    ///   push both return. `RepoTagRequest` in
    ///   `reference/api-specs/langsmith/prompt-schemas.json` types it as a UUID.
    pub async fn create_tag(
        &self,
        owner: &str,
        repo: &str,
        tag_name: &str,
        commit_id: &str,
    ) -> Result<RepoTag> {
        #[derive(Serialize)]
        struct CreateTagRequest<'a> {
            tag_name: &'a str,
            commit_id: &'a str,
        }

        let path = format!("/api/v1/repos/{}/{}/tags", owner, repo);
        let request = self.client.langsmith_post(&path)?.json(&CreateTagRequest {
            tag_name,
            commit_id,
        });
        let tag: RepoTag = self.client.execute(request).await?;
        Ok(tag)
    }

    /// Move an existing tag to another commit.
    ///
    /// # Arguments
    /// * `owner` - The owner of the prompt (username or organization)
    /// * `repo` - The prompt repository name
    /// * `tag_name` - The name of the tag to move
    /// * `commit_id` - The ID of the commit the tag should point at
    pub async fn move_tag(
        &self,
        owner: &str,
        repo: &str,
        tag_name: &str,
        commit_id: &str,
    ) -> Result<RepoTag> {
        #[derive(Serialize)]
        struct MoveTagRequest<'a> {
            commit_id: &'a str,
        }

        let path = format!(
            "/api/v1/repos/{}/{}/tags/{}",
            owner,
            repo,
            urlencoding::encode(tag_name)
        );
        let request = self
            .client
            .langsmith_patch(&path)?
            .json(&MoveTagRequest { commit_id });
        let tag: RepoTag = self.client.execute(request).await?;
        Ok(tag)
    }

    /// Delete a tag, leaving the commit it pointed at alone.
    ///
    /// # Arguments
    /// * `owner` - The owner of the prompt (username or organization)
    /// * `repo` - The prompt repository name
    /// * `tag_name` - The name of the tag to delete
    pub async fn delete_tag(&self, owner: &str, repo: &str, tag_name: &str) -> Result<()> {
        let path = format!(
            "/api/v1/repos/{}/{}/tags/{}",
            owner,
            repo,
            urlencoding::encode(tag_name)
        );
        let request = self.client.langsmith_delete(&path)?;
        self.client.execute_status_only_request(request).await
    }

    /// Change a prompt's description, readme, tags or visibility.
    ///
    /// These details live on the prompt rather than on a commit, so this call changes
    /// them without pushing one.
    ///
    /// # Arguments
    /// * `owner` - The owner of the prompt (username or organization)
    /// * `repo` - The prompt repository name
    /// * `update` - The details to change; each `None` field stays as it is
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, prompts::UpdateRepoRequest};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let updated = client
    ///     .prompts()
    ///     .update_repo(
    ///         "-",
    ///         "my-prompt",
    ///         &UpdateRepoRequest {
    ///             description: Some("Answers support questions".to_string()),
    ///             ..Default::default()
    ///         },
    ///     )
    ///     .await?;
    /// println!("{:?}", updated.description);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_repo(
        &self,
        owner: &str,
        repo: &str,
        update: &UpdateRepoRequest,
    ) -> Result<Prompt> {
        let path = format!("/api/v1/repos/{}/{}", owner, repo);
        let request = self.client.langsmith_patch(&path)?.json(update);

        #[derive(Deserialize)]
        struct UpdateRepoResponse {
            repo: Prompt,
        }

        let response: UpdateRepoResponse = self.client.execute(request).await?;
        Ok(response.repo)
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
        // Deserialize only the manifest so pull() stays tolerant of responses that
        // omit commit_hash (get_commit() enforces it).
        let path = format!("/api/v1/commits/{}/{}/{}", owner, repo, commit);
        let request = self.client.langsmith_get(&path)?;
        let response: ManifestOnlyResponse = self.client.execute(request).await?;
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

    /// Delete a prompt from the PromptHub.
    ///
    /// For private prompts owned by the current user/organization, use "-" as the owner.
    /// This follows the LangSmith API pattern where "-" refers to the authenticated user's namespace.
    ///
    /// # Arguments
    /// * `prompt_name` - The name of the prompt to delete (without owner prefix)
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    /// let prompts = client.prompts();
    ///
    /// // Delete a private prompt
    /// prompts.delete("my-test-prompt").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `DELETE /api/v1/repos/-/{prompt_name}`
    /// - The "-" owner refers to the current authenticated user/organization
    /// - See: Python SDK `langsmith/client.py:7630-7647` for reference implementation
    pub async fn delete(&self, prompt_name: &str) -> Result<()> {
        // Use "-" as owner for private prompts (current user's namespace)
        // Example: DELETE https://api.smith.langchain.com/repos/-/my-prompt
        let path = format!("/api/v1/repos/-/{}", prompt_name);
        let request = self.client.langsmith_delete(&path)?;
        self.client.execute_status_only_request(request).await
    }

    /// Delete a prompt by full handle (owner/repo format).
    ///
    /// # Arguments
    /// * `handle` - The full prompt handle in "owner/repo" format
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    /// let prompts = client.prompts();
    ///
    /// // Delete using full handle
    /// prompts.delete_by_handle("my-org/my-prompt").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_by_handle(&self, handle: &str) -> Result<()> {
        let (owner, repo) = handle.split_once('/').ok_or_else(|| {
            LangstarError::InvalidInput("Handle must be in format 'owner/repo'".into())
        })?;
        let path = format!("/api/v1/repos/{}/{}", owner, repo);
        let request = self.client.langsmith_delete(&path)?;
        self.client.execute_status_only_request(request).await
    }
}

/// Request to create a commit (upload/update a prompt)
///
/// Corresponds to the LangSmith API CreateRepoCommitRequest schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRequest {
    /// The prompt manifest/template (required)
    pub manifest: serde_json::Value,
    /// What this commit changes, which the commit read returns as `description`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
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
    /// The commit's ID, which the tag endpoints take.
    ///
    /// `RepoTagRequest` in `reference/api-specs/langsmith/prompt-schemas.json` types it
    /// as a UUID, and a commit's hash is a different string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Commit hash
    pub commit_hash: String,
    /// URL to the commit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// One commit in a prompt's commit list
///
/// Returned by `GET /api/v1/commits/{owner}/{repo}`. The commit read returns the hash
/// alone, so this list is where a caller finds the ID of a commit it wants to tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitListEntry {
    /// The commit's ID, which the tag endpoints take
    pub id: String,
    /// The commit's hash
    pub commit_hash: String,
    /// What the person who pushed the commit wrote about it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The hash of the commit this one was pushed on top of
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_commit_hash: Option<String>,
    /// When the commit was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// A page of a prompt's commits, newest first
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitList {
    /// The commits on this page
    pub commits: Vec<CommitListEntry>,
    /// How many commits the prompt has in all
    #[serde(default)]
    pub total: u32,
}

/// A tag on a prompt commit, such as `prod`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoTag {
    /// The tag's own ID
    pub id: String,
    /// The tag's name
    pub tag_name: String,
    /// The ID of the commit the tag points at
    pub commit_id: String,
    /// The hash of the commit the tag points at
    pub commit_hash: String,
    /// The ID of the prompt the tag belongs to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_id: Option<String>,
    /// When the tag was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// When the tag last moved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Repo details a repo update sends
///
/// Each field the caller leaves `None` stays out of the request body, and LangSmith
/// leaves that detail as it is.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateRepoRequest {
    /// The prompt's one-line description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The prompt's readme
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
    /// The prompt's tags, which name the prompt rather than a commit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Whether anyone can read the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
}

/// Response from getting commit metadata
///
/// Returned by the GET /api/v1/commits/{owner}/{repo}/{commit} endpoint.
/// Includes both the commit hash and the full manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitManifestResponse {
    /// The commit hash
    pub commit_hash: String,
    /// The commit manifest (prompt template/schema)
    pub manifest: serde_json::Value,
    /// The commit's description, which the person who pushed it wrote
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The model's settings, beside the manifest rather than inside it.
    ///
    /// LangSmith leaves this field out for a commit with no model, rather than
    /// returning null, so `None` means the commit has no model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_config: Option<serde_json::Value>,
    /// The chat model's class name, such as `ChatOpenAI`.
    ///
    /// Left out, like `model_config`, for a commit with no model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_provider: Option<String>,
    /// Optional example run records (serialized RepoExampleResponse objects)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<serde_json::Value>>,
}

/// Minimal commit response used by `pull()`, which only needs the manifest
#[derive(Debug, Deserialize)]
struct ManifestOnlyResponse {
    manifest: serde_json::Value,
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
        let auth = AuthConfig::new(Some("test".to_string()), None, None);
        let client = LangchainClient::new(auth).unwrap();
        let _prompt_client = client.prompts();
    }

    #[test]
    fn test_prompt_serialization() {
        let prompt = Prompt {
            id: "test-id".to_string(),
            repo_handle: "prompt".to_string(),
            full_name: Some("owner/prompt".to_string()),
            owner: Some("owner".to_string()),
            description: Some("Test prompt".to_string()),
            readme: None,
            tags: vec![],
            tenant_id: None,
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

#[cfg(test)]
mod export_field_tests {
    use super::*;
    use crate::auth::AuthConfig;
    use serde_json::json;

    fn mock_client(server: &mockito::ServerGuard) -> LangchainClient {
        LangchainClient::with_base_urls(
            AuthConfig::new(
                Some("test_key".to_string()),
                None,
                Some("test_workspace".to_string()),
            ),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap()
    }

    /// A commit read as the live API returns it for a prompt saved with a model.
    ///
    /// The outer constructor is a `RunnableSequence`, `kwargs.first` is the prompt and
    /// `kwargs.last` is a `RunnableBinding` whose `kwargs.bound` is the `ChatOpenAI`
    /// constructor. `reference/research/807-openapi-validation.md` read three such
    /// commits on the live API and found the binding in every one.
    fn commit_with_model_body() -> serde_json::Value {
        json!({
            "commit_hash": "8c0ba491ce9b01f707e56db935d7eeee7aa6ff29c04258b458928cf7c8ebe35b",
            "description": "Raise the temperature",
            "manifest": {
                "lc": 1,
                "type": "constructor",
                "id": ["langchain", "schema", "runnable", "RunnableSequence"],
                "kwargs": {
                    "first": {
                        "lc": 1,
                        "type": "constructor",
                        "id": ["langchain", "prompts", "chat", "ChatPromptTemplate"],
                        "kwargs": {"input_variables": ["question"], "messages": []}
                    },
                    "last": {
                        "lc": 1,
                        "type": "constructor",
                        "id": ["langchain", "schema", "runnable", "RunnableBinding"],
                        "kwargs": {
                            "bound": {
                                "lc": 1,
                                "type": "constructor",
                                "id": ["langchain", "chat_models", "openai", "ChatOpenAI"],
                                "kwargs": {
                                    "model": "gpt-4.1-mini",
                                    "temperature": 0,
                                    "stream_usage": true,
                                    "openai_api_key": {
                                        "lc": 1,
                                        "type": "secret",
                                        "id": ["OPENAI_API_KEY"]
                                    }
                                }
                            },
                            "kwargs": {}
                        }
                    }
                }
            },
            "model_config": {
                "model": "gpt-4.1-mini",
                "temperature": 0,
                "stream_usage": true,
                "openai_api_key": {"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}
            },
            "model_provider": "ChatOpenAI"
        })
    }

    #[tokio::test]
    async fn test_get_commit_with_model_sends_include_model() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/commits/-/my-prompt/latest")
            .match_query(mockito::Matcher::UrlEncoded(
                "include".into(),
                "model".into(),
            ))
            .with_body(commit_with_model_body().to_string())
            .create_async()
            .await;

        let commit = mock_client(&server)
            .prompts()
            .get_commit_with_model("-", "my-prompt", "latest")
            .await
            .expect("the commit read should succeed");

        mock.assert_async().await;
        assert_eq!(commit.description.as_deref(), Some("Raise the temperature"));
        assert_eq!(commit.model_provider.as_deref(), Some("ChatOpenAI"));
        assert_eq!(
            commit
                .model_config
                .as_ref()
                .and_then(|c| c["model"].as_str()),
            Some("gpt-4.1-mini")
        );
        // The chat model sits under the RunnableBinding.
        assert_eq!(
            commit.manifest["kwargs"]["last"]["id"]
                .as_array()
                .and_then(|id| id.last())
                .and_then(|name| name.as_str()),
            Some("RunnableBinding")
        );
        assert_eq!(
            commit.manifest["kwargs"]["last"]["kwargs"]["bound"]["id"]
                .as_array()
                .and_then(|id| id.last())
                .and_then(|name| name.as_str()),
            Some("ChatOpenAI")
        );
    }

    #[tokio::test]
    async fn test_commit_read_leaves_out_the_model_for_a_prompt_without_one() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/api/v1/commits/-/plain-prompt/latest")
            .match_query(mockito::Matcher::UrlEncoded(
                "include".into(),
                "model".into(),
            ))
            .with_body(
                json!({
                    "commit_hash": "abc123",
                    "manifest": {"lc": 1, "type": "constructor", "id": ["x"], "kwargs": {}}
                })
                .to_string(),
            )
            .create_async()
            .await;

        let commit = mock_client(&server)
            .prompts()
            .get_commit_with_model("-", "plain-prompt", "latest")
            .await
            .expect("the commit read should succeed");

        assert!(commit.model_config.is_none());
        assert!(commit.model_provider.is_none());
        assert!(commit.description.is_none());
    }

    #[tokio::test]
    async fn test_push_sends_the_commit_description_and_reads_back_the_commit_id() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/v1/commits/-/my-prompt")
            .match_body(mockito::Matcher::PartialJson(
                json!({"description": "Add the system message"}),
            ))
            .with_body(
                json!({"commit": {
                    "id": "4f2b2d9e-0b4a-4a1f-9f3a-3b6b5f1c2d7e",
                    "commit_hash": "abc123"
                }})
                .to_string(),
            )
            .create_async()
            .await;

        let response = mock_client(&server)
            .prompts()
            .push(
                "-",
                "my-prompt",
                &CommitRequest {
                    manifest: json!({"lc": 1}),
                    description: Some("Add the system message".to_string()),
                    parent_commit: None,
                    example_run_ids: None,
                },
            )
            .await
            .expect("the push should succeed");

        mock.assert_async().await;
        assert_eq!(
            response.commit.id.as_deref(),
            Some("4f2b2d9e-0b4a-4a1f-9f3a-3b6b5f1c2d7e")
        );
        assert_eq!(response.commit.commit_hash, "abc123");
    }

    #[tokio::test]
    async fn test_a_reserved_character_in_a_tag_name_or_commit_stays_in_the_path() {
        let mut server = mockito::Server::new_async().await;
        let read = server
            .mock("GET", "/api/v1/commits/-/my-prompt/prod%23candidate")
            .match_query(mockito::Matcher::UrlEncoded(
                "include".into(),
                "model".into(),
            ))
            .with_body(json!({"commit_hash": "abc123", "manifest": {"lc": 1}}).to_string())
            .create_async()
            .await;
        let moved = server
            .mock("PATCH", "/api/v1/repos/-/my-prompt/tags/prod%23candidate")
            .with_body(
                json!({
                    "id": "tag-1",
                    "repo_id": "repo-1",
                    "commit_id": "id-second",
                    "commit_hash": "hash-second",
                    "tag_name": "prod#candidate"
                })
                .to_string(),
            )
            .create_async()
            .await;
        let deleted = server
            .mock("DELETE", "/api/v1/repos/-/my-prompt/tags/prod%23candidate")
            .with_status(204)
            .create_async()
            .await;

        let client = mock_client(&server);
        client
            .prompts()
            .get_commit_with_model("-", "my-prompt", "prod#candidate")
            .await
            .expect("the commit read should succeed");
        client
            .prompts()
            .move_tag("-", "my-prompt", "prod#candidate", "id-second")
            .await
            .expect("the tag move should succeed");
        client
            .prompts()
            .delete_tag("-", "my-prompt", "prod#candidate")
            .await
            .expect("the tag delete should succeed");

        // Without the encoding each request would land on `prod` and the commit read
        // would lose `include=model`, so these three assertions are the whole test.
        read.assert_async().await;
        moved.assert_async().await;
        deleted.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_commits_returns_each_commit_id_with_its_hash() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/commits/-/my-prompt")
            .match_query(mockito::Matcher::UrlEncoded("limit".into(), "2".into()))
            .with_body(
                json!({
                    "commits": [
                        {"id": "id-second", "commit_hash": "hash-second",
                         "parent_commit_hash": "hash-first", "manifest": {}},
                        {"id": "id-first", "commit_hash": "hash-first", "manifest": {}}
                    ],
                    "total": 2
                })
                .to_string(),
            )
            .create_async()
            .await;

        let commits = mock_client(&server)
            .prompts()
            .list_commits("-", "my-prompt", Some(2), None)
            .await
            .expect("the commit list should succeed");

        mock.assert_async().await;
        assert_eq!(commits.total, 2);
        assert_eq!(commits.commits[0].id, "id-second");
        assert_eq!(commits.commits[0].commit_hash, "hash-second");
        assert_eq!(
            commits.commits[0].parent_commit_hash.as_deref(),
            Some("hash-first")
        );
        assert_eq!(commits.commits[1].id, "id-first");
    }

    #[tokio::test]
    async fn test_create_tag_names_the_commit_by_its_id() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api/v1/repos/-/my-prompt/tags")
            .match_body(mockito::Matcher::Json(json!({
                "tag_name": "prod",
                "commit_id": "id-first"
            })))
            .with_body(
                json!({
                    "id": "tag-1",
                    "repo_id": "repo-1",
                    "commit_id": "id-first",
                    "commit_hash": "hash-first",
                    "tag_name": "prod",
                    "created_at": "2026-10-30T00:00:00Z",
                    "updated_at": "2026-10-30T00:00:00Z"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let tag = mock_client(&server)
            .prompts()
            .create_tag("-", "my-prompt", "prod", "id-first")
            .await
            .expect("the tag create should succeed");

        mock.assert_async().await;
        assert_eq!(tag.tag_name, "prod");
        assert_eq!(tag.commit_id, "id-first");
    }

    #[tokio::test]
    async fn test_move_tag_sends_the_new_commit_id() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PATCH", "/api/v1/repos/-/my-prompt/tags/prod")
            .match_body(mockito::Matcher::Json(json!({"commit_id": "id-second"})))
            .with_body(
                json!({
                    "id": "tag-1",
                    "repo_id": "repo-1",
                    "commit_id": "id-second",
                    "commit_hash": "hash-second",
                    "tag_name": "prod",
                    "created_at": "2026-10-30T00:00:00Z",
                    "updated_at": "2026-10-30T00:01:00Z"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let tag = mock_client(&server)
            .prompts()
            .move_tag("-", "my-prompt", "prod", "id-second")
            .await
            .expect("the tag move should succeed");

        mock.assert_async().await;
        assert_eq!(tag.commit_id, "id-second");
        assert_eq!(tag.commit_hash, "hash-second");
    }

    #[tokio::test]
    async fn test_delete_tag_sends_the_tag_name_in_the_path() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("DELETE", "/api/v1/repos/-/my-prompt/tags/prod")
            .with_status(204)
            .create_async()
            .await;

        mock_client(&server)
            .prompts()
            .delete_tag("-", "my-prompt", "prod")
            .await
            .expect("the tag delete should succeed");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_tags_returns_each_tag_with_the_commit_it_points_at() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/api/v1/repos/-/my-prompt/tags")
            .with_body(
                json!([{
                    "id": "tag-1",
                    "repo_id": "repo-1",
                    "commit_id": "id-first",
                    "commit_hash": "hash-first",
                    "tag_name": "prod",
                    "created_at": "2026-10-30T00:00:00Z",
                    "updated_at": "2026-10-30T00:00:00Z"
                }])
                .to_string(),
            )
            .create_async()
            .await;

        let tags = mock_client(&server)
            .prompts()
            .list_tags("-", "my-prompt")
            .await
            .expect("the tag list should succeed");

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].tag_name, "prod");
        assert_eq!(tags[0].commit_hash, "hash-first");
    }

    #[tokio::test]
    async fn test_update_repo_sends_each_detail_the_caller_set() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PATCH", "/api/v1/repos/-/my-prompt")
            .match_body(mockito::Matcher::Json(json!({
                "description": "Answers support questions",
                "readme": "# My prompt",
                "tags": ["support"],
                "is_public": false
            })))
            .with_body(
                json!({"repo": {
                    "id": "repo-1",
                    "repo_handle": "my-prompt",
                    "description": "Answers support questions",
                    "readme": "# My prompt",
                    "tags": ["support"],
                    "tenant_id": "workspace-1",
                    "is_public": false
                }})
                .to_string(),
            )
            .create_async()
            .await;

        let updated = mock_client(&server)
            .prompts()
            .update_repo(
                "-",
                "my-prompt",
                &UpdateRepoRequest {
                    description: Some("Answers support questions".to_string()),
                    readme: Some("# My prompt".to_string()),
                    tags: Some(vec!["support".to_string()]),
                    is_public: Some(false),
                },
            )
            .await
            .expect("the repo update should succeed");

        mock.assert_async().await;
        assert_eq!(updated.readme.as_deref(), Some("# My prompt"));
        assert_eq!(updated.tags, vec!["support".to_string()]);
    }

    #[tokio::test]
    async fn test_update_repo_leaves_out_every_detail_the_caller_left_alone() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PATCH", "/api/v1/repos/-/my-prompt")
            .match_body(mockito::Matcher::Json(json!({"is_public": true})))
            .with_body(
                json!({"repo": {
                    "id": "repo-1",
                    "repo_handle": "my-prompt",
                    "is_public": true
                }})
                .to_string(),
            )
            .create_async()
            .await;

        let updated = mock_client(&server)
            .prompts()
            .update_repo(
                "-",
                "my-prompt",
                &UpdateRepoRequest {
                    is_public: Some(true),
                    ..Default::default()
                },
            )
            .await
            .expect("the repo update should succeed");

        mock.assert_async().await;
        assert!(updated.is_public);
        assert!(updated.readme.is_none());
        assert!(updated.tags.is_empty());
    }

    #[tokio::test]
    async fn test_get_reads_the_readme_tags_and_tenant_id() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/api/v1/repos/-/my-prompt")
            .with_body(
                json!({"repo": {
                    "id": "repo-1",
                    "repo_handle": "my-prompt",
                    "description": "Answers support questions",
                    "readme": "# My prompt\n\nWhat it does.",
                    "tags": ["support", "triage"],
                    "tenant_id": "workspace-1",
                    "is_public": false
                }})
                .to_string(),
            )
            .create_async()
            .await;

        let prompt = mock_client(&server)
            .prompts()
            .get("my-prompt")
            .await
            .expect("the prompt read should succeed");

        assert_eq!(
            prompt.readme.as_deref(),
            Some("# My prompt\n\nWhat it does.")
        );
        assert_eq!(
            prompt.tags,
            vec!["support".to_string(), "triage".to_string()]
        );
        assert_eq!(prompt.tenant_id.as_deref(), Some("workspace-1"));
    }
}
