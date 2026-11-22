# Joltv documentation

Static HTML + CSS documentation site for **Joltv**, a fast Java project and dependency tool built with Rust.

> The current CLI executable is still named `jolt`. Joltv is the product identity used by this documentation while the rename is completed.

## Structure

```text
frontend-docs/
├── index.html                  # Spanish home
├── en/index.html               # English home
├── en/docs/*/index.html        # English docs (quickstart, cli, configuration, architecture, toolchains, contributing)
├── es/docs/*/index.html        # Spanish docs (quickstart, cli, configuration, architecture, toolchains, contributing)
├── styles/global.css           # Shared stylesheet
├── public/favicon.ico
├── public/favicon.svg
└── README.md
```

## Pages

- `/es/` and `/en/`: localized home pages.
- `docs/quickstart/`: requirements, installation, first project, and next steps.
- `docs/cli/`: commands, flags, workspaces, builds, and diagnostics.
- `docs/configuration/`: `jolt.toml`, dependencies, and packaging.
- `docs/architecture/`: Rust modules and execution flow.
- `docs/toolchains/`: JDK discovery, compatibility, and downloads.
- `docs/contributing/`: local checks and contribution workflow.

No build step is required: open any `.html` file directly in a browser or serve the folder with any static file server.
