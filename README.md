# rusty_extract

A Rust port of [Universal Extractor 2](https://github.com/mzelivsky-spec/UniExtract2)'s
file-type detection and extraction-orchestration engine — the part of UniExtract2
that identifies what kind of archive/installer/image a file is and drives the
right helper binary (7-Zip, innoextract, etc.) to unpack it.

## Status
Active — early migration in progress, tracked capability-by-capability against
`capability-manifest.md`. Not yet feature-complete with the source.

This is a **staged** migration. This phase covers the detection + extraction
engine only, as a Rust CLI/library, Windows-only parity target. The AutoIt GUI,
Windows registry context-menu integration, built-in auto-updater, and feedback
system are separate later phases — see `capability-manifest.md` for what's
in vs. out of the current pass.

## Getting started
```bash
cargo build
cargo run -- <file-to-extract>
```

## Architecture
See [ARCHITECTURE.md](./ARCHITECTURE.md) for boundaries, key decisions, and data flow.

## Development
```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## Contributing
See [CONTRIBUTING.md](./CONTRIBUTING.md).

## Security
See [SECURITY.md](./SECURITY.md) to report a vulnerability.

## License
Internal — not for external distribution.

The source project, UniExtract2, is GPLv2-licensed and bundles third-party
helper binaries under their own (sometimes non-commercial) licenses — see
[UniExtract2's license notes](https://github.com/mzelivsky-spec/UniExtract2#license)
before this port acquires any of its own distribution plans.
