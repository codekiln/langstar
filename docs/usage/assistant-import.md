# Assistant Import

`langstar assistant import` creates an assistant on a deployment from a file that [`langstar assistant export`](assistant-export.md) wrote. With the two commands you copy an assistant from one deployment to another, and keep the file in Git between them.

To move to a new deployment in the same workspace, you copy the assistants and leave the prompts where they are: every deployment in a workspace reads the same prompts, while each assistant belongs to one deployment and gets a new ID on each copy.

## Import an assistant

```bash
langstar assistant import support-bot.json --deployment my-agent-prod
```

Before import creates anything, it checks the destination: the deployment has to serve the file's graph ID, and the workspace has to hold each prompt the file lists under `requires.prompts`. When either check fails, import names what is missing, creates nothing and exits with status 3. Then:

1. It chooses an ID for the new assistant and writes it to the destination record, `support-bot.destinations.json` beside the export file.
2. It creates the assistant with that ID and the file's name, description, graph ID, settings and metadata, and marks the ID created in the record.
3. It reads the assistant back from the deployment and compares it with the file. It prints each field that differs, by its JSON Pointer path, and exits with status 3 when one does.

Import looks for the prompts in the workspace you set with `--workspace-id`, `LANGSMITH_WORKSPACE_ID` or the config file, which is also where it looks for the deployment. When you set none of them and your API key reaches several workspaces, import looks in the workspace that holds the deployment.

To see every flag:

```bash
langstar assistant import --help
```

## The destination record

For each deployment you import the file onto, the record holds the ID of the assistant that import created there:

```json
{
  "destinations": {
    "<deployment-id>": { "assistant_id": "<new-assistant-id>", "state": "created" }
  },
  "format_version": 1,
  "kind": "assistant"
}
```

When you import the same file onto the same deployment again, import finds that assistant through the record, compares it with the file and creates nothing. It reports the action `unchanged` and exits with status 0 when they match and 3 when they differ. [`langstar assistant compare`](assistant-compare.md) and [`langstar assistant verify`](assistant-verify.md) read the same record to find the copy. Keep the record in Git beside the export file. Without it, a second import creates a second assistant.

Import writes the ID to the record as `pending` before it sends the create, and marks it `created` once the create succeeds. If langstar stops in between, the next import sends the same create with the same ID, and the deployment answers with the assistant the first import made instead of creating another.

Run one import of a file at a time. Two imports of the same file that run at once each read the record before the other writes it, so the one that writes last drops the other's entry, and two first imports onto one deployment can create two assistants. [#828 make concurrent prompt and assistant imports into one deployment safe](https://github.com/codekiln/langstar/issues/828) tracks making that safe.

To keep the record somewhere else, name the path with `--record`:

```bash
langstar assistant import support-bot.json --deployment my-agent-prod --record records/prod.json
```

When the record names an assistant that someone has since deleted from the deployment, import says so and exits with status 1. Remove that deployment's entry from the record to create the assistant again.

## Import sends settings in config.configurable or context, never both

The Agent Server keeps an assistant's settings in both `config.configurable` and `context`, and export writes both. The Agent Server answers a create that sets both with HTTP 400 and the message `Cannot specify both configurable and context`, so import sends one: `config.configurable` when the file holds the settings there, and `context` when only `context` holds them. The server fills the other field in itself, so the read-back matches the file. When the file's two fields hold different settings, import cannot tell which one you meant. It names both paths, creates nothing and exits with status 1.

## Machine-readable output

With `-f json`, import prints one JSON object to stdout and its progress messages to stderr:

```json
{
  "action": "create",
  "checks": [{ "name": "graph", "outcome": "passed", "detail": "The deployment serves graph 'agent'." }],
  "destination_id": "<new-assistant-id>",
  "differences": [],
  "status": "passed"
}
```

Each difference names its `path` and the `file` and `destination` values. Import prints `<redacted>` in place of any value the credential check from [Assistant Export](assistant-export.md) flags.

## Copy every assistant from one deployment to another

Follow these steps to copy the assistants you created on an old deployment onto a new deployment in the same workspace, and then to check the copies. They use `jq`. Replace `<old-deployment>` and `<new-deployment>` with each deployment's name or ID from `langstar deployment list`. When your API key reaches several workspaces, add `--workspace-id <workspace-id>` to each command.

1. Unset `LANGSTAR_DEBUG_HTTP`. It makes langstar write each response, credentials included, to a debug file, so export and import refuse to run while it is set:

   ```bash
   unset LANGSTAR_DEBUG_HTTP
   ```

2. List the assistants on the old deployment, leaving out the default assistant the Agent Server makes for each graph:

   ```bash
   langstar -f json assistant list --deployment <old-deployment> --limit 100 \
     | jq -r '.assistants[] | "\(if .metadata.created_by == "system" then "default" else "custom" end) \(.assistant_id)"' > old-assistants.txt
   wc -l < old-assistants.txt
   awk '$1 == "custom" { print $2 }' old-assistants.txt > assistant-ids.txt
   ```

   `jq` writes one line per assistant: `default` for an assistant the Agent Server made for a graph, `custom` for any other, then the assistant's ID. It drops everything else, so the only assistant settings that reach the disk are the ones `langstar assistant export` has checked for credentials. `wc -l` counts every assistant on the page, default assistants included. When it prints 100, list the next page with `--offset 100`, add its IDs to `assistant-ids.txt` the same way, and go on until a page holds fewer than 100.

3. Find the assistants the new deployment already holds. Import creates a new assistant without looking for one with the same graph ID and name, so copying an assistant the new deployment already has leaves two, and code that looks an assistant up by graph ID and name finds both. The `pairs` function prints the graph ID and name of each assistant on a deployment, and `comm -12` keeps the lines both deployments print:

   ```bash
   pairs() {
     langstar -f json assistant list --deployment "$1" --limit 100 \
       | jq -r '.assistants[] | select(.metadata.created_by != "system") | "\(.graph_id) \(.name)"' | sort
   }
   comm -12 <(pairs <old-deployment>) <(pairs <new-deployment>)
   ```

   `pairs` reads the first 100 assistants on each deployment; on a larger deployment, list the next page with `--offset 100` as well.

   When it prints a pair, the new deployment already holds that assistant. Take its ID out of `assistant-ids.txt`, or delete that assistant with `langstar assistant delete` before you import.

4. Export each assistant to a file:

   ```bash
   mkdir -p assistants
   while read -r id; do
     langstar assistant export "$id" --deployment <old-deployment> --output "assistants/$id.json" < /dev/null
   done < assistant-ids.txt
   ```

   Read each file before you commit it. When export refuses an assistant because it holds a credential, move the credential into the deployment's environment variables or the workspace's secrets, as [Assistant Export](assistant-export.md) says.

5. Import each file onto the new deployment:

   ```bash
   while read -r id; do
     langstar assistant import "assistants/$id.json" --deployment <new-deployment> < /dev/null \
       || echo "$id: import exited with status $?"
   done < assistant-ids.txt
   ```

   Each import prints the new assistant's ID. An import that exits with 3 printed what is missing or what differs. Fix that and run the loop again. On the next run, import reads each record and compares the assistant it created earlier with the file instead of creating it again.

6. Check the copies with [`langstar assistant verify`](assistant-verify.md). Each one should report the status `passed`:

   ```bash
   while read -r id; do
     langstar -f json assistant verify "assistants/$id.json" --deployment <new-deployment> < /dev/null \
       | jq -r --arg id "$id" '"\($id) \(.status)"'
   done < assistant-ids.txt
   ```

   Then list each graph ID and name that appears more than once on the new deployment. An empty result means every copy is the only assistant with its graph ID and name:

   ```bash
   pairs <new-deployment> | uniq -d
   ```

7. Commit `assistants/`, with each export file and its `.destinations.json` record. The records hold the new assistant IDs, so the next import of a changed file finds the copy instead of making another.

The copies have new assistant IDs. Code that refers to an assistant by ID has to use the new ID from the record; code that finds it by graph ID and name keeps working.

## Exit statuses

- 0: the assistant on the deployment matches the file, and every check passed.
- 1: the file is not an assistant export, its `requires.prompts` no longer matches its assistant, its two settings fields differ, the record names an assistant the deployment no longer has, or `LANGSTAR_DEBUG_HTTP` is set; import created nothing in these cases. Another error, such as a network failure, also exits with 1; when it comes after the create, the record holds the new ID and the next import finds that assistant.
- 2: a mistyped flag or a missing required one.
- 3: a check failed and import created nothing, or the assistant on the deployment differs from the file. When the copy differs from the file, the record keeps the ID of the assistant that import created, so the next import compares that assistant instead of creating another.

## Planned: import's --update

The langstar developers will add import's `--update`, `--assistant-id` and `--dry-run` flags under [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807). Until `--update` exists, import only compares an assistant it finds through the record; to change a copy, edit it with `langstar assistant update`.
