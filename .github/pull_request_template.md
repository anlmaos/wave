## Description

<!-- One-paragraph summary of what this PR does and why. -->

## Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all -- -D warnings` passes
- [ ] `cargo deny --log-level error check` passes
- [ ] `potfiles` check passes (if you touched source/UI files with translatable text)
- [ ] Manually tested the change on Flatpak (`flatpak-builder --run build de.haeckerfelix.Animawave.Devel.json --branch=devel animawave`)

## Testing done

<!-- What did you test and on what platform? -->

## Screenshots / recordings

<!-- If the change affects the UI, attach a screenshot or short screen recording. -->

## Related issues

<!-- Link any issues, e.g. Closes #123, Fixes #456. Use TODO(#NNN) in new code that should be revisited after an issue is merged. -->
