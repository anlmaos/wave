name: Bug Report
description: Report a reproducible defect in Animawave
title: "[Bug] "
labels: ["type:bug", "status:triage"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to file a bug! Please fill out the sections below so we can reproduce and fix it.
  - type: textarea
    id: what-happened
    attributes:
      label: What happened?
      description: Describe the bug and what you expected to happen.
      placeholder: "When I do X, Y actually happens. I expected Z."
    validations:
      required: true
  - type: textarea
    id: repro-steps
    attributes:
      label: Steps to reproduce
      description: "1. Go to…\n2. Click on…\n3. Scroll down to…\n4. See error"
      placeholder: "The more precise the steps, the faster we can pin down the cause."
    validations:
      required: true
  - type: dropdown
    id: version
    attributes:
      label: Version
      description: Which Animawave version are you running?
      options:
        - Development Flatpak (de.haeckerfelix.Animawave.Devel)
        - Stable Flatpak from Flathub
        - Built from source (git)
        - "Other (describe in Additional context)"
    validations:
      required: true
  - type: input
    id: distro
    attributes:
      label: OS / Distribution
      placeholder: "e.g. GNOME 46 on Fedora 40, GNOME Nightly"
    validations:
      required: false
  - type: textarea
    id: logs
    attributes:
      label: Relevant output
      description: Paste any error messages from the terminal or Flatpak logs (`flatpak run --command=journalctl de.haeckerfelix.Animawave.Devel`).
      render: shell
    validations:
      required: false
  - type: textarea
    id: additional
    attributes:
      label: Additional context
      description: "Anything else we should know? Screenshots are welcome."
    validations:
      required: false
