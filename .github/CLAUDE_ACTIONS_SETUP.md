# Claude Code GitHub Actions Setup

This repository is configured to use Claude Code in GitHub Actions. Claude can be triggered by mentioning `@claude` in issues, pull requests, and comments.

## Prerequisites

1. **Install the Claude GitHub App**
   - Visit: https://github.com/apps/claude
   - Install the app on this repository
   - Grant the necessary permissions (read/write access to code, PRs, and issues)

2. **Configure GitHub Secrets**

   You need to add your Anthropic API key to GitHub repository secrets:

   ### Steps to Add Secret:

   1. Go to your repository on GitHub
   2. Click **Settings** → **Secrets and variables** → **Actions**
   3. Click **New repository secret**
   4. Add the following secret:
      - **Name:** `ANTHROPIC_API_KEY`
      - **Value:** Your Anthropic API key (starts with `sk-ant-`)
   5. Click **Add secret**

   ### Getting an Anthropic API Key:

   - Visit: https://console.anthropic.com/settings/keys
   - Create a new API key
   - Copy the key (it will only be shown once)
   - Store it securely

   **Alternative:** If you have Claude Pro/Max subscription, you can use:
   - **Name:** `CLAUDE_CODE_OAUTH_TOKEN`
   - **Value:** Generate via `claude setup-token` command

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
