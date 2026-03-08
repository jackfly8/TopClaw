# Workflow Patterns

## Sequential Workflows

For complex tasks, give the agent a clear step order near the top of `SKILL.md`.

```markdown
Filling a PDF form involves these steps:

1. Analyze the form
2. Create field mapping
3. Validate mapping
4. Fill the form
5. Verify output
```

This works well when the task has a stable lifecycle and each step builds on the previous one.

## Conditional Workflows

For tasks with branching logic, guide the agent through the decision points first.

```markdown
1. Determine the modification type:
   Creating new content? -> Follow the creation workflow
   Editing existing content? -> Follow the editing workflow

2. Creation workflow: [steps]
3. Editing workflow: [steps]
```

Use this pattern when the same skill supports multiple distinct modes of operation.
