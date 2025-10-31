# Claude Code GitHub Actions Setup

This repository is configured to use Claude Code in GitHub Actions via **AWS Bedrock with static credentials**. Claude can be triggered by mentioning `@claude` in issues, pull requests, and comments.

## Prerequisites

### 1. AWS Bedrock Setup

Ensure you have:
- AWS Bedrock enabled with Claude model access in us-east-1 region
- AWS IAM user with Bedrock invoke permissions
- AWS access key ID and secret access key for that user

### 2. Configure GitHub Secrets

Add the required AWS credentials to your GitHub repository:

**Required Secrets for Claude Code Workflow:**
- `AWS_ACCESS_KEY_ID` - Your AWS access key ID
- `AWS_SECRET_ACCESS_KEY` - Your AWS secret access key
- `AWS_REGION` - AWS region (default: `us-east-1`)
- `ANTHROPIC_MODEL` - Bedrock model ID (e.g., `us.anthropic.claude-sonnet-4-5-20250929-v1:0`)
- `ANTHROPIC_SMALL_FAST_MODEL` - Bedrock small/fast model ID (e.g., `us.anthropic.claude-haiku-4-5-20251001-v1:0`)
- `LANGSMITH_API_KEY` - LangSmith API key (for integration tests)
- `LANGSMITH_ORGANIZATION_ID` - LangSmith organization ID (for integration tests)
- `LANGSMITH_WORKSPACE_ID` - LangSmith workspace ID (for integration tests)

**Steps to Add Secrets:**

1. Go to your repository on GitHub
2. Click **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Add each required secret listed above

**Additional Secrets (Local Development Only):**

These secrets exist in the repository but are **not used by the Claude Code workflow**:
- `GH_PROJECT_PAT` - GitHub Projects API access (local development only)

The Claude Code workflow follows the **principle of least privilege** and only has access to credentials needed for Bedrock and running integration tests.

## How It Works

The workflow (`.github/workflows/claude.yml`) triggers when:

- Someone mentions `@claude` in an issue comment
- Someone mentions `@claude` in a PR comment or review
- An issue is opened with `@claude` in the title or body

**Security Note:** The workflow is restricted to specific trusted users only. Untrusted contributors cannot trigger the workflow or access secrets.

Claude will then:
- Analyze the context (code, issue, PR)
- Generate responses or code changes
- Create commits and PRs as needed
- Follow the guidelines in `CLAUDE.md`

## Usage Examples

### In Issues:
```
@claude please implement a new user authentication feature
```

### In Pull Requests:
```
@claude review this code and suggest improvements
```

### In Comments:
```
@claude fix the failing tests in this PR
```

## Configuration Options

The workflow can be customized in `.github/workflows/claude.yml`:

- **Model selection:** Change the Claude model version via `claude_args`
- **Max turns:** Limit the number of interaction rounds
- **Allowed tools:** Restrict which bash commands Claude can run
- **System prompt:** Add project-specific instructions
- **Trigger phrase:** Change from `@claude` to something else

See the workflow file for commented examples.

## Security Notes

⚠️ **Important:**
- Never commit credentials directly to the repository
- Always use GitHub Actions secrets for sensitive values
- Rotate AWS access keys and API keys regularly
- Monitor usage in your AWS Bedrock and LangSmith consoles
- The workflow is restricted to trusted users only (see `if:` condition in workflow)
- Claude Code workflow only has access to credentials needed for Bedrock and integration tests (least privilege)

## Troubleshooting

If Claude isn't responding:

1. **Check workflow triggers:** Ensure you used `@claude` in a comment/issue/PR
2. **Verify you're a trusted user:** Check the `if:` condition in `.github/workflows/claude.yml`
3. **Check AWS secrets:** Confirm `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_REGION` are set in repository secrets
4. **Check model secrets:** Confirm `ANTHROPIC_MODEL` and `ANTHROPIC_SMALL_FAST_MODEL` are set with correct Bedrock model IDs
5. **Check LangSmith secrets:** If integration tests are failing, confirm `LANGSMITH_API_KEY`, `LANGSMITH_ORGANIZATION_ID`, and `LANGSMITH_WORKSPACE_ID` are set
6. **Verify AWS Bedrock access:** Ensure your AWS account has Claude model access enabled for the specified model IDs
7. **Check workflow logs:** Go to Actions tab → Select failed workflow run → Review logs for error messages
8. **Verify environment variable:** Ensure `CLAUDE_CODE_USE_BEDROCK: "1"` is set in workflow `env:` section

### Common Issues

**Exit code 1 from Claude Code:**
- Missing or invalid AWS credentials
- AWS Bedrock model not enabled in region (check model IDs match what's enabled in Bedrock)
- Missing `CLAUDE_CODE_USE_BEDROCK` environment variable
- Incorrect model IDs in `ANTHROPIC_MODEL` or `ANTHROPIC_SMALL_FAST_MODEL` secrets
- Network connectivity issues to AWS Bedrock

**Workflow doesn't trigger:**
- User not in the allowed list (check `if:` condition)
- Workflow file syntax error
- `@claude` not mentioned in comment/issue/PR

## Resources

- [Claude Code GitHub Actions Documentation](https://docs.claude.com/en/docs/claude-code/github-actions)
- [Claude Code Action Repository](https://github.com/anthropics/claude-code-action)
- [AWS Bedrock Documentation](https://docs.aws.amazon.com/bedrock/)
