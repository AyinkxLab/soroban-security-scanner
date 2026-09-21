# Release Process

## Versioning

This project follows [Semantic Versioning](https://semver.org/). While the
version is `0.x`, minor releases may include breaking changes; they are always
documented in [`CHANGELOG.md`](../../CHANGELOG.md).

## Before releasing

Run the full gate:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo build --release
cargo run --release -p soroban-scan-core --example benchmark
```

CI must be green, including `cargo-deny` (advisories, licenses, bans, sources).

## Release checklist

1. Ensure `main` is green in CI.
2. Update `CHANGELOG.md`: move `Unreleased` entries under the new version and
   add the date.
3. Bump `version` in the workspace `Cargo.toml` (`[workspace.package]`).
4. Update `docs/PROJECT_STATUS.md`.
5. Commit with `chore(release): vX.Y.Z`.
6. Create an annotated, signed tag:

   ```bash
   git tag -a vX.Y.Z -m "Soroban Security Scanner vX.Y.Z"
   git push origin main --follow-tags
   ```

7. Create the GitHub release:

   ```bash
   gh release create vX.Y.Z --title "vX.Y.Z" --notes-file CHANGELOG.md
   ```

8. Verify the release and tag on GitHub.

## Artifacts

Source is the primary artifact. Prebuilt binaries for Linux, macOS, and Windows
are attached to releases once a cross-compilation workflow is added (tracked as
future work).

## Package publishing

Publishing to crates.io is not performed yet. Any future publishing must ensure
that `soroban-scan-core` and `soroban-scan-cli` versions stay in lockstep.

## Post-release

- Announce in the repository discussions.
- Open a fresh `Unreleased` section in the changelog.
