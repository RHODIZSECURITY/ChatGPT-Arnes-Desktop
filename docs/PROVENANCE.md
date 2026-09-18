# Source Provenance Ledger

This file is the admission ledger for all copied or adapted third-party source.
A source pin is not permission to import arbitrary code: each imported module
must receive an individual ledger entry before merge.

## Pinned source repositories

| Source | Exact pin | License | Role | Current code import |
| --- | --- | --- | --- | --- |
| `RHODIZSECURITY/openhands` | `a0403035a20c91decadd011b907ee5b489f6788b` | MIT | coding UI reference/components | none |
| `RHODIZSECURITY/onyx-foss` | `e11c874019dbf04032cfc3476d82eba1d069a3d8` | MIT | Tauri 2 desktop reference | none |
| `RHODIZSECURITY/librechat` | `2452f499a86ae215146d988e9418a481616238ad` | MIT | chat/MCP/artifact UI reference/components | none |

## Admission policy

For every code import or adaptation, add an entry with:

- source repository and exact commit;
- original path;
- destination path;
- original copyright/license notice;
- classification: ADOPT or ADAPT;
- modification summary;
- authority/security review;
- test coverage.

No floating branch is a release input.

## Architecture reference entries

### Onyx Tauri desktop structure

- Source: `RHODIZSECURITY/onyx-foss`
- Commit: `e11c874019dbf04032cfc3476d82eba1d069a3d8`
- Paths reviewed:
  - `desktop/package.json`
  - `desktop/src-tauri/Cargo.toml`
  - `desktop/src-tauri/tauri.conf.json`
  - `desktop/src-tauri/tauri.windows.conf.json`
  - `desktop/src-tauri/src/main.rs`
- Classification: REFERENCE ONLY for the initial scaffold.
- Security divergence: ChatGPT Arnes Desktop deliberately does **not** adopt
  Onyx's generic shell plugin or remote-server WebView authority.

### OpenHands coding surface

- Source: `RHODIZSECURITY/openhands`
- Commit: `a0403035a20c91decadd011b907ee5b489f6788b`
- Paths reviewed:
  - `package.json`
  - `electron/`
  - `README.windows.md`
- Classification: REFERENCE ONLY until individual React components are admitted.

### LibreChat conversation surface

- Source: `RHODIZSECURITY/librechat`
- Commit: `2452f499a86ae215146d988e9418a481616238ad`
- Paths reviewed:
  - `client/package.json`
  - `client/src/`
- Classification: REFERENCE ONLY until individual React components are admitted.
