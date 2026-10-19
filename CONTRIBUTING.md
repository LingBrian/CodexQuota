# Contributing

Use Chinese or English issues and pull requests. Do not commit credentials, `auth.json`, token files, generated bundles, or local caches. Run `npm run check`, `npm run build`, `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` before requesting review.

Changes to quota parsing must include a sanitized real CLI fixture, a source reference, parser tests, and evidence that unavailable data cannot appear as full quota.
