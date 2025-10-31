# Claude Code GitHub Actions Setup

This repository is configured to use Claude Code in GitHub Actions via **AWS Bedrock with static credentials**. Claude can be triggered by mentioning `@claude` in issues, pull requests, and comments.

## Prerequisites

### 1. AWS Bedrock Setup

Ensure you have:
- AWS Bedrock enabled with Claude model access in us-east-1 region
- AWS IAM user with Bedrock invoke permissions
- AWS access key ID and secret access key for that user

**Note on Authentication:** This repository uses **static AWS credentials** (access key ID + secret access key) for simplicity. The [official claude-code-action documentation](https://github.com/anthropics/claude-code-action/blob/main/docs/cloud-providers.md) recommends OIDC authentication with IAM roles for enhanced security. If you need OIDC, see the "Alternative: OIDC Authentication" section below.

### 2. Configure GitHub Secrets

Add the required AWS credentials to your GitHub repository:

**Required Secrets for Claude Code Workflow:**
- `AWS_ACCESS_KEY_ID` - Your AWS access key ID
- `AWS_SECRET_ACCESS_KEY` - Your AWS secret access key
- `AWS_REGION` - AWS region (default: `us-east-1`)

**Steps to Add Secrets:**

1. Go to your repository on GitHub
2. Click **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Add each secret:
   - **Name:** `AWS_ACCESS_KEY_ID`
   - **Value:** Your AWS access key ID
5. Click **Add secret**
6. Repeat for `AWS_SECRET_ACCESS_KEY` and `AWS_REGION`

**Additional Secrets (Local Development Only):**

These secrets may exist in the repository but are **not used by the Claude Code workflow**. They're for local devcontainer development:
- `LANGSMITH_API_KEY`, `LANGSMITH_ORGANIZATION_ID`, `LANGSMITH_WORKSPACE_ID` - LangSmith access
- `GH_PROJECT_PAT` - GitHub Projects API access
- `ANTHROPIC_MODEL`, `ANTHROPIC_SMALL_FAST_MODEL` - Model configuration

The Claude Code workflow follows the **principle of least privilege** and only has access to AWS credentials needed for Bedrock.

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
- Never commit AWS credentials directly to the repository
- Always use GitHub Actions secrets (`${{ secrets.AWS_ACCESS_KEY_ID }}`)
- Rotate AWS access keys regularly
- Monitor usage in your AWS Bedrock console
- The workflow is restricted to trusted users only (see `if:` condition in workflow)
- Claude Code workflow only has access to AWS credentials (least privilege)

## Troubleshooting

If Claude isn't responding:

1. **Check workflow triggers:** Ensure you used `@claude` in a comment/issue/PR
2. **Verify you're a trusted user:** Check the `if:` condition in `.github/workflows/claude.yml`
3. **Check AWS secrets:** Confirm `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, and `AWS_REGION` are set in repository secrets
4. **Verify AWS Bedrock access:** Ensure your AWS account has Claude model access enabled in the specified region
5. **Check workflow logs:** Go to Actions tab → Select failed workflow run → Review logs for error messages
6. **Verify environment variable:** Ensure `CLAUDE_CODE_USE_BEDROCK: "1"` is set in workflow `env:` section

### Common Issues

**Exit code 1 from Claude Code:**
- Missing or invalid AWS credentials
- AWS Bedrock model not enabled in region
- Missing `CLAUDE_CODE_USE_BEDROCK` environment variable
- Network connectivity issues to AWS Bedrock

**Workflow doesn't trigger:**
- User not in the allowed list (check `if:` condition)
- Workflow file syntax error
- `@claude` not mentioned in comment/issue/PR

## Alternative: OIDC Authentication

For enhanced security, you can use OIDC authentication instead of static credentials. This is the [officially recommended approach](https://github.com/anthropics/claude-code-action/blob/main/docs/cloud-providers.md).

**OIDC requires:**
1. AWS IAM role with Bedrock permissions
2. GitHub App with appropriate permissions
3. Additional workflow setup steps

**Required secrets for OIDC:**
- `AWS_ROLE_TO_ASSUME` - IAM role ARN
- `APP_ID` - GitHub App ID
- `APP_PRIVATE_KEY` - GitHub App private key

**Workflow changes:**
```yaml
- name: Configure AWS Credentials (OIDC)
  uses: aws-actions/configure-aws-credentials@v4
  with:
    role-to-assume: ${{ secrets.AWS_ROLE_TO_ASSUME }}
    aws-region: us-east-1

- name: Generate GitHub App token
  id: app-token
  uses: actions/create-github-app-token@v2
  with:
    app-id: ${{ secrets.APP_ID }}
    private-key: ${{ secrets.APP_PRIVATE_KEY }}

- uses: anthropics/claude-code-action@v1
  with:
    use_bedrock: "true"
    claude_args: |
      --model anthropic.claude-4-0-sonnet-20250805-v1:0
```

See the [cloud providers documentation](https://github.com/anthropics/claude-code-action/blob/main/docs/cloud-providers.md) for full OIDC setup instructions.

## Resources

- [Claude Code GitHub Actions Documentation](https://docs.claude.com/en/docs/claude-code/github-actions)
- [Claude Code Action Repository](https://github.com/anthropics/claude-code-action)
- [Claude Code Action - Cloud Providers (Bedrock)](https://github.com/anthropics/claude-code-action/blob/main/docs/cloud-providers.md)
- [AWS Bedrock Documentation](https://docs.aws.amazon.com/bedrock/)
