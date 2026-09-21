# Wave Board (first program)

A curated, single-cycle subset of the backlog for the first Drips Wave, chosen
to be **independent**, **completable in one cycle**, and **representative** of
the project. It is deliberately a subset of the full 170-issue backlog, with a
bounded points budget.

Point values are assigned in the Drips Wave maintainer dashboard (Trivial 100 /
Medium 150 / High 200). The labels here are the honest pre-estimate.

## Budget

| Complexity | Issues | Points each | Subtotal |
| ---------- | -----: | ----------: | -------: |
| Trivial | 9 | 100 | 900 |
| Medium | 9 | 150 | 1,350 |
| High | 2 | 200 | 400 |
| **Total** | **20** | | **2,650** |

## Good first issues (100 points)

| # | Title |
| - | ----- |
| [#30](https://github.com/AyinkxLab/soroban-security-scanner/issues/30) | test(corpus): add a second negative fixture for SS-003 |
| [#31](https://github.com/AyinkxLab/soroban-security-scanner/issues/31) | feat(cli): add usage examples to `scan --help` |
| [#32](https://github.com/AyinkxLab/soroban-security-scanner/issues/32) | docs: add a troubleshooting entry for findings on unchanged lines |
| [#33](https://github.com/AyinkxLab/soroban-security-scanner/issues/33) | test(core): cover severity and confidence parsing aliases |
| [#34](https://github.com/AyinkxLab/soroban-security-scanner/issues/34) | docs: add a short quickstart to the README |
| [#46](https://github.com/AyinkxLab/soroban-security-scanner/issues/46) | feat(parser): expose doc comments as evidence |
| [#8](https://github.com/AyinkxLab/soroban-security-scanner/issues/8) | feat(cli): support `--max-findings` to bound report size |
| [#10](https://github.com/AyinkxLab/soroban-security-scanner/issues/10) | feat(cli): add `--no-config` to ignore discovered configuration |
| [#63](https://github.com/AyinkxLab/soroban-security-scanner/issues/63) | feat(report): add severity summary and top rules to Markdown reports |

## Medium (150 points)

| # | Title |
| - | ----- |
| [#171](https://github.com/AyinkxLab/soroban-security-scanner/issues/171) | feat(parser): extract direct intra-file function calls |
| [#4](https://github.com/AyinkxLab/soroban-security-scanner/issues/4) | feat(cli): add `--format github` workflow annotations |
| [#5](https://github.com/AyinkxLab/soroban-security-scanner/issues/5) | feat(cli): add shell completion generation |
| [#24](https://github.com/AyinkxLab/soroban-security-scanner/issues/24) | docs: add a rule cookbook with end-to-end detection examples |
| [#25](https://github.com/AyinkxLab/soroban-security-scanner/issues/25) | docs: explain the detection methodology and its limits |
| [#59](https://github.com/AyinkxLab/soroban-security-scanner/issues/59) | feat(report): add JUnit XML output for CI test reporters |
| [#66](https://github.com/AyinkxLab/soroban-security-scanner/issues/66) | test(report): golden snapshot tests for JSON and SARIF |
| [#67](https://github.com/AyinkxLab/soroban-security-scanner/issues/67) | test(corpus): add at least three negative fixtures per rule |
| [#70](https://github.com/AyinkxLab/soroban-security-scanner/issues/70) | test(core): cross-platform path normalization tests |

## High (200 points)

| # | Title |
| - | ----- |
| [#41](https://github.com/AyinkxLab/soroban-security-scanner/issues/41) | feat(parser): resolve `use` imports to qualify call paths |
| [#64](https://github.com/AyinkxLab/soroban-security-scanner/issues/64) | test(parser): add a cargo-fuzz target for source parsing |

## Notes

- No issue in this set depends on another; each can be merged independently.
- Contributors must request assignment before starting; see [`WAVE.md`](WAVE.md).
- After repository approval, add these issues to the Program. Apply the Program
  label manually or select them in the **Maintainers → Issues** dashboard.
- Keep the budget bounded; do not add all 170 issues to a single Wave.
