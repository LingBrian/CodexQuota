## Summary

<!-- Describe what changed and why. -->

## Validation

- [ ] `npm run check`
- [ ] `npm test`
- [ ] `npm run build`
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] Windows UI/runtime checks, if applicable

## Safety and release notes

- [ ] No credentials, account data, or unredacted local paths are included.
- [ ] User-facing or configuration changes are documented.
