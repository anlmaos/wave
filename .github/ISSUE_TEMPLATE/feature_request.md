name: Feature Request
description: Propose a new feature or enhancement for Animawave
title: "[Feature] "
labels: ["type:feature", "status:triage"]
body:
  - type: markdown
    attributes:
      value: |
        Already built something similar? Check existing open issues first, then sketch what you have in mind.
  - type: textarea
    id: why
    attributes:
      label: Motivation
      description: "What problem does this solve? Why should someone else care?"
      placeholder: "I wish Animawave could…"
    validations:
      required: true
  - type: textarea
    id: proposal
    attributes:
      label: Proposed solution
      description: Describe the behavior you'd like to see.
    validations:
      required: true
  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives considered
      description: Any workarounds, competing approaches, or features in other apps?
    validations:
      required: false
  - type: checkboxes
    id: area
    attributes:
      label: Primary area affected
      description: Check all that apply.
      options:
        - label: UI / UX (GTK/libadwaita widgets)
        - label: Audio pipeline (GStreamer, recording, casting)
        - label: Radio-browser API client
        - label: Database / station storage
        - label: Settings / preferences
        - label: Packaging / Flatpak
  - type: textarea
    id: additional
    attributes:
      label: Additional context
      description: Mockups, references to other apps, anything else?
    validations:
      required: false
