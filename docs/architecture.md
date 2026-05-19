## Architecture notes

- The site generator treats `content/` as the source of truth for first-class pages and sections.
- New user-visible content should be represented as markdown in `content/` rather than hardcoded in application code.
- This keeps the build, navigation, and review workflow aligned around the same editorial pipeline.
