# Run

- Set the `GITHUB_TOKEN` environment variable.
- Optional: set `LLM=true` for concise CI output.
- Run `cargo build`, then execute the tool inside a git repository whose CI status you want to check.

# Features

1. Fetches CI status from GitHub Actions.
2. Displays the status of all workflows (success/failure).
3. Saves CI report files to local output.
4. Downloads logs for failed jobs only (instead of all jobs in a failed workflow run).

# Project structure

```text
gh-ci-tool/
|-- src/
|   |-- main.rs        # orchestrates CI fetch/report/log workflow
|   |-- args.rs        # CLI arguments
|   |-- models.rs      # CI workflow/job data models + status helpers
|   |-- output.rs      # output mode/profile (llm vs human)
|   |-- report.rs      # human + llm report rendering
|   |-- logs.rs        # job log download helpers
|   |-- repo.rs        # git branch/commit/repo parsing helpers
|-- Cargo.toml
|-- README.md
```

Log locations:

- CI status JSON (compact, no pretty formatting): `.gh-ci-tool/<commit>/ci-status.json`
- CI status report (human): `.gh-ci-tool/<commit>/ci-status.txt`
- LLM concise report (LLM): `.gh-ci-tool/<commit>/ci-status.llm.txt`
- Failed job logs (if any): `.gh-ci-tool/<commit>/logs/<workflow name (sanitized)>-<run id>/<job name (sanitized)>-<job id>.log`
- LLM compact log summaries (when `LLM=true`): `.gh-ci-tool/<commit>/logs/<workflow name (sanitized)>-<run id>/<job name (sanitized)>-<job id>.log.llm.txt`

Path components are sanitized by `sanitize_path_component` (`[A-Za-z0-9_.]` kept, other chars replaced by `-`, duplicate/trailing `-` collapsed/trimmed).

```console
Current branch: autoupdate-librats-0.8.0
Latest commit: 64b9e2ebc1aad19c854e9510d3d2b9bd62db7ed6
Repository: xmake-io/xmake-repo
- Linux ❌
  - build (ubuntu-latest, static, debug): Failure
  - build (ubuntu-latest, shared, release): Cancelled
  - build (ubuntu-latest, static, release): Cancelled
  - build (ubuntu-latest, shared, debug): Cancelled
- Windows (arm64) ✅
- MingW (Msys2) ✅
- Windows ❌
  - build (windows-2025, static, x64, MD): Failure
  - build (windows-2025, static, x64, MT): Failure
  - build (windows-2025, static, arm64, MT): Success
  - build (windows-2025, static, x86, MT): Failure
  - build (windows-2025, shared, x64, MT): Failure
  - build (windows-2025, shared, x86, MT): Failure
  - build (windows-2025, shared, x64, MD): Failure
  - build (windows-2025, static, x86, MD): Failure
  - build (windows-2025, static, arm64, MD): Success
  - build (windows-2025, shared, x86, MD): Failure
  - build (windows-2025, shared, arm64, MT): Success
  - build (windows-2025, shared, arm64, MD): Success
- iPhoneOS ✅
- Cross ❌
  - build (ubuntu-latest, aarch64-none-linux-gnu): Cancelled
  - build (ubuntu-latest, arm-none-linux-gnueabihf): Failure
- Linux (Clang) ❌
  - build (ubuntu-latest, static): Cancelled
  - build (ubuntu-latest, shared): Failure
- Linux (arm64) ❌
  - build (ubuntu-24.04-arm, static, debug): Cancelled
  - build (ubuntu-24.04-arm, shared, release): Failure
  - build (ubuntu-24.04-arm, shared, debug): Cancelled
  - build (ubuntu-24.04-arm, static, release): Cancelled
- Fedora ❌
  - build (ubuntu-latest, shared): Failure
  - build (ubuntu-latest, static): Cancelled
- MingW (MacOS) ✅
- Archlinux ❌
  - build (ubuntu-latest, shared): Failure
  - build (ubuntu-latest, static): Cancelled
- macOS (x86_64) ❌
  - build (macos-15-intel, x86_64, static): Failure
  - build (macos-15-intel, x86_64, shared): Failure
- macOS (arm64) ❌
  - build (macos-14, arm64, shared): Failure
  - build (macos-14, arm64, static): Failure
- Android ✅
- Android (Windows) ✅
- Wasm (Ubuntu) ✅
- FreeBSD ✅
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\Linux-22424544319\build-ubuntu-latest-static-debug-64929684784.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\Windows-22424544326\build-windows-2025-static-x64-MD-64929684843.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\Windows-22424544326\build-windows-2025-static-x64-MT-64929684875.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\Windows-22424544326\build-windows-2025-static-x86-MT-64929684877.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\Windows-22424544326\build-windows-2025-shared-x64-MT-64929684904.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\Windows-22424544326\build-windows-2025-shared-x86-MT-64929684905.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\Windows-22424544326\build-windows-2025-shared-x64-MD-64929684907.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\Windows-22424544326\build-windows-2025-static-x86-MD-64929684912.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\Windows-22424544326\build-windows-2025-shared-x86-MD-64929684914.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\Cross-22424544346\build-ubuntu-latest-arm-none-linux-gnueabihf-64929684705.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\Linux-Clang-22424544318\build-ubuntu-latest-shared-64929684722.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\Linux-arm64-22424544338\build-ubuntu-24.04-arm-shared-release-64929684766.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\Fedora-22424544332\build-ubuntu-latest-shared-64929684707.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\Archlinux-22424544342\build-ubuntu-latest-shared-64929684814.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\macOS-x86_64-22424544328\build-macos-15-intel-x86_64-static-64929684704.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\macOS-x86_64-22424544328\build-macos-15-intel-x86_64-shared-64929684721.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\macOS-arm64-22424544315\build-macos-14-arm64-shared-64929684665.log
Failed job log saved to .gh-ci-tool\64b9e2eb\logs\macOS-arm64-22424544315\build-macos-14-arm64-static-64929684668.log```
```
