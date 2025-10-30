use langstar_sdk::{AuthConfig, LangchainClient};
use reqwest::StatusCode;

/// Experimental integration tests for organization and workspace ID header behavior.
///
/// This test suite explores how the LangSmith API responds to different combinations
/// of organization and workspace scoping headers:
/// - `x-organization-id`: For organization-level scoping
/// - `X-Tenant-Id`: For workspace-level scoping
///
/// **Prerequisites:**
/// 1. Valid LANGSMITH_API_KEY with appropriate permissions
/// 2. Access to at least one organization
/// 3. (Optional) Access to workspaces for workspace testing
///
/// Run with: cargo test --test org_workspace_experiments -- --ignored --nocapture

/// Helper to make a raw HTTP request with custom headers for experimentation
async fn make_request_with_headers(
    client: &LangchainClient,
    path: &str,
    org_id: Option<&str>,
    workspace_id: Option<&str>,
) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
    let auth = AuthConfig::from_env()?;
    let api_key = auth
        .require_langsmith_key()
        .map_err(|e| format!("Missing API key: {}", e))?;
    let url = format!("https://api.smith.langchain.com{}", path);

    let mut request = client.http_client().get(&url).header("x-api-key", api_key);

    if let Some(org) = org_id {
        println!("  Adding x-organization-id: {}", org);
        request = request.header("x-organization-id", org);
    }

    if let Some(ws) = workspace_id {
        println!("  Adding X-Tenant-Id: {}", ws);
        request = request.header("X-Tenant-Id", ws);
    }

    Ok(request.send().await?)
}

/// Test 1: Request with x-organization-id header only
///
/// This tests whether the API accepts organization-level scoping via the
/// x-organization-id header.
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_org_id_header_only() {
    println!("\n=== Test 1: x-organization-id header only ===");

    let auth = AuthConfig::from_env().expect("LANGSMITH_API_KEY must be set");
    let client = LangchainClient::new(auth).expect("Failed to create client");

    // Fetch current organization to get a valid org ID
    println!("Fetching current organization...");
    let org = client
        .get_current_organization()
        .await
        .expect("Failed to get current organization");

    let org_id = org
        .id
        .as_ref()
        .expect("Organization must have an ID")
        .as_str();
    println!("Organization ID: {}", org_id);
    println!(
        "Organization name: {}",
        org.display_name.unwrap_or_default()
    );

    // Make a request to list prompts with only org ID header
    println!("\nMaking request with x-organization-id only...");
    let response = make_request_with_headers(&client, "/api/v1/repos/?limit=5", Some(org_id), None)
        .await
        .expect("Request failed");

    println!("Response status: {}", response.status());
    println!("Response headers: {:#?}", response.headers());

    if response.status().is_success() {
        let body = response.text().await.unwrap();
        println!("Response body (first 500 chars): {}", &body[..body.len().min(500)]);
        println!("\n✓ SUCCESS: API accepts x-organization-id header");
    } else {
        let body = response.text().await.unwrap();
        println!("Error response: {}", body);
        println!("\n✗ FAILED: API rejected x-organization-id header");
    }
}

/// Test 2: Request with X-Tenant-Id header only
///
/// This tests whether the API accepts workspace-level scoping via the
/// X-Tenant-Id header. Since we may not have a workspace ID yet, this test
/// documents the behavior when provided.
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_workspace_id_header_only() {
    println!("\n=== Test 2: X-Tenant-Id header only ===");

    let auth = AuthConfig::from_env().expect("LANGSMITH_API_KEY must be set");
    let client = LangchainClient::new(auth).expect("Failed to create client");

    // For this test, we'll need a workspace ID
    // This could come from:
    // 1. Environment variable LANGSMITH_WORKSPACE_ID
    // 2. API call to list workspaces (endpoint to be determined)
    // 3. Manual configuration for testing

    let workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID").ok();

    if workspace_id.is_none() {
        println!("⚠ SKIPPED: LANGSMITH_WORKSPACE_ID not set");
        println!("  Set this environment variable to test workspace scoping");
        println!("  Example: export LANGSMITH_WORKSPACE_ID=<your-workspace-id>");
        return;
    }

    let workspace_id = workspace_id.unwrap();
    println!("Workspace ID: {}", workspace_id);

    // Make a request to list prompts with only workspace ID header
    println!("\nMaking request with X-Tenant-Id only...");
    let response =
        make_request_with_headers(&client, "/api/v1/repos/?limit=5", None, Some(&workspace_id))
            .await
            .expect("Request failed");

    println!("Response status: {}", response.status());
    println!("Response headers: {:#?}", response.headers());

    if response.status().is_success() {
        let body = response.text().await.unwrap();
        println!("Response body (first 500 chars): {}", &body[..body.len().min(500)]);
        println!("\n✓ SUCCESS: API accepts X-Tenant-Id header");
    } else {
        let body = response.text().await.unwrap();
        println!("Error response: {}", body);
        println!("\n✗ FAILED: API rejected X-Tenant-Id header");
    }
}

/// Test 3: Request with both headers (x-organization-id and X-Tenant-Id)
///
/// This tests how the API handles both headers being present simultaneously.
/// Questions to answer:
/// - Does the API accept both headers?
/// - Which header takes precedence?
/// - Does the API validate that the workspace belongs to the organization?
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_both_headers() {
    println!("\n=== Test 3: Both x-organization-id and X-Tenant-Id ===");

    let auth = AuthConfig::from_env().expect("LANGSMITH_API_KEY must be set");
    let client = LangchainClient::new(auth).expect("Failed to create client");

    // Get organization ID
    println!("Fetching current organization...");
    let org = client
        .get_current_organization()
        .await
        .expect("Failed to get current organization");

    let org_id = org
        .id
        .as_ref()
        .expect("Organization must have an ID")
        .as_str();
    println!("Organization ID: {}", org_id);

    // Get workspace ID from environment
    let workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID").ok();

    if workspace_id.is_none() {
        println!("⚠ SKIPPED: LANGSMITH_WORKSPACE_ID not set");
        println!("  Set this environment variable to test workspace scoping");
        return;
    }

    let workspace_id = workspace_id.unwrap();
    println!("Workspace ID: {}", workspace_id);

    // Make a request with both headers
    println!("\nMaking request with both headers...");
    let response = make_request_with_headers(
        &client,
        "/api/v1/repos/?limit=5",
        Some(org_id),
        Some(&workspace_id),
    )
    .await
    .expect("Request failed");

    println!("Response status: {}", response.status());
    println!("Response headers: {:#?}", response.headers());

    if response.status().is_success() {
        let body = response.text().await.unwrap();
        println!("Response body (first 500 chars): {}", &body[..body.len().min(500)]);
        println!("\n✓ SUCCESS: API accepts both headers");
        println!("  NOTE: Need to determine which header takes precedence");
    } else {
        let body = response.text().await.unwrap();
        println!("Error response: {}", body);
        println!("\n✗ FAILED: API rejected the header combination");
    }
}

/// Test 4: Request with mismatched workspace and organization IDs
///
/// This tests what happens when providing a workspace ID that doesn't belong
/// to the specified organization. This helps us understand:
/// - Does the API validate this relationship?
/// - What error message/code does it return?
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_mismatched_ids() {
    println!("\n=== Test 4: Mismatched workspace and organization IDs ===");

    let auth = AuthConfig::from_env().expect("LANGSMITH_API_KEY must be set");
    let client = LangchainClient::new(auth).expect("Failed to create client");

    // Get organization ID
    println!("Fetching current organization...");
    let org = client
        .get_current_organization()
        .await
        .expect("Failed to get current organization");

    let org_id = org
        .id
        .as_ref()
        .expect("Organization must have an ID")
        .as_str();
    println!("Organization ID: {}", org_id);

    // Use a fake/mismatched workspace ID
    let fake_workspace_id = "00000000-0000-0000-0000-000000000000";
    println!("Using fake workspace ID: {}", fake_workspace_id);

    // Make a request with mismatched IDs
    println!("\nMaking request with mismatched IDs...");
    let response = make_request_with_headers(
        &client,
        "/api/v1/repos/?limit=5",
        Some(org_id),
        Some(fake_workspace_id),
    )
    .await
    .expect("Request failed");

    let status = response.status();
    println!("Response status: {}", status);
    println!("Response headers: {:#?}", response.headers());

    let body = response.text().await.unwrap();
    println!("Response body: {}", body);

    match status {
        StatusCode::BAD_REQUEST | StatusCode::FORBIDDEN | StatusCode::NOT_FOUND => {
            println!("\n✓ EXPECTED: API rejected mismatched IDs");
            println!("  Error handling appears to be working correctly");
        }
        StatusCode::OK => {
            println!("\n⚠ UNEXPECTED: API accepted mismatched IDs");
            println!("  This suggests validation may not be enforced");
        }
        _ => {
            println!("\n? UNCLEAR: Unexpected status code");
        }
    }
}

/// Test 5: Request with no scoping headers (baseline)
///
/// This establishes a baseline for comparison by making a request without
/// any organization or workspace scoping headers.
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_no_headers_baseline() {
    println!("\n=== Test 5: No scoping headers (baseline) ===");

    let auth = AuthConfig::from_env().expect("LANGSMITH_API_KEY must be set");
    let client = LangchainClient::new(auth).expect("Failed to create client");

    // Make a request without any scoping headers
    println!("Making request without scoping headers...");
    let response = make_request_with_headers(&client, "/api/v1/repos/?limit=5", None, None)
        .await
        .expect("Request failed");

    println!("Response status: {}", response.status());

    if response.status().is_success() {
        let body = response.text().await.unwrap();
        println!("Response body (first 500 chars): {}", &body[..body.len().min(500)]);
        println!("\n✓ SUCCESS: Baseline request works");
        println!("  This shows what results are returned without scoping");
    } else {
        let body = response.text().await.unwrap();
        println!("Error response: {}", body);
        println!("\n✗ FAILED: Even baseline request failed");
    }
}

// NOTE: To run all experiments at once, use:
// cargo test --test org_workspace_experiments -- --ignored --nocapture
//
// This will run all tests in this file and display detailed output.
//
// Key Questions to Answer:
//   1. Can both headers be used simultaneously?
//   2. How does the API handle workspace ID + mismatched org ID?
//   3. Should we auto-fetch org ID from workspace ID to validate?
//
// Next steps after running experiments:
//   - Document findings in GitHub issue #66
//   - Research LangChain documentation for official guidance
//   - Proceed with Phase 2 implementation based on findings
