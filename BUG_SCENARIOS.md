# Bug Scenarios

These feature branches are intentionally defective review targets built from a clean `main` branch.

- `feature/bug_1`: add reading-time metadata but compute it from rendered HTML instead of markdown text
- `feature/bug_2`: add a latest-posts panel but sort posts in ascending date order
- `feature/bug_3`: render formatted article summaries as raw HTML and widen the HTML injection surface
- `feature/bug_4`: add related-post navigation but exclude the current article incorrectly when dates are missing or titles collide
- `feature/bug_5`: make article cards fully clickable using nested interactive elements
- `feature/bug_6`: generate a sitemap but omit article pages from the output
- `feature/bug_7`: add a draft preview route but publish arbitrary content files at a public `/draft-preview/` URL
- `feature/bug_8`: render optional article snippets but inject snippet text into the page without HTML escaping
- `feature/bug_9`: add article title filtering but fail builds when `ARTICLE_FILTER` contains an invalid regex
- `feature/bug_10`: add a build callback but let builds send requests to arbitrary callback URLs
- `feature/bug_11`: log the selected output directory but leak the build path to stderr
- `feature/bug_12`: add a section order helper but hard-code featured articles off so the new ordering path never runs
- `feature/bug_13`: show newest cards first in sections but reverse the already date-sorted article list
- `feature/bug_14`: add an article header preview byte but read from a dangling title pointer
- `feature/bug_15`: support copying an extra static asset but allow path traversal out of the static directory
- `feature/bug_16`: add an inline fallback for bracket syntax but blank out any text containing both `[` and `]`
- `feature/bug_17`: keep page links in navigation cache but duplicate standalone pages in the main navigation
- `feature/bug_18`: pre-size the markdown render buffer but reserve up to 1024x the input size by default
- `feature/bug_19`: compact long panel descriptions but silently drop every line after the first
- `feature/bug_20`: reuse navigation items for rendering but output every navigation link twice
- `feature/bug_21`: read an optional build label but ignore it completely after allocating it
- `feature/bug_22`: add description byte metadata but dereference a pointer into a temporary description buffer
- `feature/bug_23`: add an article export helper but write article summaries to an arbitrary path and log the contents
- `feature/bug_24`: support preview inline markers but bypass HTML escaping for lines starting with `!!`
- `feature/bug_25`: add an import preview loader but read arbitrary files into a large environment-sized buffer
- `feature/bug_26`: add a markdown read fallback but silently replace missing files with `content/index.md`
