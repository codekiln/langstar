# Assistant Verify

`langstar assistant verify` checks that an assistant can run on a deployment. It compares the assistant with the file as [`langstar assistant compare`](assistant-compare.md) does, and also checks that the deployment serves the file's graph and that the workspace holds each prompt the file lists under `requires.prompts`. These are the checks that [`langstar assistant import`](assistant-import.md) runs before it creates anything. Verify changes nothing on the deployment and writes no file.

## Verify an assistant

```bash
langstar assistant verify support-bot.json --deployment my-agent-prod
```

Verify finds the assistant the way compare does: through the destination record, `support-bot.destinations.json` beside the export file, unless you name another record with `--record` or name the assistant with `--assistant-id`:

```bash
langstar assistant verify support-bot.json --deployment my-agent-prod --record records/prod.json
langstar assistant verify support-bot.json --deployment my-agent-prod --assistant-id <assistant-id>
```

Verify looks for the prompts in the workspace you set with `--workspace-id`, `LANGSMITH_WORKSPACE_ID` or the config file, as import does. When you imported the prompts into another workspace, name it:

```bash
langstar assistant verify support-bot.json --deployment my-agent-prod --workspace-id <workspace-id>
```

To see every flag:

```bash
langstar assistant verify --help
```

## Graph and prompt checks

- `graph`: the deployment serves the file's graph ID. When it does not, verify names the missing graph ID. When the server's answer lists the graph IDs the deployment does serve, verify prints those too.
- `prompt`: the workspace holds each prompt under `requires.prompts`, at the tag the name gives. When a name has no owner, such as `support-system:prod`, verify looks for the prompt in the workspace. When a name has no tag, such as `support-system`, verify checks the prompt's latest commit.

Verify runs each check, then compares the assistant with the file as compare does. Verify prints every failed check and every field that differs, and exits with status 3 when it finds either.

## The report

```text
No differences.

Checks

  passed  graph: The deployment serves graph 'agent'.
  failed  prompt: The workspace has no prompt support-system:prod.
```

With `-f json`, verify prints one JSON object with `status`, `differences` and `checks` to stdout, and its progress messages to stderr:

```bash
langstar -f json assistant verify support-bot.json --deployment my-agent-prod
```

```json
{
  "checks": [
    { "detail": "The deployment serves graph 'agent'.", "name": "graph", "outcome": "passed" },
    { "detail": "The workspace has no prompt support-system:prod.", "name": "prompt", "outcome": "failed" }
  ],
  "differences": [],
  "status": "failed"
}
```

As compare does, verify prints `<redacted>` in place of each value the credential check from [Assistant Export](assistant-export.md) flags, and prints a secret reference unchanged.

## Exit statuses

- 0: the assistant on the deployment matches the file, and every check passed.
- 1: any error that makes [compare](assistant-compare.md#exit-statuses) exit with 1. Verify also exits with 1 when the server answers a graph or prompt lookup with an error other than 'not found', such as a server error.
- 2: a mistyped flag or a missing required one.
- 3: a check failed, or the assistant on the deployment differs from the file.
