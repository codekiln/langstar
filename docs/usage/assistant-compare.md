# Assistant Compare

`langstar assistant compare` compares an assistant on a deployment with the file that [`langstar assistant export`](assistant-export.md) wrote. It lists each field that differs, so you can see whether someone changed a copy on the deployment, or changed the file in Git, since the last import. It changes nothing on the deployment and writes no file.

Compare and verify refuse to run while `LANGSTAR_DEBUG_HTTP` is set, so run `unset LANGSTAR_DEBUG_HTTP` first. With that variable set, langstar writes every server response to a debug file, and the assistant these commands read from the deployment may hold a credential.

To also check that the deployment serves the file's graph and that the workspace holds the prompts the assistant names, use [`langstar assistant verify`](assistant-verify.md).

## Compare an assistant

```bash
langstar assistant compare support-bot.json --deployment my-agent-prod
```

When [`langstar assistant import`](assistant-import.md) creates the assistant, it writes the new assistant's ID to a destination record, `support-bot.destinations.json`, beside the export file. Compare reads the ID from that record. When you imported with `--record`, pass the same path:

```bash
langstar assistant compare support-bot.json --deployment my-agent-prod --record records/prod.json
```

To compare the file with another assistant, such as the one you exported it from, name the assistant with `--assistant-id`. Compare then reads no record:

```bash
langstar assistant compare support-bot.json --deployment my-agent --assistant-id <assistant-id>
```

Without `--assistant-id`, compare needs a created assistant in the record, and exits with status 1 when it finds none. When the record has no entry for the deployment, compare names the deployment and suggests `--assistant-id`. A `pending` entry means an earlier import stopped before it created the assistant. Compare says so, and you can import the file again to finish creating the assistant, or pass `--assistant-id`.

To see every flag:

```bash
langstar assistant compare --help
```

## Fields in the comparison

Compare reads the assistant from the deployment and compares it with the file's `assistant` object: the name, description, graph ID, configuration, context and metadata. It leaves out the assistant ID, version and timestamps, which the server sets.

The Agent Server keeps an assistant's settings in both `config.configurable` and `context`, and export writes both. When you remove one of them from the file, compare checks the field the file still holds against the same settings on the deployment, and leaves the other field out of the comparison.

Before it reads the deployment, compare checks the file against itself. For each prompt, `requires.prompts` holds a JSON Pointer path into the file's `assistant` object and the prompt name export found at that path. When someone edits the prompt name in the configuration and leaves `requires.prompts` as it was, compare prints the path, the name `requires.prompts` holds and the name the configuration now holds, and exits with status 1.

## The report

Compare prints each difference by its JSON Pointer path, with the file's value and the deployment's:

```text
Differences

  /assistant/metadata/team
    file:        "support"
    destination: "billing"
```

Compare prints `<redacted>` in place of each value that the credential check from [Assistant Export](assistant-export.md) flags, on the file's side and on the deployment's. It prints a secret reference as it stands, so the report names the secret: `{"id": ["OPENAI_API_KEY"], "lc": 1, "type": "secret"}`.

With `-f json`, compare prints one JSON object to stdout and its progress messages to stderr:

```bash
langstar -f json assistant compare support-bot.json --deployment my-agent-prod
```

```json
{
  "checks": [],
  "differences": [{ "destination": "billing", "file": "support", "path": "/assistant/metadata/team" }],
  "status": "failed"
}
```

## Exit statuses

- 0: the assistant on the deployment matches the file.
- 1: the file is not an assistant export, its `requires.prompts` no longer matches its assistant, the record has no created assistant for the deployment and you passed no `--assistant-id`, the deployment has no assistant with that ID, `LANGSTAR_DEBUG_HTTP` is set, or another error, such as a network failure.
- 2: a mistyped flag or a missing required one.
- 3: the assistant on the deployment differs from the file.
