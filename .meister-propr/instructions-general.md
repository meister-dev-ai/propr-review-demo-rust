"""
description: Rust static-site generator review conventions covering content-driven sections, routing, navigation, ordering, and HTML rendering.
when-to-use: When files change in src/main.rs, content/, static/, or generated route/navigation behavior.
"""

# ProPR Review Instructions

## Project Summary

This repository is a small static blog demo used for pull request review workflows.

- `content/` contains markdown source files and frontmatter.
- `src/main.rs` is the native Rust static-site generator.
- `static/styles.css` is copied directly into the generated site output.
- `dist/` is generated output and should be treated as a build artifact.
- `tests/site.spec.ts` contains Playwright coverage for the generated routes.

## Review Priorities

Prioritize correctness, regressions, and maintainability over style nits.

Focus most on:

- content pipeline correctness
- route generation and navigation behavior
- correctness of markdown-to-HTML rendering
- article ordering and section listing behavior
- user-facing rendering issues in generated HTML
- changes that weaken output safety or introduce inconsistent escaping

Avoid low-value comments about minor wording, formatting, or subjective style unless they affect behavior, clarity, or consistency.

## Important Repo Conventions

### Content Structure

- `content/index.md` maps to `/`.
- `content/<name>.md` maps to `/<name>/`.
- `content/<section>/_index.md` defines a top-level section at `/<section>/`.
- additional markdown files in `content/<section>/` become article pages at `/<section>/<article>/`.

Reviewers should flag changes that break these conventions without updating the generator consistently.

### Build Output Rules

- `dist/` is generated output and should not normally be edited by hand.
- if a PR changes `content/`, `static/styles.css`, or `src/main.rs`, verify the generated site still reflects those changes.
- if a PR edits generated HTML in `dist/` directly without a matching source change or explanation, that is likely a problem.

### Sorting and Navigation Invariants

The current generator behavior is intentional:

- pages and sections are ordered by `order`, then `title`
- articles are ordered by `date` descending, then `order`, then `title`
- navigation is derived from generated pages and sections, not maintained separately

Flag PRs that accidentally change these ordering rules or introduce duplicated sources of truth.

### Routing Expectations

The generated site uses folder-based static routes.

- page, section, and article output should exist as `index.html` files under their route directories
- links should consistently use trailing slashes except for `/`
- the build should also copy `static/styles.css` to `dist/styles.css`

Be alert for regressions where route output, navigation highlighting, or internal links stop matching the generated paths.

## Risk Areas Worth Extra Attention

### HTML Rendering

`src/main.rs` renders markdown directly into HTML during the build step.

Reviewers should scrutinize changes that:

- allow raw untrusted HTML through without escaping
- broaden markdown support in ways that break current content or output safety
- bypass the generator's content pipeline with a second rendering path

### Generator Drift

Frontmatter parsing, route generation, and generated page templates must stay aligned.

If a PR changes the content shape, verify that:

- the generator still reads the expected frontmatter fields correctly
- the generated routes and navigation still match repo conventions
- Playwright coverage still exercises the changed behavior when needed

### Date Handling

Dates come from markdown frontmatter strings and drive article ordering.

Flag changes that make article ordering unstable, display invalid dates, or mix formatted dates with raw values inconsistently.

## Good Review Questions

When relevant, ask yourself and analyze:

- Does this change preserve the content-to-route mapping rules?
- If source markdown changed, does the generated HTML still reflect it correctly?
- If the generator changed, do route output and page templates still stay consistent?
- Does this introduce a second source of truth for navigation or routes?
- Does this change broaden the HTML safety surface?
- Does this alter article or navigation order unintentionally?

## Review Tone

Keep comments concrete and actionable. Prefer identifying:

- broken behavior
- routing regressions
- incorrect generated HTML or escaping
- maintainability risks from duplicated logic

Prefer not to comment on purely stylistic or syntactic choices unless they obscure intent or increase the chance of future mistakes.

## Semantic Benchmark Guidance

- New user-facing grouped sections must reuse the existing shared section/article pipeline.
- Flag bespoke parallel content models, rendering paths, or route generation flows that bypass the repository's standard content pipeline.
