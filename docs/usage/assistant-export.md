# Assistant Export

`langstar assistant export` writes one assistant to a JSON file. The file holds what you need to recreate the assistant on another deployment, and you can keep it in Git so a reviewer sees each change to the assistant in a pull request.

An assistant belongs to a deployment, so you name the deployment with `--deployment`, by name or by ID, as the other `langstar assistant` commands do.

## Export an assistant

```bash
langstar assistant export <assistant-id> --deployment my-agent --output support-bot.json
```

Without `--output`, export prints the file to stdout and nothing else, and writes its progress messages to stderr, so you can redirect it:

```bash
langstar assistant export <assistant-id> --deployment my-agent > support-bot.json
```

To see every flag:

```bash
langstar assistant export --help
```

## What the file holds

```json
{
  "assistant": {
    "config": { "configurable": { "system_prompt": "support-system:prod" }, "recursion_limit": 10 },
    "context": { "system_prompt": "support-system:prod" },
    "description": "Answers support questions.",
    "graph_id": "agent",
    "metadata": { "team": "support" },
    "name": "support-bot"
  },
  "format_version": 1,
  "kind": "assistant",
  "requires": { "prompts": { "/config/configurable/system_prompt": "support-system:prod" } },
  "source": { "assistant_id": "<assistant-id>", "deployment_id": "<deployment-id>", "version": 3 }
}
```

- `assistant` holds the name, description, graph ID, configuration, context and metadata. The Agent Server returns an assistant's settings in both `config.configurable` and `context`, and export writes both as the server returned them.
- `source` names the deployment, the assistant ID and the assistant's version. The assistant ID, the version and the timestamps the server sets stay out of `assistant`, so they never read as a change against a copy.
- `requires.prompts` lists the prompts the assistant refers to, as the next section shows.

The keys are sorted and the file holds no export time, so exporting the same assistant version twice writes the same file, and Git has nothing to commit when nothing changed.

## Record the prompts an assistant refers to

An assistant names a prompt through whatever configuration key its graph reads, so langstar cannot find those references by itself. Name each one with `--prompt-ref` and the JSON Pointer of the string that holds the prompt name, inside the assistant:

```bash
langstar assistant export <assistant-id> --deployment my-agent --output support-bot.json \
  --prompt-ref /config/configurable/system_prompt
```

The string at that pointer holds a prompt name of the form `[<owner>/]<handle>[:<tag>]`, such as `support-system:prod`. A name with no owner, or with the owner `-`, means the prompt in the current workspace. A name with no tag means the prompt's latest commit. Export writes each pointer and the name it holds under `requires.prompts`. When a pointer names no string in the assistant, export names the pointer, writes no file and exits with status 1.

Name each prompt by its handle and a tag, such as `support-system:prod`. The same prompt has a different commit hash in each workspace, and import copies the tag to the destination workspace.

## A graph's default assistant

Every deployment creates a default assistant for each graph it serves. Export refuses one, says why, writes no file and exits with status 1, because the destination deployment creates its own. Create an assistant with `langstar assistant create` and export that one.

## Export refuses to write a credential

A secret pushed to Git stays in the repository's history after someone deletes the file. So before it writes anything, export runs a credential check over every string in the assistant: its name, description, configuration, context and metadata, at any depth. The check flags a string by the name of its key, such as `openai_api_key` or `Authorization`, or by the shape of its value, such as a bearer token or a URL with a password in it. The [prompt export page](prompt-export.md) lists every key name and value shape the check flags.

The check skips a LangChain secret reference, `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`, because it holds a secret's name and never its value, and it skips the empty string.

When the check flags a string, export names the JSON Pointer path of each one and the rule that flagged it, leaves the string out of the message, writes no file and nothing to stdout, and exits with status 1:

```text
Error: This would write a credential to the export file:
  /config/configurable/OPENAI_API_KEY (key name)
Move each one into the workspace's secrets or the deployment's environment variables. To write a string that holds no credential, read it first and name its path with --not-a-secret.
```

A refused export writes nothing at all. A file with `<redacted>` in place of the credential would copy an assistant that cannot authenticate, and would hide from a reviewer that the source holds a credential.

Keep credentials out of an assistant's configuration, context and metadata. Put each one in the deployment's environment variables, which the graph reads, or in the workspace's secrets ([Workspace Secrets Management](workspace-secrets.md)).

### Write a flagged string that holds no secret

Some flagged strings hold no secret, such as `strict` under a setting called `token_budget_policy`, which the key-name rule flags for the word `token`. Read the string first. Then name its path with `--not-a-secret`, once for each path:

```bash
langstar assistant export <assistant-id> --deployment my-agent --output support-bot.json \
  --not-a-secret /config/configurable/token_budget_policy
```

Export writes that string and prints a warning naming the path to stderr. A flagged string at any path you did not name still stops the export. When `--not-a-secret` names a path where the check flags nothing, export names the path, writes no file and exits with status 1, so a script that keeps naming a path after the string there has changed stops at the export. Export runs the check on every run.

### What the check cannot catch

A credential under an ordinary key name with an ordinary-looking value, such as a random string under a key called `x`, passes the check and reaches the file. Read an export file before you commit it.

## Exit statuses

- 0: export wrote the file.
- 1: export wrote nothing, because the assistant holds a credential, a `--prompt-ref` or `--not-a-secret` pointer did not match, the deployment has no assistant with that ID, the assistant is a graph's default assistant, or another error stopped it.
- 2: a mistyped flag or a missing required one.
