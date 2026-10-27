# Export, import, compare and verify prompts and assistants

Issue: [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807)

## Why

Moving an application to a new LangSmith workspace and a new deployment means recreating its prompts and its assistants there. Today a user does that by hand in the web interface, or with `langstar prompt push`, which builds a single-message prompt and drops the model, the system message and any placeholders. Nothing checks afterwards that the copy matches the original or that it can still run.

A prompt belongs to a workspace and an assistant belongs to a deployment, but both need the same four operations: write the definition to a file that can live in Git, recreate it somewhere else, compare a copy with that file, and check that the copy has what it needs to run.

## What Changes

- `langstar prompt export <handle>` writes one JSON file holding a saved prompt commit as LangSmith returns it with the model included: message templates, input variables, output schema, structured output method, model binding and commit description, plus the repo's description, readme, tags and visibility, the commit tags, and where it came from.
- `langstar prompt import <file> --workspace-id <id>` creates the prompt in the destination workspace, or adds a commit to an existing prompt, updates its repo details and creates, moves or deletes its commit tags when the user passes `--update`. An import that stopped before it pushed the first commit finishes on the next run, and one that stopped before it created the commit tags finishes with `--update`. `--dry-run` shows what it would do. After writing, it reads the prompt back and reports any difference.
- `langstar prompt compare <file>` reports the differences between the file and a prompt in a workspace. `langstar prompt verify <file>` also checks that the workspace has the secrets the prompt's model reads, and notes when the workspace has no saved model configuration for the model's class, such as `ChatOpenAI`.
- `langstar assistant export`, `import`, `compare` and `verify` do the same for assistants on a deployment. The file holds the name, description, graph ID, configuration, context, metadata and any prompt names the assistant refers to. `verify` checks that the destination deployment has the graph and that the destination workspace has the referenced prompts.
- `prompt export` and `assistant export` write no file when the prompt or assistant holds a credential. Before writing, export checks every string for a key name such as `api_key`, `token`, `password`, `secret` or `Authorization`, and for a value shaped like a bearer token, a private key, a URL or connection string with a password, a JSON Web Token or a provider's API key. Export names the path of each flagged string and exits with status 1. A user who has read a flagged string and found it harmless passes `--not-a-secret <json-pointer>` to write it. Compare, verify and import print `<redacted>` in place of the same strings.
- Import records each destination ID (the new commit hash, or the new assistant ID) in a separate file next to the export. When a user imports the same file again, langstar reads that file to find the assistant it created earlier instead of creating another one, and to tell whether someone pushed a commit to the prompt since the last import.
- Import runs the dependency checks of `verify` before it writes, and writes nothing when one fails.
- `compare`, `verify` and `import` share one exit status scheme, 0 for a match, 3 for a difference or a failed check and 1 for any other error, and print a JSON report with `-f json`. Status 2 stays clap's usage error.

## Capabilities

### New Capabilities

- `prompt`: the `langstar prompt` command group. This change adds its export, import, compare and verify requirements; the existing prompt subcommands get their requirements when [#728 Implement CRUD command structure (create, get, update) for prompt commands](https://github.com/codekiln/langstar/issues/728) lands.

### Modified Capabilities

- `assistant`: adds export, import, compare and verify requirements to `langstar assistant`.

## Impact

- SDK: `sdk/src/prompts.rs` gains the prompt calls export and import need: a commit read that asks for the model and returns it on `CommitManifestResponse`, a commit create that sends a description and returns the new commit's `id`, a commit list, calls that list, create, move and delete commit tags, and a repo update. `Prompt` gains the repo's readme, tags and `tenant_id`, and export fills `source.workspace_id` from `tenant_id`. `sdk/src/assistants.rs` gains the `context` and `version` fields, `assistant_id` and `if_exists` on create, `graph_id` on update, and a search by graph ID. These are `✨ feat(sdk)` commits with no `BREAKING CHANGE:` footer, so the Prepare Release workflow makes the next release a minor version, as [SDK field additions ship as `✨ feat` commits](design.md#sdk-field-additions-ship-as--feat-commits) explains from `docs/dev/git-scm-conventions.md`, `docs/dev/ci-cd.md` and `docs/dev/procedures.md`.
- CLI: new subcommands in `cli/src/commands/prompt.rs` and `cli/src/commands/assistant.rs`, and a shared module for the export file, the destination record, comparison and the exit status.
- Tests need a second workspace, a second deployment and one model provider key for the structured output check. codekiln creates the workspace and the key, and langstar's tests read them through `fnox.toml` and the CI workflow; the destination deployment fixture from [export-import-prompts-and-assistants Task 2.4 Add a destination deployment fixture](tasks.md#L19) creates the deployment whenever the cleanup workflow has deleted it, as the source fixture does today. Nobody knows yet whether the control plane accepts a second deployment from this repository. In [export-import-prompts-and-assistants Task 2.3 Register the test graph under a second graph ID](tasks.md#L18), a developer deploys the new `langgraph.json` once to find out, before anyone builds the fixture.
- Docs: a new usage page on moving prompts and assistants between environments.

## Citations

- [My/Principle/Declarative over Imperative](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Declarative%20over%20Imperative.md)
- [My/Principle/Explicit is better than Implicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Explicit%20is%20better%20than%20Implicit.md)
- [My/Principle/Simplify/Create Uniform Interfaces](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Create%20Uniform%20Interfaces.md)
- [My/Principle/Make the Right Thing Easy and the Wrong Thing Hard](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20the%20Right%20Thing%20Easy%20and%20the%20Wrong%20Thing%20Hard.md)
- [My/Principle/Simplify/Prefer Standards and Defaults](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Prefer%20Standards%20and%20Defaults.md)
