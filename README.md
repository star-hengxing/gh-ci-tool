# Run

- Set the `GITHUB_TOKEN` environment variable.
- Run `cargo build`, then execute the tool inside a git repository whose CI status you want to check.

# Features

1. Fetches CI status from GitHub Actions.
2. Displays the status of all workflows (success/failure).
3. Saves a detailed report to a local file.

Log locations:

- CI status JSON: `.gh-ci-tool/<commit>/ci-status.json`
- CI status report: `.gh-ci-tool/<commit>/ci-status.txt`
- Failed workflow logs (if any): `.gh-ci-tool/<commit>/logs/<workflow name>-<run id>/`

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
