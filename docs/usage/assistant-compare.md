# Assistant Compare

`langstar assistant compare` compares an assistant on a deployment with the file that [`langstar assistant export`](assistant-export.md) wrote. It lists each field that differs, so you can see whether someone changed a copy on the deployment, or changed the file in Git, since the last import. It changes nothing on the deployment and writes no file.

To also check that the deployment serves the file's graph and that the workspace holds the prompts the assistant names, use [`langstar assistant verify`](assistant-verify.md).

## Compare an assistant

```bash
langstar assistant compare support-bot.json --deployment my-agent-prod
```

Compare reads the ID of the assistant to compare from the destination record, `support-bot.destinations.json` beside the export file, which [`langstar assistant import`](assistant-import.md) wrote when it created the assistant on that deployment. When you imported with `--record`, pass the same path:

```bash
langstar assistant compare support-bot.json --deployment my-agent-prod --record records/prod.json
```

To compare the file with another assistant, such as the one you exported it from, name the assistant with `--assistant-id`. Compare then reads no record:

```bash
langstar assistant compare support-bot.json --deployment my-agent --assistant-id <assistant-id>
```

When the record has no entry for the deployment and you pass no `--assistant-id`, compare names the deployment, suggests `--assistant-id` and exits with status 1. When the entry is still `pending`, an import stopped before it finished; import the file again to finish it.

To see every flag:

```bash
langstar assistant compare --help
```

## What compare compares

Compare reads the assistant from the deployment and compares it with the file's `assistant` object: the name, description, graph ID, configuration, context and metadata. It leaves out the assistant ID, version and timestamps, which the server sets.

The Agent Server keeps an assistant's settings in both `config.configurable` and `context`. When the file holds them in only one of those fields, compare reads the deployment's settings from whichever field holds them, so a file that import copied from an older deployment still matches.

Before it reads the deployment, compare checks that each pointer under the file's `requires.prompts` still holds the prompt name recorded with it. When someone edited the configuration in Git and left `requires.prompts` as it was, compare names the pointer and both values and exits with status 1.

## The report

Compare prints each difference by its JSON Pointer path, with the file's value and the deployment's:

```text
Differences

  /assistant/metadata/team
    file:        "support"
    destination: "billing"
```

Compare prints `<redacted>` in place of each value the credential check from [Assistant Export](assistant-export.md) flags, whether the value came from the file or from the deployment, and prints a secret reference, such as `{"id": ["OPENAI_API_KEY"], "lc": 1, "type": "secret"}`, unchanged.

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
