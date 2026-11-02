# Exporting a Prompt

`langstar prompt export` writes one commit of a LangSmith prompt to a JSON file. You commit the file to Git, a reviewer reads the prompt change in a pull request, and `langstar prompt import` recreates the prompt in another workspace later.

Export runs with the source workspace's credentials. It needs `LANGSMITH_API_KEY`, and the workspace comes from `--workspace-id`, `LANGSMITH_WORKSPACE_ID` or `workspace_id` in `config.toml`, as for the other `prompt` commands.

## Export a commit

```bash
# The latest commit, to a file
langstar prompt export my-prompt --output my-prompt.json

# The commit the prod tag points at
langstar prompt export my-prompt --commit prod --output my-prompt.json

# A commit by its hash
langstar prompt export my-prompt --commit <commit-hash> --output my-prompt.json

# Without --output, stdout holds the file and nothing else
langstar prompt export my-prompt > my-prompt.json
```

`my-prompt` and `-/my-prompt` both name a prompt in your workspace, and both write the same file. `-/my-prompt` works as it is, with no `--` in front of it. A public prompt takes its owner, as in `owner/my-prompt`.

Progress messages and warnings go to stderr, so a file written through `>` holds only the export.

## What the file holds

```json
{
  "kind": "prompt",
  "format_version": 1,
  "source": { "workspace_id": "<source-workspace-id>", "handle": "my-prompt", "commit_hash": "<hash>" },
  "repo": { "handle": "my-prompt", "description": "...", "readme": "...", "tags": ["support"], "is_public": false },
  "commit": { "description": "...", "manifest": { "...": "..." }, "model_config": {}, "model_provider": "ChatOpenAI", "tags": ["prod"] },
  "requires": { "secrets": ["OPENAI_API_KEY"], "model_class": "ChatOpenAI" }
}
```

- `source` says where the prompt came from: the workspace, the handle and the commit hash.
- `repo` holds the prompt's description, readme, tags and visibility.
- `commit` holds the commit's description, its manifest with the chat model in it, the model's settings and class name as LangSmith returned them, and the tags that point at this commit.
- `requires.secrets` names each workspace secret the model reads, such as `OPENAI_API_KEY`. The file holds the secret's name and never its value.
- `requires.model_class` is the chat model's class name, such as `ChatOpenAI`, or null for a prompt saved without a model.

The file holds no export time, and langstar sorts its keys and its tag and secret lists. Exporting a commit twice writes the same file, unless someone changed the prompt's description, readme, tags or visibility, or moved a tag, in between.

## Export refuses to write a credential

Before it writes anything, export checks every string in the `repo` and `commit` objects. It skips a LangChain secret reference, `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`, because that holds a secret's name, and it skips the empty string.

The check flags a string when the name of its key, or of the array it sits in, holds one of these words: `secret`, `token`, `password`, `passwd`, `pwd`, `credential`, `credentials`, `apikey`, `authorization`, `cookie`. It also flags `key` or `keys` right after `api`, `access`, `private`, `secret`, `signing` or `account`. A key name splits into words at `_`, `-`, `.`, a space and each change from a lower-case to an upper-case letter, and case does not matter. So `openai_api_key`, `OPENAI_API_KEY`, `x-api-key`, `Authorization` and `privateKey` are flagged, and `max_tokens` and `cache_key` pass.

Under any key, the check flags a string that holds:

- a bearer token: `Bearer ` and at least 16 token characters;
- a `Basic ` value whose payload decodes to a user and a password;
- a PEM private key: `-----BEGIN` and `PRIVATE KEY-----`;
- a URL with a password, such as `postgresql://app:<password>@db.example.com/prod`;
- a connection string with a value after `Password=`, `Pwd=`, `AccountKey=`, `SharedAccessKey=` or `SharedAccessSignature=`;
- a JSON Web Token: three base64url parts, the first starting `eyJ`;
- a token that starts with a provider's prefix: `sk-`, `lsv2_`, `ghp_`, `gho_`, `ghs_`, `github_pat_`, `xoxb-`, `xoxp-`, `xoxa-`, `AKIA`, `ASIA`, `AIza` or `hf_`.

When the check flags a string, export writes no file and nothing to stdout. It names the JSON Pointer path of each flagged string and the rule that flagged it, leaves the string out, and exits with status 1:

```
Error: This would write a credential to the export file:
  /commit/manifest/kwargs/last/kwargs/openai_api_key (key name)
Move each one into the workspace's secrets or the deployment's environment variables. To write a string that holds no credential, read it first and name its path with --not-a-secret.
No file was written.
```

Keep credentials in the workspace's secrets (see [workspace-secrets.md](./workspace-secrets.md)), and have the model read them through a secret reference. A prompt saved in the LangSmith playground does this already.

### A flagged string that holds no secret

Some flagged strings are harmless, such as a message template that shows an example `Authorization: Bearer` header. Read the string at the path export printed, and when it is harmless, name that path with `--not-a-secret`:

```bash
langstar prompt export my-prompt --output my-prompt.json \
  --not-a-secret /commit/manifest/kwargs/first/kwargs/messages/0/kwargs/prompt/kwargs/template
```

Export then writes the string and prints a warning naming the path to stderr. Give `--not-a-secret` once for each path. A flagged string at any other path still stops the export. Export also stops, and names the path, when you name a path whose string passes the check, so a script that keeps naming a path after the string there has changed stops too. Export runs the check every time.

### What the check misses

The check reads key names and value shapes. A credential under an ordinary key name with an ordinary-looking value, such as a random string under a key called `x`, passes the check and reaches the file. Read every export file before you commit it.
