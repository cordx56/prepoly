---
title: "Installing the prepoly LSP server"
description: "Build and install the prepoly LSP server."
---

prepoly provides an LSP server implementation.

Run the following command to install `prepoly-lsp`:

```bash
./x cargo install --path crates/prepoly_language_server
```

For projects managed with [ppm](/guides/packages/), configure your editor to
launch `ppm lsp` rather than `prepoly-lsp` directly: it resolves the
dependencies declared in `package.toml` before starting the server, and in a
directory without a `package.toml` it starts the plain server, so the same
configuration works everywhere.
