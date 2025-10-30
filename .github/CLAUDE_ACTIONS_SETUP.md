# Claude Code GitHub Actions Setup

This repository is configured to use Claude Code in GitHub Actions via AWS Bedrock. Claude can be triggered by mentioning `@claude` in issues, pull requests, and comments.

## Prerequisites

### 1. AWS Bedrock Setup

Ensure you have:
- AWS Bedrock enabled with Claude model access in us-east-1 region
- AWS IAM user or access keys with Bedrock invoke permissions

### 2. Configure GitHub Secrets

   Add the required AWS credentials to your GitHub repository:

   **Steps to Add Secrets:**

   1. Go to your repository on GitHub
   2. Click **Settings** → **Secrets and variables** → **Actions**
   3. Click **New repository secret**
   4. Add the following secrets:
      - **Name:** `AWS_ACCESS_KEY_ID`
      - **Value:** Your AWS access key ID
   5. Click **Add secret**
   6. Repeat for the second secret:
      - **Name:** `AWS_SECRET_ACCESS_KEY`
      - **Value:** Your AWS secret access key
   7. Click **Add secret**

   **Note:** This replaces the previous `ANTHROPIC_API_KEY` approach. The workflow now uses AWS Bedrock for Claude model access.

## How It Works

The workflow (`.github/workflows/claude.yml`) triggers when:

- Someone mentions `@claude` in an issue comment
- Someone mentions `@claude` in a PR comment or review
- An issue is opened with `@claude` in the title or body

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

- **Model selection:** Change the Claude model version
- **Max turns:** Limit the number of interaction rounds
- **Allowed tools:** Restrict which bash commands Claude can run
- **System prompt:** Add project-specific instructions
- **Trigger phrase:** Change from `@claude` to something else

See the workflow file for commented examples.

## Security Notes

⚠️ **Important:**
- Never commit API keys directly to the repository
- Always use GitHub Actions secrets (`${{ secrets.ANTHROPIC_API_KEY }}`)
- Rotate API keys regularly
- Monitor usage in your Anthropic console

## Troubleshooting

If Claude isn't responding:

1. Verify the GitHub App is installed on this repository
2. Check that `ANTHROPIC_API_KEY` is set in repository secrets
3. Ensure the workflow file exists in `.github/workflows/`
4. Check GitHub Actions logs for error messages
5. Verify you're using the correct trigger phrase (default: `@claude`)

## Resources

- [Claude Code GitHub Actions Documentation](https://docs.claude.com/en/docs/claude-code/github-actions)
- [Claude Code Action Repository](https://github.com/anthropics/claude-code-action)
- [Anthropic API Documentation](https://docs.anthropic.com/)
