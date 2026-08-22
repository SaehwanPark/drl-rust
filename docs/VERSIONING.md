# Project versioning

DRL-Rust currently reports version `0.2.12` and uses the three non-negative
integer components in `VERSION` as its
canonical project version. The workspace package metadata, MCP server metadata,
and generated release manifest must agree with that value.

The repository had no release tags before this policy was adopted. Its existing
untagged `0.1.0` package metadata is therefore the retrospective baseline; the
first versioned codebase change (this adoption slice) is recorded as `0.1.1`.
Earlier untagged history remains one baseline rather than being rewritten into
invented releases.

Version transitions follow this exact rule:

- `x.y.z` contains only non-negative integer components.
- A major release increments `x` and resets `y` and `z` to zero.
- A significant change increments `y` and resets `z` to zero.
- A codebase change increments `z`.
- Documentation-only and setting-only changes do not increment any component.
- Components never carry automatically when they pass 10; values such as
  `0.12.10` are valid.

For a pull request, `scripts/check-version.sh` compares the candidate against
`DRL_VERSION_BASE` (the base commit in CI). It requires exactly one allowed
component transition when code paths change and rejects a version bump for a
documentation-only or setting-only diff. Without a comparison base it still
checks the canonical value and all package/manifest projections.
