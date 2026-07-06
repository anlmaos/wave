name: Chore
description: Internal cleanup, dependency bump, refactor, or CI work
title: "[Chore] "
labels: ["type:chore", "tech-debt"]
body:
  - type: markdown
    attributes:
      value: |
        Use this template for tasks that don't change user-visible behavior.
  - type: textarea
    id: why
    attributes:
      label: Why is this needed?
      description: "Link a tracking issue if one exists."
    validations:
      required: true
  - type: textarea
    id: what
    attributes:
      label: What should change?
      description: Concrete, reviewable outcome.
    validations:
      required: true
  - type: checkboxes
    id: area
    attributes:
      label: Area
      description: Primary system(s) affected.
      options:
        - label: CI / packaging
        - label: Code refactoring / dead-code cleanup
        - label: Dependency bump
        - label: Documentation / tooling
        - label: Tests
  - type: textarea
    id: additional
    attributes:
      label: Additional context
      description: "Links to related issues, prior art, etc."
    validations:
      required: false
