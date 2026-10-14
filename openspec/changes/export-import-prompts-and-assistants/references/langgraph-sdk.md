# LangGraph Python SDK assistants client

Read at [langgraph 8a77332d](https://github.com/langchain-ai/langgraph/tree/8a77332d0db75c49d296b1f3833a3a5ccd5f0dc8).

[`create`](https://github.com/langchain-ai/langgraph/blob/8a77332d0db75c49d296b1f3833a3a5ccd5f0dc8/libs/sdk-py/langgraph_sdk/_async/assistants.py#L314) posts `graph_id`, `config`, `context`, `metadata`, `assistant_id`, `if_exists` (`raise` or `do_nothing`), `name` and `description` to `/assistants`.

[`update`](https://github.com/langchain-ai/langgraph/blob/8a77332d0db75c49d296b1f3833a3a5ccd5f0dc8/libs/sdk-py/langgraph_sdk/_async/assistants.py#L385) patches `graph_id`, `config`, `context`, `metadata`, `name` and `description`, and the server merges `metadata` into the existing metadata. [`get_versions`](https://github.com/langchain-ai/langgraph/blob/8a77332d0db75c49d296b1f3833a3a5ccd5f0dc8/libs/sdk-py/langgraph_sdk/_async/assistants.py#L656) lists the versions updates create.

[`AssistantBase`](https://github.com/langchain-ai/langgraph/blob/8a77332d0db75c49d296b1f3833a3a5ccd5f0dc8/libs/sdk-py/langgraph_sdk/schema.py#L246) returns `assistant_id`, `graph_id`, `config`, `context`, `created_at`, `metadata`, `version`, `name` and `description`. `context` sits beside `config.configurable`; newer graphs read it and older ones read `configurable`.

There is no endpoint that lists graphs. [`search`](https://github.com/langchain-ai/langgraph/blob/8a77332d0db75c49d296b1f3833a3a5ccd5f0dc8/libs/sdk-py/langgraph_sdk/_async/assistants.py#L528) filters by `graph_id`, and every graph has a default assistant with `metadata.created_by` set to `system`, as the [`get` docstring](https://github.com/langchain-ai/langgraph/blob/8a77332d0db75c49d296b1f3833a3a5ccd5f0dc8/libs/sdk-py/langgraph_sdk/_async/assistants.py#L81) shows.

What langstar takes from it: the fields an assistant export carries, including `context`, and the graph check through search.
