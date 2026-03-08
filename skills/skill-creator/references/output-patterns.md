# Output Patterns

Use these patterns when a skill needs to produce consistent, high-quality output.

## Template Pattern

Provide a template when the output structure matters.

Use a strict template when downstream consumers or validators expect fixed sections or exact formatting.

```markdown
## Report structure

ALWAYS use this exact template structure:

# [Analysis Title]

## Executive summary
[One-paragraph overview of key findings]

## Key findings
- Finding 1 with supporting data
- Finding 2 with supporting data
- Finding 3 with supporting data

## Recommendations
1. Specific actionable recommendation
2. Specific actionable recommendation
```

Use a softer template when adaptation is valuable and the agent should keep discretion:

```markdown
## Report structure

Here is a sensible default format, but use your best judgment:

# [Analysis Title]

## Executive summary
[Overview]

## Key findings
[Adapt sections based on what you discover]

## Recommendations
[Tailor to the specific context]
```

## Examples Pattern

If output quality depends on style or precedent, include input/output examples.

```markdown
## Commit message format

Generate commit messages following these examples:

Example 1
Input: Added user authentication with JWT tokens
Output:
feat(auth): implement JWT-based authentication

Add login endpoint and token validation middleware

Example 2
Input: Fixed bug where dates displayed incorrectly in reports
Output:
fix(reports): correct date formatting in timezone conversion

Use UTC timestamps consistently across report generation
```

Examples often communicate tone and shape better than abstract description.
