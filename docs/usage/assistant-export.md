# Assistant Export

`langstar assistant export` writes one assistant to a JSON file. You can keep the file in Git, so a reviewer sees each change to the assistant in a pull request. [`langstar assistant import`](assistant-import.md) recreates the assistant from that file on another deployment.

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
- `source` names the deployment, the assistant ID and the assistant's version. Export leaves the assistant ID, the version and the server's timestamps out of `assistant`, so the `assistant` object of a copy on another deployment differs from this one only where a setting differs.
- `requires.prompts` lists the prompts the assistant refers to, as the next section shows.

Exporting the same assistant version twice writes the same file, because export sorts the keys and records no export time. Git shows a change to the file only when the assistant changed.

## Record the prompts an assistant refers to

Each graph reads its prompt name from a configuration key its author chose, so langstar cannot tell which strings name a prompt. Name each one with `--prompt-ref` and the JSON Pointer, inside the assistant, of the string that holds the prompt name:

```bash
langstar assistant export <assistant-id> --deployment my-agent --output support-bot.json \
  --prompt-ref /config/configurable/system_prompt
```

The string at that pointer holds a prompt name of the form `[<owner>/]<handle>[:<tag>]`, such as `support-system:prod`. A name with no owner, or with the owner `-`, means the prompt in the current workspace. A name with no tag means the prompt's latest commit. Export writes each pointer and the name it holds under `requires.prompts`. When a pointer names no string in the assistant, export names the pointer, writes no file and exits with status 1.

When you set a prompt name in the assistant's configuration, give the prompt's handle and a tag, such as `support-system:prod`. `langstar prompt import` will copy a prompt's tags into the workspace it imports to, so `support-system:prod` names the same prompt version in both workspaces. A commit hash differs in each workspace.

## A graph's default assistant

Export refuses a graph's default assistant, because the Agent Server creates one for each graph on every deployment. It says why, writes no file and exits with status 1. Create an assistant with `langstar assistant create` and export that one.

## Export refuses to write a credential

A secret pushed to Git stays in the repository's history after someone deletes the file. So before it writes anything, export runs a credential check over every string in the assistant: its name, description, configuration, context and metadata, at any depth. The check flags a string by the name of its key, such as `openai_api_key` or `Authorization`, or by the shape of its value, such as a bearer token or a URL with a password in it. The [prompt export page](prompt-export.md) lists every key name and value shape the check flags.

The check skips the empty string and a LangChain secret reference such as `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`, which holds the name of a secret and never its value.

When the check flags a string, export names the JSON Pointer path of each one and the rule that flagged it, leaves the string out of the message, writes no file and nothing to stdout, and exits with status 1:

```text
Error: This would write a credential to the export file:
  /config/configurable/OPENAI_API_KEY (key name)
Move each one into the workspace's secrets or the deployment's environment variables. To write a string that holds no credential, read it first and name its path with --not-a-secret.
```

Keep credentials out of an assistant's configuration, context and metadata. Put each one in the deployment's environment variables, which the graph reads, or in the workspace's secrets ([Workspace Secrets Management](workspace-secrets.md)).

### Write a flagged string that holds no secret

Some flagged strings hold no secret, such as `strict` under a setting called `token_budget_policy`, which the key-name rule flags for the word `token`. Read the string first. Then name its path with `--not-a-secret`, once for each path. The Agent Server returns each setting in both `config.configurable` and `context`, so a setting has two paths:

```bash
langstar assistant export <assistant-id> --deployment my-agent --output support-bot.json \
  --not-a-secret /config/configurable/token_budget_policy \
  --not-a-secret /context/token_budget_policy
```

Export writes that string and prints a warning naming the path to stderr. A flagged string at any path you did not name still stops the export. When the check flags nothing at a path you named with `--not-a-secret`, export names that path, writes no file and exits with status 1. When the check still flags the string at a path you named, export writes whatever that string now holds and prints the warning, so read the string again each time the assistant changes.

### What the check cannot catch

A credential under an ordinary key name with an ordinary-looking value, such as a random string under a key called `x`, passes the check and reaches the file. Read an export file before you commit it.

## Exit statuses

- 0: export wrote the file.
- 1: export wrote nothing, because the assistant holds a credential, a `--prompt-ref` pointer named no string, a `--not-a-secret` pointer named a path where the check flagged nothing, the deployment has no assistant with that ID, the assistant is a graph's default assistant, or another error stopped it.
- 2: a mistyped flag or a missing required one.
