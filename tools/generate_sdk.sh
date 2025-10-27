#!/usr/bin/env bash

# Generate Rust SDK from OpenAPI specifications
# This script fetches OpenAPI specs from LangChain services and generates Rust client code

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
LANGSMITH_OPENAPI_URL="https://api.smith.langchain.com/openapi.json"
LANGGRAPH_OPENAPI_URL="https://api.langgraph.cloud/openapi.json"
OUTPUT_DIR="sdk/src/generated"
SPECS_DIR="tools/specs"

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to fetch OpenAPI spec
fetch_spec() {
    local url=$1
    local output=$2

    print_info "Fetching OpenAPI spec from $url"

    if command_exists curl; then
        curl -sSL "$url" -o "$output"
    elif command_exists wget; then
        wget -q "$url" -O "$output"
    else
        print_error "Neither curl nor wget is available. Please install one of them."
        exit 1
    fi

    print_success "Saved spec to $output"
}

# Main script
main() {
    print_info "OpenAPI SDK Generation for Langstar"
    echo ""

    # Create directories if they don't exist
    mkdir -p "$SPECS_DIR"
    mkdir -p "$OUTPUT_DIR"

    # Fetch OpenAPI specifications
    print_info "Step 1: Fetching OpenAPI specifications"
    fetch_spec "$LANGSMITH_OPENAPI_URL" "$SPECS_DIR/langsmith-openapi.json"
    fetch_spec "$LANGGRAPH_OPENAPI_URL" "$SPECS_DIR/langgraph-openapi.json"
    echo ""

    # Check for OpenAPI generators
    print_info "Step 2: Checking for OpenAPI generators"

    GENERATOR=""

    if command_exists openapi-generator-cli; then
        print_success "Found openapi-generator-cli"
        GENERATOR="openapi-generator-cli"
    elif command_exists docker; then
        print_success "Found docker (will use openapitools/openapi-generator-cli)"
        GENERATOR="docker"
    else
        print_warning "No OpenAPI generator found"
        print_info "You can install one of the following:"
        print_info "  1. openapi-generator-cli: npm install -g @openapitools/openapi-generator-cli"
        print_info "  2. Docker: https://docs.docker.com/get-docker/"
        print_info ""
        print_info "For now, manual client implementations in sdk/src/ will be used."
        exit 0
    fi
    echo ""

    # Generate code
    print_info "Step 3: Generating Rust client code"

    if [ "$GENERATOR" = "openapi-generator-cli" ]; then
        print_info "Generating LangSmith client..."
        openapi-generator-cli generate \
            -i "$SPECS_DIR/langsmith-openapi.json" \
            -g rust \
            -o "$OUTPUT_DIR/langsmith" \
            --additional-properties=packageName=langsmith_client

        print_info "Generating LangGraph client..."
        openapi-generator-cli generate \
            -i "$SPECS_DIR/langgraph-openapi.json" \
            -g rust \
            -o "$OUTPUT_DIR/langgraph" \
            --additional-properties=packageName=langgraph_client

    elif [ "$GENERATOR" = "docker" ]; then
        print_info "Generating LangSmith client..."
        docker run --rm \
            -v "${PWD}:/local" \
            openapitools/openapi-generator-cli generate \
            -i "/local/$SPECS_DIR/langsmith-openapi.json" \
            -g rust \
            -o "/local/$OUTPUT_DIR/langsmith" \
            --additional-properties=packageName=langsmith_client

        print_info "Generating LangGraph client..."
        docker run --rm \
            -v "${PWD}:/local" \
            openapitools/openapi-generator-cli generate \
            -i "/local/$SPECS_DIR/langgraph-openapi.json" \
            -g rust \
            -o "/local/$OUTPUT_DIR/langgraph" \
            --additional-properties=packageName=langgraph_client
    fi

    print_success "Code generation complete!"
    echo ""

    # Summary
    print_info "Summary"
    print_info "  Generated code: $OUTPUT_DIR"
    print_info "  Next steps:"
    print_info "    1. Review generated code in $OUTPUT_DIR"
    print_info "    2. Update sdk/src/lib.rs to use generated clients"
    print_info "    3. Run 'cargo build' to verify compilation"
    echo ""
}

main "$@"
