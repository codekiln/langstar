# Developer Documentation

This directory contains instructions and conventions for developing the Langstar project.

## Contents

- [GitHub Workflow](./github-workflow.md) - Complete guide to the GitHub issue-driven development workflow
- [GitHub Projects](./github-projects.md) - GitHub Projects V2 configuration, fields, and API usage
- [Git SCM Conventions](./git-scm-conventions.md) - Guidelines for commit messages and version control practices
- [Spec-Kit Integration](./spec-kit.md) - Spec-driven development with GitHub Spec-Kit

## Purpose

These documents outline the coding conventions, best practices, and standards that all contributors should follow when working on this project. Please review these guidelines before making commits or submitting pull requests.

## Tips from Development - Memories
- This project uses a devcontainer to standardize the development environment. Never configure the environment in a 1-off way, unless running a 1-off test. Always prefer modifications to .devcontainer folder and related assets.
- <rules>
for ANY question about the langchain ecosystem (langsmith, LangGraph, langchain) use the langgraph-docs-mcp server to help answer -- 
+ call list_doc_sources tool to get the available llms.txt file
+ call fetch_docs tool to read it
+ reflect on the urls in llms.txt 
+ reflect on the input question 
+ call fetch_docs on any urls relevant to the question
</rules>