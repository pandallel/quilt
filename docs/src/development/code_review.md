# Automated Code Review

Quilt uses automated AI code review on pull requests to improve code quality and maintain best practices.

## OpenAI Code Review Workflow

The project is configured with a GitHub Action that uses OpenAI's models to perform automated code reviews on pull requests when specifically requested. This helps catch issues early and ensures consistency with project standards.

### How It Works

1. When a PR comment containing `/ai-review` is posted by authorized users (currently only `monday-sun`), the OpenAI Code Review workflow is triggered
2. The workflow sends the code changes to OpenAI's service
3. The AI reviews the code changes focusing on:
   - Code correctness and reliability
   - Error handling and robustness
   - Performance considerations
   - Adherence to Rust best practices
   - Alignment with actor-based concurrency patterns
4. The review is posted as a comment on the pull request

### Usage

To request an AI review on a pull request:

1. Create a pull request or navigate to an existing one
2. Add a comment containing `/ai-review`
3. The workflow will run and post the AI's review as a new comment

This on-demand approach gives you control over when to use AI review and avoids unnecessary processing.

### Setup Requirements

To enable this functionality in your fork or if you're contributing to the project:

1. Generate an OpenAI API key from [OpenAI's platform](https://platform.openai.com/api-keys)
2. Add the API key as a repository secret named `OPENAI_API_KEY` in your GitHub repository settings
   (Settings > Secrets and variables > Actions > New repository secret)

### Configuration

The workflow is configured in `.github/workflows/openai_code_review.yml`. Key settings include:

- **Model**: Currently using the `4o-mini` model for a balance of quality and cost
- **Maximum Length**: Set to 8000 characters to allow for comprehensive reviews
- **User Filtering**: Only runs when requested by specific GitHub users
- **Trigger**: Activated by commenting `/ai-review` on a pull request

### Benefits

- **Consistent Reviews**: Ensures all code is reviewed using the same standards
- **Early Feedback**: Provides immediate feedback on pull requests
- **Reduced Review Burden**: Handles routine aspects of code review, allowing human reviewers to focus on higher-level concerns
- **Continuous Learning**: Helps contributors learn best practices with each submission
- **On-demand Usage**: Consumes API resources only when explicitly requested

### Limitations

- Reviews are limited to code changes visible in the PR (not the entire codebase)
- Very complex architectural issues might still require human review
- The system depends on the OpenAI API being available
