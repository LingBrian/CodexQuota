# Third-party notices

CodexQuota is MIT licensed. Its dependency licenses are resolved from Cargo.lock and package-lock.json during a reproducible build and must be reviewed before release.

The reference project [xjoker/codex-switch](https://github.com/xjoker/codex-switch), version 20261001.5.0 at commit `fe14af0d6b9317a464493816f8bd074eccfae785`, is distributed as a separate CLI executable. Its MIT license states Copyright (c) 2025 xjoker. CodexQuota invokes this executable rather than copying its credential implementation. The bundle includes `licenses/codex-switch/LICENSE` and the OpenAI Apache-2.0 assets' `licenses/upstream-codex/LICENSE` and `NOTICE.md`. Source and patched dependency lock checksums are enforced by `scripts/prepare-cli.ps1`; the patch upgrades rustls and quinn-proto only. Protocol provenance and limits are in docs/UPSTREAM_RESEARCH.md; security results are in docs/SECURITY_REVIEW.md. Reference checkouts, executable build products and credentials are excluded from the source repository.

Tauri, Vue, Vite, Pinia, Tokio, Serde and their transitive dependencies remain subject to their own licenses.
