# Propr Review Demo Rust

Native Rust static blog demo for pull request review workflows.

## Structure

- `content/` contains markdown content and frontmatter.
- `src/main.rs` is the static-site generator.
- `static/styles.css` is copied into the generated site.
- `dist/` is generated output.
- `tests/site.spec.ts` covers the generated routes with Playwright.

## Content conventions

- `content/index.md` becomes `/`.
- `content/<name>.md` becomes `/<name>/`.
- `content/<section>/_index.md` becomes `/<section>/`.
- Additional markdown files in `content/<section>/` become article pages under that section.
- User-facing sections should flow through the shared section/article pipeline so rendering, ordering, and navigation stay consistent.

## Commands

- `cargo run -- --output dist`
- `cargo test`
- `npm install`
- `npm run test:e2e`

## Review branches

- `BUG_SCENARIOS.md` lists the intentionally defective feature branches that should be reviewed against `main`.
