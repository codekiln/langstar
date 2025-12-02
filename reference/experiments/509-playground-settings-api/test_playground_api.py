#!/usr/bin/env python3
"""
Experiment: Test LangSmith Playground Settings API

Investigates JSON decoding errors in playground-settings integration tests:
- Error: "error decoding response body"
- Location: "premature end of input, line: 1, column: 348"

Tests the complete CRUD lifecycle:
1. Create a test playground setting
2. Verify creation (GET by ID)
3. Update the test setting
4. Verify update
5. Delete the test setting
6. Test GET on non-existent ID (to reproduce error)

Prerequisites:
- LANGSMITH_API_KEY environment variable
- LANGSMITH_ORGANIZATION_ID environment variable (or auto-discovery)
"""

import os
import sys
import json
import requests
import uuid
from datetime import datetime

# Configuration
API_KEY = os.environ.get("LANGSMITH_API_KEY")
ORG_ID = os.environ.get("LANGSMITH_ORGANIZATION_ID")
BASE_URL = "https://api.smith.langchain.com"
TEST_NAME_PREFIX = "LANGSTAR_TEST_PLAYGROUND_509"

def print_section(title):
    """Print a formatted section header"""
    print(f"\n{'='*70}")
    print(f"  {title}")
    print(f"{'='*70}\n")

def print_raw_response(response):
    """Print raw response details for debugging truncation"""
    print(f"→ Status: {response.status_code}")
    print(f"→ Headers:")
    for key, value in response.headers.items():
        if key.lower() in ['content-type', 'content-length', 'transfer-encoding']:
            print(f"  {key}: {value}")

    # Get raw body
    raw_body = response.content
    print(f"→ Raw body length: {len(raw_body)} bytes")
    print(f"→ Raw body (first 500 chars): {raw_body[:500]}")

    # Try to decode
    try:
        decoded = raw_body.decode('utf-8')
        print(f"→ Decoded length: {len(decoded)} chars")
        print(f"→ Decoded content: {decoded}")
    except Exception as e:
        print(f"→ Decode error: {e}")

def make_request(method, endpoint, data=None, expect_error=False):
    """Make an authenticated request to the LangSmith API"""
    url = f"{BASE_URL}{endpoint}"
    headers = {
        "X-Api-Key": API_KEY,
        "Content-Type": "application/json"
    }

    print(f"\n→ {method} {endpoint}")
    if data:
        print(f"  Request body: {json.dumps(data, indent=2)}")

    try:
        if method == "GET":
            response = requests.get(url, headers=headers)
        elif method == "POST":
            response = requests.post(url, headers=headers, json=data)
        elif method == "PATCH":
            response = requests.patch(url, headers=headers, json=data)
        elif method == "DELETE":
            response = requests.delete(url, headers=headers)
        else:
            raise ValueError(f"Unsupported method: {method}")

        print_raw_response(response)

        # Try to parse JSON response
        try:
            response_data = response.json()
            print(f"✓ JSON parsed successfully")
            print(f"  Parsed response: {json.dumps(response_data, indent=2)}")
        except (json.JSONDecodeError, ValueError) as e:
            print(f"✗ JSON decode failed: {e}")
            print(f"  This is the error we're investigating!")
            if not expect_error:
                return None

        if not expect_error:
            response.raise_for_status()

        return response

    except requests.exceptions.RequestException as e:
        print(f"✗ Request failed: {e}")
        if hasattr(e, 'response') and e.response is not None:
            print_raw_response(e.response)
        return None

def create_test_config():
    """Create a test playground settings configuration"""
    test_config = {
        "name": f"{TEST_NAME_PREFIX}_{uuid.uuid4().hex[:8]}",
        "description": "Test configuration for API truncation investigation",
        "settings": {
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
            "kwargs": {
                "model": "claude-3-5-sonnet-20241022",
                "temperature": 0.0
            }
        },
        "options": {
            "requests_per_second": 5
        }
    }
    return test_config

def test_create(config):
    """Create a new playground setting (POST)"""
    print_section("TEST 1: Create Playground Setting")
    response = make_request("POST", "/api/v1/playground-settings", config)
    if response and response.status_code in (200, 201):
        try:
            data = response.json()
            setting_id = data.get("id")
            print(f"\n✓ Playground setting created successfully")
            print(f"  ID: {setting_id}")
            return setting_id
        except Exception as e:
            print(f"✗ Failed to parse create response: {e}")
            return None
    return None

def test_get(setting_id):
    """Get a playground setting by ID (GET)"""
    print_section(f"TEST 2: Get Playground Setting by ID")
    response = make_request("GET", f"/api/v1/playground-settings/{setting_id}")
    if response and response.status_code == 200:
        print(f"\n✓ Successfully retrieved setting")
        return True
    return False

def test_update(setting_id):
    """Update a playground setting (PATCH)"""
    print_section(f"TEST 3: Update Playground Setting")
    update_data = {
        "name": f"{TEST_NAME_PREFIX}_UPDATED_{uuid.uuid4().hex[:4]}"
    }
    response = make_request("PATCH", f"/api/v1/playground-settings/{setting_id}", update_data)
    if response and response.status_code == 200:
        print(f"\n✓ Successfully updated setting")
        return True
    return False

def test_delete(setting_id):
    """Delete a playground setting (DELETE)"""
    print_section(f"TEST 4: Delete Playground Setting")
    response = make_request("DELETE", f"/api/v1/playground-settings/{setting_id}")
    if response and response.status_code in (200, 204):
        print(f"\n✓ Successfully deleted setting")
        return True
    return False

def test_get_nonexistent():
    """Test GET on non-existent ID (to reproduce the error)"""
    print_section("TEST 5: Get Non-Existent Playground Setting (Reproduce Error)")
    fake_id = "00000000-0000-0000-0000-000000000000"
    print(f"Attempting to GET fake ID: {fake_id}")
    print(f"Expected: 404 error or error message")
    print(f"Investigating: Whether response is truncated JSON")

    response = make_request("GET", f"/api/v1/playground-settings/{fake_id}", expect_error=True)

    if response:
        print(f"\n→ Response analysis:")
        print(f"  Status code: {response.status_code}")
        print(f"  Expected status: 404")
        print(f"  Match: {'✓' if response.status_code == 404 else '✗'}")

    return True  # Always return True since we expect this to "fail"

def list_playground_settings():
    """List all playground settings"""
    print_section("TEST 6: List All Playground Settings")
    response = make_request("GET", "/api/v1/playground-settings")
    if response and response.status_code == 200:
        try:
            settings = response.json()
            print(f"\n✓ Found {len(settings)} playground settings")
            return settings
        except Exception as e:
            print(f"✗ Failed to parse list response: {e}")
    return None

def cleanup_test_configs():
    """Clean up any leftover test configurations"""
    print_section("CLEANUP: Remove Test Configurations")
    settings = list_playground_settings()
    if settings:
        for setting in settings:
            if setting.get("name", "").startswith(TEST_NAME_PREFIX):
                setting_id = setting.get("id")
                print(f"→ Deleting test config: {setting.get('name')} ({setting_id})")
                test_delete(setting_id)

def main():
    """Run all experiments"""
    print(f"LangSmith Playground Settings API Experiment")
    print(f"Started: {datetime.now().isoformat()}")
    print(f"Target: Investigate 'error decoding response body' at line 1, column 348")

    # Verify prerequisites
    if not API_KEY:
        print("✗ LANGSMITH_API_KEY not set")
        sys.exit(1)

    print(f"✓ Using API key: {API_KEY[:20]}...")
    if ORG_ID:
        print(f"✓ Using Organization ID: {ORG_ID}")

    # Track results
    results = {
        "create": False,
        "get": False,
        "update": False,
        "delete": False,
        "get_nonexistent": False,
        "list": False
    }

    # Clean up any existing test configs first
    cleanup_test_configs()

    # Test 1: Create
    test_config = create_test_config()
    setting_id = test_create(test_config)
    results["create"] = setting_id is not None

    if setting_id:
        # Test 2: Get
        results["get"] = test_get(setting_id)

        # Test 3: Update
        results["update"] = test_update(setting_id)

        # Test 4: Delete
        results["delete"] = test_delete(setting_id)

    # Test 5: Get non-existent (reproduce error)
    results["get_nonexistent"] = test_get_nonexistent()

    # Test 6: List
    settings = list_playground_settings()
    results["list"] = settings is not None

    # Summary
    print_section("EXPERIMENT RESULTS SUMMARY")
    print(f"Completed: {datetime.now().isoformat()}\n")

    for test, passed in results.items():
        status = "✓" if passed else "✗"
        print(f"{status} {test}: {'PASS' if passed else 'FAIL'}")

    all_passed = all(results.values())
    print(f"\nOverall: {'✓ ALL TESTS PASSED' if all_passed else '✗ SOME TESTS FAILED'}")

    # Key findings
    print_section("KEY FINDINGS")
    print("Based on the raw response analysis above:")
    print("1. Check if response bodies are truncated mid-JSON")
    print("2. Compare Content-Length header vs actual body length")
    print("3. Identify at what byte/char position truncation occurs")
    print("4. Determine if truncation is consistent (always at column 348?)")
    print("5. Compare successful vs failed response handling")
    print("\nLook for:")
    print("- Incomplete JSON in error responses")
    print("- Missing closing braces/brackets")
    print("- Content-Length mismatches")
    print("- Difference between 200 OK and 404 responses")

    return 0 if all_passed else 1

if __name__ == "__main__":
    sys.exit(main())
