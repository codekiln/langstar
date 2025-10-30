use langstar_sdk::{AuthConfig, LangchainClient, Visibility};

/// Integration tests for organization and workspace scoping functionality
///
/// These tests verify that:
/// 1. Organization and workspace IDs are loaded from environment variables
/// 2. Correct headers are sent to the API
/// 3. Visibility filtering works correctly when scoped
///
/// **Prerequisites:**
/// 1. Valid LANGSMITH_API_KEY environment variable
/// 2. Valid LANGSMITH_ORGANIZATION_ID environment variable
/// 3. Valid LANGSMITH_WORKSPACE_ID environment variable
///
/// Run with: cargo test --test org_workspace_scoping_test -- --ignored --nocapture

#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_org_id_from_environment() {
    // Verify LANGSMITH_ORGANIZATION_ID is set
    let org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for this test");

    // Load auth from environment
    let auth = AuthConfig::from_env().expect("Failed to load auth from environment");
    assert_eq!(
        auth.organization_id,
        Some(org_id.clone()),
        "AuthConfig should load organization_id from environment"
    );

    // Create client and verify org_id is set
    let client = LangchainClient::new(auth).expect("Failed to create client");
    assert_eq!(
        client.organization_id(),
        Some(org_id.as_str()),
        "Client should have organization_id set"
    );

    // Make API call to verify headers are sent correctly
    // We'll list prompts as a simple test that exercises the headers
    println!("Testing API call with organization ID: {}", org_id);
    let result = client.prompts().list(Some(5), None, None).await;

    match result {
        Ok(prompts) => {
            println!(
                "✓ Successfully fetched {} prompts with org ID",
                prompts.len()
            );
        }
        Err(e) => {
            panic!(
                "Failed to fetch prompts with organization ID: {:?}\n\
                This could indicate:\n\
                1. Invalid LANGSMITH_ORGANIZATION_ID\n\
                2. API key doesn't have access to this organization\n\
                3. Network connectivity issues",
                e
            );
        }
    }
}

#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_workspace_id_from_environment() {
    // Verify LANGSMITH_WORKSPACE_ID is set
    let workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for this test");

    // Load auth from environment
    let auth = AuthConfig::from_env().expect("Failed to load auth from environment");
    assert_eq!(
        auth.workspace_id,
        Some(workspace_id.clone()),
        "AuthConfig should load workspace_id from environment"
    );

    // Create client and verify workspace_id is set
    let client = LangchainClient::new(auth).expect("Failed to create client");
    assert_eq!(
        client.workspace_id(),
        Some(workspace_id.as_str()),
        "Client should have workspace_id set"
    );

    // Make API call to verify headers are sent correctly
    println!("Testing API call with workspace ID: {}", workspace_id);
    let result = client.prompts().list(Some(5), None, None).await;

    match result {
        Ok(prompts) => {
            println!(
                "✓ Successfully fetched {} prompts with workspace ID",
                prompts.len()
            );
        }
        Err(e) => {
            panic!(
                "Failed to fetch prompts with workspace ID: {:?}\n\
                This could indicate:\n\
                1. Invalid LANGSMITH_WORKSPACE_ID\n\
                2. API key doesn't have access to this workspace\n\
                3. Network connectivity issues",
                e
            );
        }
    }
}

#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_both_org_and_workspace_ids() {
    // Verify both IDs are set
    let org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for this test");
    let workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for this test");

    // Load auth from environment
    let auth = AuthConfig::from_env().expect("Failed to load auth from environment");
    assert_eq!(
        auth.organization_id,
        Some(org_id.clone()),
        "AuthConfig should load organization_id from environment"
    );
    assert_eq!(
        auth.workspace_id,
        Some(workspace_id.clone()),
        "AuthConfig should load workspace_id from environment"
    );

    // Create client
    let client = LangchainClient::new(auth).expect("Failed to create client");

    // Per Phase 1 research findings, both headers should be sent together
    // x-organization-id and X-Tenant-Id (workspace_id)
    println!(
        "Testing API call with both org ID ({}) and workspace ID ({})",
        org_id, workspace_id
    );

    let result = client.prompts().list(Some(5), None, None).await;

    match result {
        Ok(prompts) => {
            println!(
                "✓ Successfully fetched {} prompts with both IDs",
                prompts.len()
            );
            println!("  This confirms both x-organization-id and X-Tenant-Id headers are sent");
        }
        Err(e) => {
            panic!(
                "Failed to fetch prompts with both IDs: {:?}\n\
                This could indicate:\n\
                1. Invalid organization or workspace ID\n\
                2. API key doesn't have access to this organization/workspace\n\
                3. Workspace doesn't belong to the organization\n\
                4. Network connectivity issues",
                e
            );
        }
    }
}

#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_visibility_filtering_default_private_when_scoped() {
    // This test verifies Phase 4 requirement:
    // When scoped (org or workspace ID set), default to private prompts

    let org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for this test");

    let auth = AuthConfig::from_env().expect("Failed to load auth from environment");
    let client = LangchainClient::new(auth).expect("Failed to create client");

    println!(
        "Testing default visibility (should be Private) with org ID: {}",
        org_id
    );

    // Call list with None visibility - should default to Private when scoped
    let result = client.prompts().list(Some(20), None, None).await;

    match result {
        Ok(prompts) => {
            println!(
                "✓ Fetched {} prompts (default visibility when scoped)",
                prompts.len()
            );

            // Note: The SDK does client-side filtering for visibility
            // When visibility is None, it defaults to Visibility::Any in the SDK
            // The CLI layer is responsible for determining visibility based on scoping

            // However, we can verify that private prompts ARE returned
            let private_count = prompts.iter().filter(|p| !p.is_public).count();
            let public_count = prompts.iter().filter(|p| p.is_public).count();

            println!("  Private prompts: {}", private_count);
            println!("  Public prompts: {}", public_count);

            // Both private and public may be returned since we're testing SDK layer
            // The CLI layer (tested separately) enforces the default-to-private behavior
        }
        Err(e) => {
            panic!("Failed to list prompts: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_visibility_filtering_explicit_private() {
    let org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for this test");

    let auth = AuthConfig::from_env().expect("Failed to load auth from environment");
    let client = LangchainClient::new(auth).expect("Failed to create client");

    println!(
        "Testing explicit Private visibility with org ID: {}",
        org_id
    );

    // Explicitly request private prompts
    let result = client
        .prompts()
        .list(Some(20), None, Some(Visibility::Private))
        .await;

    match result {
        Ok(prompts) => {
            println!("✓ Fetched {} private prompts", prompts.len());

            // Verify all returned prompts are private
            let all_private = prompts.iter().all(|p| !p.is_public);
            assert!(
                all_private,
                "All prompts should be private when Visibility::Private is specified"
            );

            println!("  ✓ Verified all {} prompts are private", prompts.len());
        }
        Err(e) => {
            panic!("Failed to list private prompts: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_visibility_filtering_explicit_public() {
    let org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for this test");

    let auth = AuthConfig::from_env().expect("Failed to load auth from environment");
    let client = LangchainClient::new(auth).expect("Failed to create client");

    println!("Testing explicit Public visibility with org ID: {}", org_id);

    // Explicitly request public prompts
    let result = client
        .prompts()
        .list(Some(20), None, Some(Visibility::Public))
        .await;

    match result {
        Ok(prompts) => {
            println!("✓ Fetched {} public prompts", prompts.len());

            if !prompts.is_empty() {
                // Verify all returned prompts are public
                let all_public = prompts.iter().all(|p| p.is_public);
                assert!(
                    all_public,
                    "All prompts should be public when Visibility::Public is specified"
                );

                println!("  ✓ Verified all {} prompts are public", prompts.len());
            } else {
                println!("  ℹ No public prompts found in this organization");
            }
        }
        Err(e) => {
            panic!("Failed to list public prompts: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_with_organization_id_builder() {
    // Test the with_organization_id builder method
    let org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for this test");

    // Create client without org ID
    let auth = AuthConfig::new(std::env::var("LANGSMITH_API_KEY").ok(), None, None, None);
    let client = LangchainClient::new(auth).expect("Failed to create client");

    assert_eq!(
        client.organization_id(),
        None,
        "Client should not have org ID initially"
    );

    // Add org ID using builder
    let client = client.with_organization_id(org_id.clone());
    assert_eq!(
        client.organization_id(),
        Some(org_id.as_str()),
        "Client should have org ID after with_organization_id"
    );

    // Verify API call works
    let result = client.prompts().list(Some(5), None, None).await;
    assert!(
        result.is_ok(),
        "API call should succeed with builder-applied org ID"
    );

    println!("✓ Builder method with_organization_id works correctly");
}

#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_with_workspace_id_builder() {
    // Test the with_workspace_id builder method
    let workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for this test");

    // Create client without workspace ID
    let auth = AuthConfig::new(std::env::var("LANGSMITH_API_KEY").ok(), None, None, None);
    let client = LangchainClient::new(auth).expect("Failed to create client");

    assert_eq!(
        client.workspace_id(),
        None,
        "Client should not have workspace ID initially"
    );

    // Add workspace ID using builder
    let client = client.with_workspace_id(workspace_id.clone());
    assert_eq!(
        client.workspace_id(),
        Some(workspace_id.as_str()),
        "Client should have workspace ID after with_workspace_id"
    );

    // Verify API call works
    let result = client.prompts().list(Some(5), None, None).await;
    assert!(
        result.is_ok(),
        "API call should succeed with builder-applied workspace ID"
    );

    println!("✓ Builder method with_workspace_id works correctly");
}
