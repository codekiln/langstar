# Troubleshooting Guide

This guide helps you diagnose and resolve common issues when using Langstar with LangSmith and LangGraph services.

## Table of Contents

- [Quick Diagnostics](#quick-diagnostics)
- [Configuration Issues](#configuration-issues)
- [Authentication Errors](#authentication-errors)
- [Scoping Problems](#scoping-problems)
- [Service-Specific Issues](#service-specific-issues)
- [Network and Connectivity](#network-and-connectivity)
- [Performance Issues](#performance-issues)
- [Getting Help](#getting-help)

## Quick Diagnostics

### First Steps

Run these commands to gather diagnostic information:

```bash
# 1. Check configuration
langstar config

# 2. Check environment variables
env | grep LANGSMITH
env | grep LANGGRAPH
env | grep LANGSTAR

# 3. Check config file
cat ~/.langstar/config.toml

# 4. Test basic connectivity
langstar prompt list
langstar assistant list
```

### Configuration Check Output

**Healthy configuration:**

```
Configuration file: ~/.langstar/config.toml

LangSmith Configuration (for 'prompt' commands):
  API key: configured
  Organization ID: <org-id> (scopes prompt operations)
  → Prompt commands will use organization-scoped resources

LangGraph Configuration (for 'assistant' commands):
  API key: configured
  → Assistant commands use deployment-level resources
```

**Problem indicators:**

```
LangSmith Configuration (for 'prompt' commands):
  API key: NOT SET  ← Problem: Missing API key

LangGraph Configuration (for 'assistant' commands):
  API key: NOT SET  ← Problem: Missing API key
```

## Configuration Issues

### Issue: "Missing required configuration: LANGSMITH_API_KEY"

**Symptoms:**

```bash
$ langstar prompt list
Error: Missing required configuration: LANGSMITH_API_KEY
```

**Cause:** No API key configured for LangSmith.

**Solution 1: Set environment variable**

```bash
export LANGSMITH_API_KEY="<your-api-key>"
langstar prompt list
```

**Solution 2: Create config file**

```bash
mkdir -p ~/.langstar
cat > ~/.langstar/config.toml <<EOF
[langstar]
langsmith_api_key = "<your-api-key>"
EOF

langstar prompt list
```

**Solution 3: Verify config file location**

```bash
# Check if file exists
ls -la ~/.langstar/config.toml

# Check file permissions (should be readable)
ls -l ~/.langstar/config.toml

# Check file contents
cat ~/.langstar/config.toml
```

### Issue: Configuration file not found

**Symptoms:**

```bash
$ langstar config
Configuration file: NOT FOUND
```

**Cause:** No configuration file at expected locations.

**Langstar checks these locations:**

1. `./config.toml` (current directory)
2. `~/.langstar/config.toml`
3. `~/.config/langstar/config.toml`

**Solution:**

```bash
# Create in user home directory
mkdir -p ~/.langstar
touch ~/.langstar/config.toml

# Or in XDG config directory
mkdir -p ~/.config/langstar
touch ~/.config/langstar/config.toml
```

### Issue: Configuration file ignored

**Symptoms:**

Config file exists but settings aren't applied.

**Causes and Solutions:**

**1. Environment variables overriding:**

```bash
# Check for environment variables
env | grep LANGSMITH
env | grep LANGGRAPH

# Unset to use config file
unset LANGSMITH_API_KEY
unset LANGGRAPH_API_KEY
```

**2. TOML syntax errors:**

```bash
# Check file syntax
cat ~/.langstar/config.toml
```

**Common TOML mistakes:**

```toml
# ❌ Wrong: Missing quotes
langsmith_api_key = your-key

# ✅ Correct: Quoted string
langsmith_api_key = "your-key"

# ❌ Wrong: Wrong section name
[langsmit]  # Typo
langsmith_api_key = "key"

# ✅ Correct: Right section
[langstar]
langsmith_api_key = "key"
```

**3. File permissions:**

```bash
# Check permissions
ls -l ~/.langstar/config.toml

# Should be readable
# If not: chmod 600 ~/.langstar/config.toml
```

### Issue: Wrong values in configuration

**Symptoms:**

Settings applied but values incorrect.

**Debug:**

```bash
# Show current configuration
langstar config

# Check config file
cat ~/.langstar/config.toml

# Check environment (overrides config file)
env | grep LANGSMITH
env | grep LANGGRAPH
```

**Remember precedence order:**

1. Command-line flags (highest)
2. Configuration file
3. Environment variables (lowest)

## Authentication Errors

### Issue: "Authentication failed"

**Symptoms:**

```bash
$ langstar prompt list
Error: Authentication failed (401)
```

**Cause 1: Invalid API key**

```bash
# Verify API key is set
langstar config

# Try setting a fresh key
export LANGSMITH_API_KEY="<your-api-key>"
```

**Cause 2: Expired API key**

```bash
# Generate a new API key at:
# https://smith.langchain.com (for LangSmith)

# Set the new key
export LANGSMITH_API_KEY="<new-api-key>"
```

**Cause 3: Wrong service key**

```bash
# Common mistake: Using LangSmith key for assistants
export LANGSMITH_API_KEY="<langsmith-key>"
langstar assistant list  # Fails or wrong deployment

# Solution: Use correct key
export LANGGRAPH_API_KEY="<langgraph-key>"
langstar assistant list
```

### Issue: "Forbidden" (403) errors

**Symptoms:**

```bash
$ langstar prompt get team/prompt-name
Error: Forbidden (403)
```

**Cause:** API key doesn't have access to the resource.

**Solutions:**

**1. Check organization/workspace access:**

```bash
# Verify scoping
langstar config

# Try without scoping
unset LANGSMITH_ORGANIZATION_ID
unset LANGSMITH_WORKSPACE_ID
langstar prompt list
```

**2. Verify resource ownership:**

```bash
# List accessible prompts
langstar prompt list

# Check if prompt exists in your scope
langstar prompt search "prompt-name"
```

**3. Check API key permissions:**

- Verify API key has correct permissions in LangSmith dashboard
- Ensure key hasn't been revoked or restricted

## Scoping Problems

### Issue: "No prompts found" but I have prompts

**Symptoms:**

```bash
$ langstar prompt list
# Empty or very limited results
```

**Cause:** Scoping is too narrow or filtering unintentionally.

**Solution 1: Check scoping configuration**

```bash
# Show current scoping
langstar config

# If scoped to organization
export LANGSMITH_ORGANIZATION_ID="<org-id>"
langstar prompt list  # Private prompts only

langstar prompt list --public  # Public prompts
```

**Solution 2: Remove scoping**

```bash
# Clear scoping
unset LANGSMITH_ORGANIZATION_ID
unset LANGSMITH_WORKSPACE_ID

# List all accessible prompts
langstar prompt list
```

**Solution 3: Verify workspace access**

```bash
# Try organization-level instead
export LANGSMITH_ORGANIZATION_ID="<org-id>"
unset LANGSMITH_WORKSPACE_ID
langstar prompt list
```

### Issue: Can't access team prompts

**Symptoms:**

```bash
$ langstar prompt get team/shared-prompt
Error: Not found (404)
```

**Cause:** Not scoped to the right organization/workspace.

**Solution:**

```bash
# Set organization ID
export LANGSMITH_ORGANIZATION_ID="<team-org-id>"
langstar prompt list

# Or use flag
langstar prompt get team/shared-prompt --organization-id "<team-org-id>"
```

### Issue: Assistants not showing up

**Symptoms:**

```bash
$ langstar assistant list
# Empty or unexpected results
```

**Cause:** Using API key for wrong deployment.

**Solution:**

```bash
# Check which API key is set
langstar config
env | grep LANGGRAPH_API_KEY

# Verify you're using the correct deployment key
export LANGGRAPH_API_KEY="<correct-deployment-key>"
langstar assistant list
```

**Remember:** Assistants are deployment-level, NOT organization-scoped.

## Service-Specific Issues

### LangSmith Prompt Issues

#### Issue: Prompt not found

**Symptoms:**

```bash
$ langstar prompt get owner/prompt-name
Error: Not found (404)
```

**Solutions:**

**1. Verify prompt name:**

```bash
# List all prompts
langstar prompt list

# Search for similar names
langstar prompt search "prompt"
```

**2. Check scoping:**

```bash
# Try public prompts
langstar prompt list --public

# Try different organization
langstar prompt list --organization-id "<different-org-id>"
```

**3. Verify ownership:**

```bash
# Format: owner/prompt-name
langstar prompt get alice/my-prompt  # User prompt
langstar prompt get team/our-prompt  # Team prompt
langstar prompt get public/shared-prompt  # Public prompt
```

#### Issue: Can't see private vs public prompts

**Symptoms:**

When scoped, can't tell which prompts are public vs private.

**Solution:**

```bash
# Scoped defaults to private
export LANGSMITH_ORGANIZATION_ID="<org-id>"
langstar prompt list  # Private prompts

# Explicitly request public
langstar prompt list --public

# Compare
diff <(langstar prompt list) <(langstar prompt list --public)
```

### LangGraph Assistant Issues

#### Issue: Assistant creation fails

**Symptoms:**

```bash
$ langstar assistant create --graph-id g123 --name "Bot"
Error: Graph not found
```

**Cause:** Graph ID doesn't exist in deployment.

**Solution:**

```bash
# Verify graph ID with your LangGraph deployment
# Check LangGraph Cloud dashboard for available graph IDs

# Use correct graph ID
langstar assistant create \
  --graph-id graph_correct_id \
  --name "Bot"
```

#### Issue: Assistant update not applying

**Symptoms:**

```bash
$ langstar assistant update asst_123 --name "New Name"
Success
$ langstar assistant get asst_123
# Name unchanged
```

**Cause:** May be updating wrong deployment or assistant ID.

**Solution:**

```bash
# Verify assistant ID
langstar assistant list | grep "New Name"

# Check full details
langstar assistant get asst_123 --format json
```

#### Issue: Can't delete assistant

**Symptoms:**

```bash
$ langstar assistant delete asst_123
Error: Assistant in use or protected
```

**Cause:** Assistant may have active sessions or protection.

**Solution:**

```bash
# Force delete (if appropriate)
langstar assistant delete asst_123 --force

# Or wait for active sessions to complete
```

## Network and Connectivity

### Issue: Connection timeout

**Symptoms:**

```bash
$ langstar prompt list
Error: Connection timeout
```

**Causes and Solutions:**

**1. Network connectivity:**

```bash
# Test internet connection
ping smith.langchain.com
ping langchain-ai.github.io

# Test with curl
curl -I https://api.smith.langchain.com
```

**2. Proxy settings:**

```bash
# Check proxy environment
env | grep -i proxy

# Set proxy if needed
export HTTP_PROXY="http://proxy:port"
export HTTPS_PROXY="http://proxy:port"
```

**3. Firewall:**

```bash
# Check if firewall is blocking
# Consult your IT department if corporate network
```

### Issue: SSL/TLS errors

**Symptoms:**

```bash
Error: SSL certificate verification failed
```

**Causes and Solutions:**

**1. Outdated system certificates:**

```bash
# Update system certificates (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install ca-certificates

# macOS
# Certificates should be updated via system updates
```

**2. Corporate proxy with SSL inspection:**

```bash
# Contact IT for corporate certificate
# Install corporate root CA if required
```

### Issue: Rate limiting

**Symptoms:**

```bash
Error: Too many requests (429)
```

**Cause:** API rate limit exceeded.

**Solution:**

```bash
# Wait before retrying
sleep 60
langstar prompt list

# Use pagination to reduce requests
langstar assistant list --limit 50

# Implement backoff in scripts
for i in {1..5}; do
  langstar prompt list && break
  sleep $((i * 2))
done
```

## Performance Issues

### Issue: Slow response times

**Symptoms:**

Commands take a long time to complete.

**Solutions:**

**1. Use pagination:**

```bash
# Instead of fetching all
langstar assistant list --limit 20

# Use offset for next page
langstar assistant list --limit 20 --offset 20
```

**2. Narrow scoping:**

```bash
# Use workspace instead of organization
export LANGSMITH_WORKSPACE_ID="<workspace-id>"
unset LANGSMITH_ORGANIZATION_ID
langstar prompt list  # Faster, narrower results
```

**3. Use JSON for large datasets:**

```bash
# JSON is more efficient for large responses
langstar prompt list --format json > prompts.json
jq '.[] | .name' prompts.json
```

### Issue: Large output overwhelming terminal

**Symptoms:**

Too much output scrolling by.

**Solutions:**

```bash
# Use pagination with less
langstar prompt list | less

# Limit results
langstar assistant list --limit 10

# Use grep to filter
langstar prompt list | grep "customer"

# JSON with jq for structured filtering
langstar prompt list --format json | jq '.[] | select(.name | contains("prod"))'
```

## Getting Help

### Debug Mode

Enable debug mode for detailed error information:

```bash
# Set debug environment variable
export LANGSTAR_DEBUG=1
langstar prompt list

# Or use debug flag (if available)
langstar --debug prompt list
```

### Gathering Diagnostic Information

When reporting issues, include:

```bash
# 1. Configuration (redact API keys!)
langstar config

# 2. Environment (redact sensitive values!)
env | grep LANGSMITH | sed 's/=.*/=<redacted>/'
env | grep LANGGRAPH | sed 's/=.*/=<redacted>/'

# 3. Version information
langstar --version
cargo --version
rustc --version

# 4. Full error message
langstar prompt list 2>&1 | tee error.log
```

### Common Redaction Mistakes

❌ **DON'T share:**

```bash
export LANGSMITH_API_KEY="lsv2_sk_abc123..."
export LANGSMITH_ORGANIZATION_ID="d1e1dfff-39bf..."
```

✅ **DO share:**

```bash
export LANGSMITH_API_KEY="<redacted>"
export LANGSMITH_ORGANIZATION_ID="<redacted>"
```

### Reporting Bugs

When reporting issues:

1. **Check existing issues** first
2. **Provide minimal reproduction** steps
3. **Include error messages** (redacted)
4. **Describe expected vs actual** behavior
5. **Include environment info** (OS, Rust version, etc.)

**Issue template:**

```markdown
## Description

Brief description of the issue.

## Steps to Reproduce

1. Set configuration: `export LANGSMITH_API_KEY="<redacted>"`
2. Run command: `langstar prompt list`
3. Observe error: `Error: Authentication failed`

## Expected Behavior

Should list prompts.

## Actual Behavior

Returns authentication error.

## Environment

- OS: Ubuntu 22.04
- Langstar version: 0.2.0
- Rust version: 1.78.0
- Configuration: Using config file

## Additional Context

- Issue started after updating API key
- Works with old key
- Verified new key in web UI
```

### Getting Support

**Documentation:**
- [README](../README.md)
- [Configuration Guide](./configuration.md)
- [Example Workflows](./examples/)
- [Architecture Documentation](./architecture.md)

**Community:**
- [GitHub Issues](https://github.com/codekiln/langstar/issues)
- [Discussions](https://github.com/codekiln/langstar/discussions)

**Upstream Documentation:**
- [LangSmith Docs](https://docs.smith.langchain.com/)
- [LangGraph Cloud Docs](https://langchain-ai.github.io/langgraph/cloud/)

## Quick Reference

### Configuration Check

```bash
langstar config
env | grep LANGSMITH
env | grep LANGGRAPH
cat ~/.langstar/config.toml
```

### Reset Configuration

```bash
# Clear environment
unset LANGSMITH_API_KEY
unset LANGSMITH_ORGANIZATION_ID
unset LANGSMITH_WORKSPACE_ID
unset LANGGRAPH_API_KEY

# Remove config file
rm ~/.langstar/config.toml

# Start fresh
export LANGSMITH_API_KEY="<key>"
export LANGGRAPH_API_KEY="<key>"
```

### Test Connectivity

```bash
# LangSmith
langstar prompt list --format json | jq '. | length'

# LangGraph
langstar assistant list --format json | jq '. | length'
```

### Common Error Codes

| Code | Meaning | Common Cause |
|------|---------|--------------|
| 401 | Unauthorized | Invalid or missing API key |
| 403 | Forbidden | No access to resource |
| 404 | Not Found | Resource doesn't exist or wrong scoping |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Server Error | Upstream service issue |
| Timeout | Connection timeout | Network or firewall issue |

## Still Having Issues?

If you've tried the solutions in this guide and still have problems:

1. **Search GitHub Issues**: https://github.com/codekiln/langstar/issues
2. **Ask in Discussions**: https://github.com/codekiln/langstar/discussions
3. **Create a New Issue**: Include diagnostic info (redacted!)
4. **Check Upstream Status**: LangSmith/LangGraph may have service issues

**Before asking for help:**
- [ ] Checked configuration with `langstar config`
- [ ] Verified API keys are correct and active
- [ ] Tried basic commands (`langstar prompt list`, `langstar assistant list`)
- [ ] Read relevant documentation sections
- [ ] Searched existing GitHub issues
- [ ] Prepared redacted diagnostic information
