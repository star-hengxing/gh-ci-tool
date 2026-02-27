# Run

- Set the `GITHUB_TOKEN` environment variable.
- Optional: set `LLM=true` for concise CI output and compact log summaries.
- Run `cargo build`, then execute the tool inside a git repository whose CI status you want to check.

# Features

1. Fetches CI status from GitHub Actions.
2. Displays the status of all workflows (success/failure).
3. Saves CI report files to local output.
4. Downloads logs for failed jobs only (instead of all jobs in a failed workflow run).
5. In `LLM=true` mode, writes compact log summaries to reduce tokens.
6. In `LLM=true` mode, terminal output includes a concise CI summary; if failed logs exist, it also prints an LLM compact log index block (no index file is written).

# Project structure

```text
gh-ci-tool/
|-- src/
|   |-- main.rs        # orchestrates CI fetch/report/log workflow
|   |-- args.rs        # CLI arguments
|   |-- models.rs      # CI workflow/job data models + status helpers
|   |-- output.rs      # output mode/profile (llm vs human)
|   |-- report.rs      # human + llm report rendering
|   |-- logs.rs        # job log download + llm log compaction
|   `-- repo.rs        # git branch/commit/repo parsing helpers
|-- Cargo.toml
`-- README.md
```

Log locations:

- CI status JSON (compact, no pretty formatting): `.gh-ci-tool/<commit>/ci-status.json`
- CI status report (human): `.gh-ci-tool/<commit>/ci-status.txt`
- LLM concise report (LLM): `.gh-ci-tool/<commit>/ci-status.llm.txt`
- Failed job logs (if any): `.gh-ci-tool/<commit>/logs/<workflow name>-<run id>/<job name>-<job id>.log`
- LLM compact log summaries (when `LLM=true`): `.gh-ci-tool/<commit>/logs/<workflow name>-<run id>/<job name>-<job id>.log.llm.txt`

```console
Current branch: autoupdate-libjxl-v0.11.2
Latest commit: 7f25db0b89eff104d54124cecf597b13966c1c81
Repository: xmake-io/xmake-repo
Fetching current branch workflows...
⠉ [00:00:01] 2/17 Fetching FreeBSD jobs                                                                                                                                    - Windows (arm64) ✔
- macOS (x86_64) ✔
- Linux (arm64) ✔
- iPhoneOS ✔
- macOS (arm64) ✔
- Archlinux ✔
- Wasm (Ubuntu) ❌
  - build (ubuntu-latest, shared): Cancelled
  - build (ubuntu-latest, static): Failure
- FreeBSD ❌
  - build (ubuntu-latest, shared): Cancelled
  - build (ubuntu-latest, static): Failure
- Android (Windows) ✔
- MingW (Msys2) ✔
- MingW (MacOS) ✔
- Linux ✔
- Linux (Clang) ✔
- Fedora ✔
- Cross ✔
- Windows ✔
- Android ✔
Logs extracted to .gh-ci-tool\7f25db0b\logs\Wasm-(Ubuntu)-22009069016
Logs extracted to .gh-ci-tool\7f25db0b\logs\FreeBSD-22009069021
```
