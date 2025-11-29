#!/usr/bin/env python3
"""
Structured Output Prompts Experiment
Issue: https://github.com/codekiln/langstar/issues/398

This script experiments with structured output prompts in LangSmith to understand:
1. How to create and push structured output prompts
2. The manifest structure stored in LangSmith
3. The transform logic between SDK and API representations
"""

import argparse
import json
import os
import sys
from typing import Any

# Check for required dependencies
try:
    from langsmith import Client
except ImportError:
    print("ERROR: langsmith package not installed. Run: pip install langsmith")
    sys.exit(1)

try:
    from langchain_core.prompts import ChatPromptTemplate
    from langchain_core.prompts.structured import StructuredPrompt
except ImportError:
    print("ERROR: langchain-core package not installed. Run: pip install langchain-core")
    sys.exit(1)

# Environment setup
LANGSMITH_API_KEY = os.getenv("LANGSMITH_API_KEY")
if not LANGSMITH_API_KEY:
    print("ERROR: LANGSMITH_API_KEY environment variable not set")
    sys.exit(1)

# Initialize client
client = Client()


def cmd_create_and_push(args):
    """Create a structured output prompt and push it to LangSmith.

    This demonstrates how to:
    1. Create a ChatPromptTemplate
    2. Convert it to a StructuredPrompt with a JSON schema
    3. Push it to LangSmith
    """
    from pydantic import BaseModel, Field

    # Define the output schema using Pydantic
    class MovieReview(BaseModel):
        """A structured movie review."""
        title: str = Field(description="The movie title")
        rating: int = Field(description="Rating from 1-10", ge=1, le=10)
        summary: str = Field(description="Brief review summary")
        recommended: bool = Field(description="Whether you recommend this movie")

    # Create base prompt template
    base_prompt = ChatPromptTemplate.from_messages([
        ("system", "You are a movie critic. Analyze the movie and provide a structured review."),
        ("human", "Review the movie: {movie_name}")
    ])

    # Create structured prompt with the schema
    # Note: StructuredPrompt wraps the prompt with schema information
    structured_prompt = StructuredPrompt(
        messages=base_prompt.messages,
        schema_=MovieReview,
        method="json_schema"  # Can be "json_schema" or "function_calling"
    )

    # Generate unique prompt name
    prompt_name = args.name or "test-structured-output-prompt-398"

    print(f"Creating structured prompt: {prompt_name}")
    print(f"Schema: {json.dumps(MovieReview.model_json_schema(), indent=2)}")

    # Push to LangSmith
    try:
        url = client.push_prompt(
            prompt_name,
            object=structured_prompt,
            description="Test structured output prompt for issue #398",
            is_public=False,
        )
        print(f"\n✓ Pushed successfully!")
        print(f"  URL: {url}")
        print(f"\nTo examine the manifest, run:")
        print(f"  ./run_test.sh pull --name {prompt_name}")
    except Exception as e:
        print(f"\n✗ Failed to push: {e}")
        sys.exit(1)


def cmd_pull_and_examine(args):
    """Pull a prompt from LangSmith and examine its manifest structure.

    This helps understand how structured output prompts are stored.
    """
    prompt_name = args.name

    print(f"Pulling prompt: {prompt_name}")

    try:
        # Pull without model binding to see raw structure
        prompt = client.pull_prompt(prompt_name, include_model=False)

        print(f"\n=== Prompt Type ===")
        print(f"Type: {type(prompt).__name__}")
        print(f"Module: {type(prompt).__module__}")

        # Try to get the underlying structure
        if hasattr(prompt, "to_json"):
            manifest = prompt.to_json()
            print(f"\n=== Manifest (to_json) ===")
            print(json.dumps(manifest, indent=2, default=str))

        if hasattr(prompt, "dict"):
            prompt_dict = prompt.dict()
            print(f"\n=== Prompt Dict ===")
            print(json.dumps(prompt_dict, indent=2, default=str))

        # Check for structured output specific attributes
        print(f"\n=== Structured Output Attributes ===")
        for attr in ["schema_", "schema", "method", "ls_structured_output_format"]:
            if hasattr(prompt, attr):
                val = getattr(prompt, attr)
                print(f"{attr}: {val}")

        # Get raw commit data via API
        print(f"\n=== Raw Commit Data (API) ===")
        prompts_list = list(client.list_prompts(prompt_identifier=prompt_name))
        if prompts_list:
            prompt_info = prompts_list[0]
            print(f"Prompt ID: {prompt_info.id}")

            # Get the latest commit
            commits = list(client.list_commits(prompt_identifier=prompt_name, limit=1))
            if commits:
                commit = commits[0]
                print(f"\nCommit Hash: {commit.commit_hash}")
                print(f"\nManifest (raw):")
                print(json.dumps(commit.manifest, indent=2, default=str))

    except Exception as e:
        print(f"\n✗ Failed to pull: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


def cmd_raw_api_create(args):
    """Test creating a structured output prompt via raw API call.

    This bypasses SDK transforms to understand the raw API format.
    """
    import requests

    prompt_name = args.name or "test-raw-structured-prompt-398"

    # Construct a manifest that looks like a StructuredPrompt
    manifest = {
        "lc": 1,
        "type": "constructor",
        "id": ["langchain", "prompts", "structured", "StructuredPrompt"],
        "kwargs": {
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
                                "template": "You are a helpful assistant.",
                                "input_variables": [],
                                "template_format": "f-string"
                            }
                        }
                    }
                },
                {
                    "lc": 1,
                    "type": "constructor",
                    "id": ["langchain", "prompts", "chat", "HumanMessagePromptTemplate"],
                    "kwargs": {
                        "prompt": {
                            "lc": 1,
                            "type": "constructor",
                            "id": ["langchain", "prompts", "prompt", "PromptTemplate"],
                            "kwargs": {
                                "template": "{input}",
                                "input_variables": ["input"],
                                "template_format": "f-string"
                            }
                        }
                    }
                }
            ],
            "schema_": {
                "type": "object",
                "title": "Response",
                "properties": {
                    "answer": {"type": "string", "description": "The answer"},
                    "confidence": {"type": "number", "description": "Confidence 0-1"}
                },
                "required": ["answer", "confidence"]
            },
            "method": "json_schema"
        }
    }

    print(f"Creating prompt via raw API: {prompt_name}")
    print(f"\nManifest to send:")
    print(json.dumps(manifest, indent=2))

    # Use the client's session for auth
    api_url = client.api_url
    headers = {
        "X-API-Key": LANGSMITH_API_KEY,
        "Content-Type": "application/json"
    }

    # First, create/get the prompt repo
    try:
        response = requests.post(
            f"{api_url}/repos/",
            headers=headers,
            json={
                "repo_handle": prompt_name,
                "description": "Test raw API structured prompt",
                "is_public": False,
            }
        )
        if response.status_code not in [200, 201, 409]:  # 409 = already exists
            print(f"Failed to create repo: {response.status_code} {response.text}")
            # Continue anyway, repo might already exist

        # Create a commit with the manifest
        response = requests.post(
            f"{api_url}/commits/{prompt_name}/",
            headers=headers,
            json={
                "manifest": manifest,
            }
        )

        if response.status_code in [200, 201]:
            print(f"\n✓ Created successfully via raw API!")
            commit_data = response.json()
            print(f"Response: {json.dumps(commit_data, indent=2)}")
        else:
            print(f"\n✗ Failed: {response.status_code} {response.text}")

    except Exception as e:
        print(f"\n✗ Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


def cmd_list_prompts(args):
    """List prompts in LangSmith, optionally filtered."""
    print("Listing prompts...")

    try:
        prompts = list(client.list_prompts(limit=args.limit))

        if not prompts:
            print("No prompts found.")
            return

        print(f"\nFound {len(prompts)} prompt(s):\n")
        for p in prompts:
            print(f"  Name: {p.repo_handle}")
            print(f"  ID: {p.id}")
            print(f"  Public: {p.is_public}")
            if p.description:
                print(f"  Description: {p.description}")
            print()

    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)


def cmd_delete_prompt(args):
    """Delete a prompt from LangSmith."""
    prompt_name = args.name

    print(f"Deleting prompt: {prompt_name}")

    if not args.yes:
        confirm = input("Are you sure? (y/N): ")
        if confirm.lower() != 'y':
            print("Aborted.")
            return

    try:
        client.delete_prompt(prompt_name)
        print(f"✓ Deleted: {prompt_name}")
    except Exception as e:
        print(f"✗ Failed to delete: {e}")
        sys.exit(1)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Structured Output Prompts Experiment for LangSmith",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Create and push a structured output prompt
  ./run_test.sh create --name my-test-prompt

  # Pull and examine a prompt's manifest
  ./run_test.sh pull --name my-test-prompt

  # Test raw API call (bypass SDK transforms)
  ./run_test.sh raw --name my-raw-test

  # List prompts
  ./run_test.sh list

  # Delete a test prompt
  ./run_test.sh delete --name my-test-prompt --yes
        """
    )

    subparsers = parser.add_subparsers(dest="command", help="Command to execute")
    subparsers.required = True

    # Create command
    create_parser = subparsers.add_parser(
        "create",
        help="Create and push a structured output prompt"
    )
    create_parser.add_argument(
        "--name",
        help="Prompt name (default: test-structured-output-prompt-398)",
        default=None
    )
    create_parser.set_defaults(func=cmd_create_and_push)

    # Pull command
    pull_parser = subparsers.add_parser(
        "pull",
        help="Pull and examine a prompt's manifest structure"
    )
    pull_parser.add_argument(
        "--name",
        help="Prompt name to pull",
        required=True
    )
    pull_parser.set_defaults(func=cmd_pull_and_examine)

    # Raw API command
    raw_parser = subparsers.add_parser(
        "raw",
        help="Test creating a prompt via raw API (bypass SDK)"
    )
    raw_parser.add_argument(
        "--name",
        help="Prompt name (default: test-raw-structured-prompt-398)",
        default=None
    )
    raw_parser.set_defaults(func=cmd_raw_api_create)

    # List command
    list_parser = subparsers.add_parser("list", help="List prompts")
    list_parser.add_argument(
        "--limit",
        help="Maximum number of prompts to list",
        type=int,
        default=20
    )
    list_parser.set_defaults(func=cmd_list_prompts)

    # Delete command
    delete_parser = subparsers.add_parser("delete", help="Delete a prompt")
    delete_parser.add_argument(
        "--name",
        help="Prompt name to delete",
        required=True
    )
    delete_parser.add_argument(
        "--yes", "-y",
        help="Skip confirmation",
        action="store_true"
    )
    delete_parser.set_defaults(func=cmd_delete_prompt)

    args = parser.parse_args()
    args.func(args)
