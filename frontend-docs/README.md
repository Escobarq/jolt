# Joltv documentation

Official documentation site for **Joltv**, a fast Java project and dependency
tool built with Rust. The site is an independent Astro project that can be
deployed at a domain root or under the `/Joltv` GitHub Pages project path.

> The current CLI executable is still named `jolt`. Joltv is the product
> identity used by this documentation while the rename is completed.

## Local development

```bash
npm install
npm run dev
```

The development server is available at `http://localhost:4321/`.

To expose the development server through a tunnel such as Localtunnel, start
the server with its network interface enabled:

```powershell
npm run dev -- --host 0.0.0.0
```

By default, development builds accept Localtunnel subdomains such as
`hot-tables-wave.loca.lt` and `better-candles-ask.loca.lt`, so a new tunnel URL
does not require a configuration edit. For a stricter allowlist, replace the
default with exact hostnames:

```powershell
$env:DEV_ALLOWED_HOSTS = "better-candles-ask.loca.lt"
npm run dev -- --host 0.0.0.0
```

Multiple exact hosts can be provided as a comma-separated list. Do not use
`true` or `*`, because that allows any `Host` header.

## Production build

```bash
npm run build
npm run preview
```

The static output is written to `dist/`.

### Deployment base path

The default build targets a site hosted at the domain root (`/`), which is
used by Netlify, Vercel, Cloudflare Pages, and most static hosts. For the
`Escobarq/Joltv` GitHub Pages project site, set the project path explicitly:

```bash
SITE_BASE=/Joltv SITE_URL=https://escobarq.github.io/Joltv npm run build
```

Do not set `SITE_BASE=/Joltv` when the host serves `dist/` directly at its
domain root; that causes links and assets to point to a non-existent
`/Joltv/` directory and produces a 404.

## Documentation map

- `/es/` and `/en/`: localized home pages.
- `docs/quickstart/`: requirements, installation, first project, and next steps.
- `docs/cli/`: commands, flags, workspaces, builds, and diagnostics.
- `docs/configuration/`: `jolt.toml`, dependencies, packaging, and GraalVM.
- `docs/architecture/`: Rust modules and execution flow.
- `docs/toolchains/`: JDK discovery, compatibility, downloads, and Native Image.
- `docs/contributing/`: local checks and contribution workflow.

## Project structure

```text
src/
├── layouts/DocsLayout.astro
├── pages/
│   ├── es/
│   └── en/
└── styles/global.css
```
