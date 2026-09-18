# ChatGPT Arnes Desktop

Windows desktop client for the certified ChatGPT Arnes Cloud/Core runtime.

## Architecture

The product boundary is:

```text
Windows 11
  -> Tauri 2 desktop shell
  -> narrow Rust host broker
  -> WSL2 distribution: RHODIZ-Arnes
  -> Docker / Compose
  -> certified ChatGPT-Arnes runtime
  -> Project / Session / signed workspace lease
```

The desktop renderer is not a host shell. It must not receive generic
PowerShell, WSL, Docker, filesystem, process, or backend-secret authority.

## Source strategy

This repository combines selected patterns/components from three MIT-licensed
RHODIZSECURITY forks while keeping ChatGPT-Arnes as the only Cloud/Core
authority:

- OpenHands: coding workspace and editor/terminal/file UX.
- Onyx: Tauri 2 desktop/window/build patterns.
- LibreChat: chat, conversation, MCP/tool and artifact UX.

Exact source pins and adoption status are recorded in
[`docs/PROVENANCE.md`](docs/PROVENANCE.md).

## Current phase

Phase 1 establishes the native shell and a typed WSL2 broker. No upstream UI
component is imported until its source path, commit, license, modifications,
security review, and tests are recorded.

The certified backend architecture and security contract are maintained in
`RHODIZSECURITY/ChatGPT-Arnes`.

## Security defaults

- local packaged renderer only;
- explicit Tauri command allowlist;
- no generic Tauri shell/process plugin;
- backend credentials never persisted by renderer code;
- loopback-only backend exposure by default;
- versioned backend manifest with pinned image digests;
- no `:latest` release images;
- no `docker.sock` inside ChatGPT-Arnes containers;
- Project/Session lease, Landlock, and Resource Governor semantics remain
  authoritative.

## License

MIT. Third-party source remains subject to its original MIT notices; see the
provenance ledger before importing or adapting code.
