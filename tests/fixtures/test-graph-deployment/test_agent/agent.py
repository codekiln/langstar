"""Minimal test graph for Langstar integration testing.

This module provides a simple echo graph that can be deployed to LangGraph Cloud
for testing the Langstar SDK and CLI assistant functionality.
"""

from typing import TypedDict
from langgraph.graph import StateGraph, END


class State(TypedDict):
    """Minimal state for testing.

    Attributes:
        message: The input message to be echoed back
    """
    message: str


def echo_node(state: State) -> State:
    """Simple echo node that prefixes the message.

    Args:
        state: The current graph state

    Returns:
        Updated state with echoed message
    """
    return {"message": f"Echo: {state['message']}"}


# Build the graph
builder = StateGraph(State)

# Add nodes
builder.add_node("echo", echo_node)

# Define edges
builder.set_entry_point("echo")
builder.add_edge("echo", END)

# Compile the graph
# IMPORTANT: Must be named 'graph' to match langgraph.json reference
graph = builder.compile()
