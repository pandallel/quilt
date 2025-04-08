# Quilt

A local-first, modular memory and context engine. Quilt watches your work, fragments your documents into meaningful pieces (swatches), embeds them into a searchable memory (the swatch book), and assembles contextual spreads in response to queries.

Use Quilt to power LLM tools with fast, structured, and evolving context—without relying on cloud infrastructure or leaking your knowledge.

## Features

- 📂 Watch folders for new or updated materials
- ✂️ Cut materials into meaningful swatches
- 🔢 Embed swatches using local models
- 📚 Store in a fast, local swatch book
- 🧠 Assemble contextual spreads for queries

## Getting Started

[Coming soon]

## Development

See the [Development Guide](docs/src/development/guide.md) for information on testing, linting, and contributing to Quilt.

### Automated Code Review

This repository uses OpenAI's GPT-4 for automated code reviews on pull requests created by the repository owner. To enable this functionality:

1. Generate an OpenAI API key from [OpenAI's platform](https://platform.openai.com/api-keys)
2. Add the API key as a repository secret named `OPENAI_API_KEY` in your GitHub repository settings
   (Settings > Secrets and variables > Actions > New repository secret)

The code review will automatically run on PRs and provide feedback as a comment.

## Documentation

See the [documentation](docs/book) for detailed information about Quilt's architecture and usage.
