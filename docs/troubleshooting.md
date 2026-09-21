# Troubleshooting

## The scanner reports no findings on my Soroban project

- Confirm the project is recognized: the terminal header shows the project kind.
  If it says `generic_rust`, the scanner found no Soroban dependency or
  `#[contract]`/`#[contractimpl]`. Check that `Cargo.toml` declares
  `soroban-sdk` and that the contract macro is present.
- If the project is classified as `likely_soroban`, detectors that only apply to
  contract entry points will not run. This is intentional: the scanner does not
  guess.
- Check `--min-severity` / `--min-confidence` and any `disabled` rules.

## I disagree with a finding (false positive)

1. Read the rule's documentation: `soroban-scan explain SS-00X`.
2. Suppress it for the project if warranted:

   ```toml
   [rules]
   disabled = ["SS-006"]
   ```

3. If the rule is genuinely wrong, please open a bug report with a minimal
   example. False positives are treated as first-class bugs.

## A file or directory is not scanned

- `target/`, `.git/`, `node_modules/`, and `vendor/` are skipped by default.
- Check `exclude` globs in configuration.
- Symlinks are not followed by default; enable `follow_symlinks` only if you
  trust the target.

## "unknown rule id" error

The id does not exist. List valid ids with `soroban-scan rules`. Ids are of the
form `SS-###`.

## Configuration is rejected

Unknown fields are errors. Run `soroban-scan config validate` for a message, and
compare against [Configuration](configuration.md).

## `linker link.exe not found` (Windows)

Install the Visual Studio C++ Build Tools, or switch to the GNU toolchain:

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup override set stable-x86_64-pc-windows-gnu
```

## Build fails with an MSRV error

The workspace requires Rust `1.74` or newer. Update with `rustup update stable`.

## Exit code 1 in CI, but there are no "new" findings

Exit code `1` means a finding at or above `--fail-on` is present. Without a
baseline, that includes pre-existing findings. Add a baseline
(see [Baselines](baselines.md)) or set `--fail-on none`.

## SARIF upload works but no results appear on GitHub

- The upload step must have `security-events: write` permission.
- Fork pull requests cannot upload SARIF; guard the step with the fork check in
  [GitHub Integration](github-integration.md).
- Code scanning must be enabled for the repository (it is automatic for public
  repositories on first upload).

## The scan is slow on a large repository

Parsing dominates. See [Benchmarks](development/benchmarks.md) and exclude large
generated or vendor directories.

## Still stuck?

Open a discussion or issue on the
[repository](https://github.com/AyinkxLab/soroban-security-scanner/issues). Do
not report security vulnerabilities publicly; see
[SECURITY.md](../SECURITY.md).
