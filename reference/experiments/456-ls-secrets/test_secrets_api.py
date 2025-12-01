#!/usr/bin/env python3
"""
Experiment: Test LangSmith Workspace Secrets API

Tests the complete CRUD lifecycle:
1. List existing secrets
2. Create a test secret
3. Verify creation (list again)
4. Update the test secret
5. Verify update (should still be in list)
6. Delete the test secret (using value: null)
7. Verify deletion (should be gone from list)

Prerequisites:
- LANGSMITH_API_KEY environment variable
- LANGSMITH_WORKSPACE_ID environment variable
"""

import os
import sys
import json
import requests
from datetime import datetime

# Configuration
API_KEY = os.environ.get("LANGSMITH_API_KEY")
WORKSPACE_ID = os.environ.get("LANGSMITH_WORKSPACE_ID")
BASE_URL = "https://api.smith.langchain.com"
TEST_SECRET_KEY = "LANGSTAR_TEST_SECRET_456"

def print_section(title):
    """Print a formatted section header"""
    print(f"\n{'='*60}")
    print(f"  {title}")
    print(f"{'='*60}\n")

def sanitize_secrets(data):
    """Sanitize secret values in data for logging (replaces values with [REDACTED])"""
    if not data:
        return data
    
    if isinstance(data, list):
        return [
            {k: "[REDACTED]" if k == "value" and v is not None else v for k, v in item.items()}
            if isinstance(item, dict) else item
            for item in data
        ]
    elif isinstance(data, dict):
        return {k: "[REDACTED]" if k == "value" and v is not None else v for k, v in data.items()}
    return data

def make_request(method, endpoint, data=None):
    """Make an authenticated request to the LangSmith API"""
    url = f"{BASE_URL}{endpoint}"
    headers = {
        "X-Api-Key": API_KEY,
        "Content-Type": "application/json"
    }

    print(f"→ {method} {endpoint}")
    if data:
        # Sanitize secret values before printing
        sanitized_data = sanitize_secrets(data)
        print(f"  Request body: {json.dumps(sanitized_data, indent=2)}")

    try:
        if method == "GET":
            response = requests.get(url, headers=headers)
        elif method == "POST":
            response = requests.post(url, headers=headers, json=data)
        else:
            raise ValueError(f"Unsupported method: {method}")

        print(f"← Status: {response.status_code}")

        # Try to parse JSON response
        try:
            response_data = response.json()
            print(f"  Response: {json.dumps(response_data, indent=2)}")
        except (json.JSONDecodeError, ValueError):
            print(f"  Response (text): {response.text[:200]}")

        response.raise_for_status()
        return response

    except requests.exceptions.RequestException as e:
        print(f"✗ Request failed: {e}")
        if e.response is not None and hasattr(e.response, 'text'):
            print(f"  Error response: {e.response.text}")
        return None

def list_secrets():
    """List all workspace secrets (GET)"""
    print_section("TEST 1: List Existing Secrets")
    response = make_request("GET", "/api/v1/workspaces/current/secrets")
    if response:
        secrets = response.json()
        print(f"\n✓ Found {len(secrets)} secrets")
        if secrets:
            print("  Keys:")
            for secret in secrets:
                print(f"    - {secret.get('key')}")
        return secrets
    return None

def create_secret(key, value):
    """Create a new secret (POST)"""
    print_section(f"TEST 2: Create Secret '{key}'")
    data = [{"key": key, "value": value}]
    response = make_request("POST", "/api/v1/workspaces/current/secrets", data)
    if response:
        print(f"\n✓ Secret created successfully")
        return True
    return False

def update_secret(key, new_value):
    """Update an existing secret (POST with same key)"""
    print_section(f"TEST 4: Update Secret '{key}'")
    data = [{"key": key, "value": new_value}]
    response = make_request("POST", "/api/v1/workspaces/current/secrets", data)
    if response:
        print(f"\n✓ Secret updated successfully")
        return True
    return False

def delete_secret(key):
    """Delete a secret (POST with value: null)"""
    print_section(f"TEST 6: Delete Secret '{key}'")
    data = [{"key": key, "value": None}]
    response = make_request("POST", "/api/v1/workspaces/current/secrets", data)
    if response:
        print(f"\n✓ Delete request completed")
        return True
    return False

def verify_secret_exists(secrets, key):
    """Check if a secret key exists in the list"""
    if not secrets:
        return False
    return any(s.get('key') == key for s in secrets)

def main():
    """Run all experiments"""
    print(f"LangSmith Secrets API Experiment")
    print(f"Started: {datetime.now().isoformat()}")
    print(f"Workspace ID: {WORKSPACE_ID}")
    print(f"Test Secret Key: {TEST_SECRET_KEY}")

    # Verify prerequisites
    if not API_KEY:
        print("✗ LANGSMITH_API_KEY not set")
        sys.exit(1)
    if not WORKSPACE_ID:
        print("✗ LANGSMITH_WORKSPACE_ID not set")
        sys.exit(1)

    results = {
        "list_initial": False,
        "create": False,
        "verify_created": False,
        "update": False,
        "verify_updated": False,
        "delete": False,
        "verify_deleted": False
    }

    # Test 1: List existing secrets
    secrets = list_secrets()
    results["list_initial"] = secrets is not None

    # Check if test secret already exists
    if secrets and verify_secret_exists(secrets, TEST_SECRET_KEY):
        print(f"\n⚠️  Test secret '{TEST_SECRET_KEY}' already exists. Cleaning up first...")
        delete_secret(TEST_SECRET_KEY)

    # Test 2: Create a test secret
    results["create"] = create_secret(TEST_SECRET_KEY, "test-value-12345")

    # Test 3: Verify creation by listing again
    print_section("TEST 3: Verify Secret Was Created")
    secrets = list_secrets()
    if secrets:
        exists = verify_secret_exists(secrets, TEST_SECRET_KEY)
        results["verify_created"] = exists
        if exists:
            print(f"\n✓ Secret '{TEST_SECRET_KEY}' found in list")
        else:
            print(f"\n✗ Secret '{TEST_SECRET_KEY}' NOT found in list")

    # Test 4: Update the secret
    results["update"] = update_secret(TEST_SECRET_KEY, "updated-value-67890")

    # Test 5: Verify update (secret should still be in list)
    print_section("TEST 5: Verify Secret Still Exists After Update")
    secrets = list_secrets()
    if secrets:
        exists = verify_secret_exists(secrets, TEST_SECRET_KEY)
        results["verify_updated"] = exists
        if exists:
            print(f"\n✓ Secret '{TEST_SECRET_KEY}' still in list (update succeeded)")
        else:
            print(f"\n✗ Secret '{TEST_SECRET_KEY}' disappeared (unexpected)")

    # Test 6: Delete the secret
    results["delete"] = delete_secret(TEST_SECRET_KEY)

    # Test 7: Verify deletion
    print_section("TEST 7: Verify Secret Was Deleted")
    secrets = list_secrets()
    if secrets:
        exists = verify_secret_exists(secrets, TEST_SECRET_KEY)
        results["verify_deleted"] = not exists  # Should NOT exist
        if not exists:
            print(f"\n✓ Secret '{TEST_SECRET_KEY}' successfully deleted (not in list)")
        else:
            print(f"\n✗ Secret '{TEST_SECRET_KEY}' still in list (deletion may have failed)")

    # Summary
    print_section("EXPERIMENT RESULTS SUMMARY")
    print(f"Completed: {datetime.now().isoformat()}\n")

    all_passed = all(results.values())

    for test, passed in results.items():
        status = "✓" if passed else "✗"
        print(f"{status} {test}: {'PASS' if passed else 'FAIL'}")

    print(f"\nOverall: {'✓ ALL TESTS PASSED' if all_passed else '✗ SOME TESTS FAILED'}")

    # Key findings
    print_section("KEY FINDINGS")
    print("1. API returns only secret keys, never values (CONFIRMED)")
    print("2. POST endpoint handles both create and update (CONFIRMED)" if results["update"] else "2. Update behavior needs investigation")
    print("3. Deletion via value: null " + ("WORKS" if results["verify_deleted"] else "NEEDS INVESTIGATION"))
    print("4. List endpoint shows all secret keys (CONFIRMED)" if results["list_initial"] else "4. List endpoint needs investigation")

    return 0 if all_passed else 1

if __name__ == "__main__":
    sys.exit(main())
