//! Langstar SDK - Rust SDK for LangChain ecosystem
//!
//! This SDK provides ergonomic access to LangSmith and LangGraph Cloud APIs.
//! It wraps OpenAPI-generated clients with authentication, error handling,
//! and convenience methods.
//!
//! # Examples
//!
//! ```no_run
//! use langstar_sdk::{AuthConfig, LangchainClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load authentication from environment variables
//!     let auth = AuthConfig::from_env()?;
//!
//!     // Create client
//!     let client = LangchainClient::new(auth)?;
//!
//!     // Make API calls...
//!
//!     Ok(())
//! }
//! ```

pub mod annotation_queues;
pub mod assistants;
pub mod auth;
pub mod client;
pub mod datasets;
pub mod deployments;
pub mod error;
pub mod evaluations;
pub mod evaluators;
pub mod graph;
pub mod graph_client;
pub mod integrations;
pub mod organization;
pub mod playground_settings;
pub mod projects;
pub mod prompts;
pub mod runs;
pub mod secrets;
pub mod serde_utils;

/// Test utilities for integration tests
///
/// Enable with the `test-utils` feature:
/// ```toml
/// [dev-dependencies]
/// langstar-sdk = { features = ["test-utils"] }
/// ```
#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

// Re-export commonly used types
pub use annotation_queues::{
    AnnotationQueue, AnnotationQueueRubricItem, AnnotationQueueWithDetails,
    CreateAnnotationQueueRequest, ListAnnotationQueuesParams, QueueType,
    RunWithAnnotationQueueInfo, UpdateAnnotationQueueRequest,
};
pub use assistants::{
    Assistant, AssistantClient, AssistantSearchRequest, CreateAssistantRequest,
    UpdateAssistantRequest,
};
pub use auth::AuthConfig;
pub use client::{LangchainClient, ListResponse};
pub use datasets::{
    AttachmentsOperations, DataType, Dataset, DatasetCreate, DatasetDiffInfo, DatasetShareSchema,
    DatasetTransformation, DatasetTransformationType, DatasetUpdate, DatasetVersion, Example,
    ExampleBulkUpdate, ExampleCreate, ExampleListOrder, ExampleSelect, ExampleSplit, ExampleUpdate,
    ListDatasetsParams, ListExamplesParams, SortByDatasetColumn,
};
pub use deployments::{
    CreateDeploymentRequest, Deployment, DeploymentClient, DeploymentFilters, DeploymentSecret,
    DeploymentSource, DeploymentStatus, DeploymentType, DeploymentsList, PatchDeploymentRequest,
    Revision, RevisionStatus, RevisionsList,
};
pub use error::{LangstarError, Result};
pub use evaluations::{
    CodeEvaluator, CodeEvaluatorLanguage, EvaluationResult, EvaluatorType, Feedback,
    FeedbackCategory, FeedbackConfig, FeedbackCreate, FeedbackSource, FeedbackSourceType,
    FeedbackType, FeedbackUpdate, HeuristicEvaluator, LlmJudgeConfig, ScoreType,
    StructuredEvaluator,
};
pub use graph::{Graph, GraphEdge, GraphNode, GraphNodeData, GraphSummary};
pub use graph_client::GraphClient;
pub use integrations::{GitHubIntegration, GitHubRepository, IntegrationClient};
pub use organization::{Organization, Workspace};
pub use playground_settings::{
    ListPlaygroundSettingsParams, PlaygroundSavedOptions, PlaygroundSettingsCreateRequest,
    PlaygroundSettingsResponse, PlaygroundSettingsUpdateRequest,
};
pub use projects::{
    ListProjectsParams, Project, ProjectCreate, ProjectSortColumn, ProjectUpdate, TraceTier,
};
pub use prompts::{
    CommitManifestResponse, CommitRequest, CommitResponse, LcJson, MessagePromptTemplateKwargs,
    Prompt, PromptClient, PromptTemplateKwargs, StructuredOutputKwargs, StructuredPrompt,
    Visibility,
};
pub use runs::{Cursors, QueryRunsRequest, QueryRunsResponse, Run, RunDateOrder, RunType};
pub use secrets::{SecretKey, SecretUpsert};
