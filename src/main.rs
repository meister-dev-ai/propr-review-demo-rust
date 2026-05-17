use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = GeneratorOptions::parse(env::args().skip(1).collect())?;
    let project_root = env::current_dir().map_err(|error| error.to_string())?;
    let generator = SiteGenerator::new(project_root, options.output_directory);
    generator.build()
}

struct GeneratorOptions {
    output_directory: String,
}

impl GeneratorOptions {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut output_directory = String::from("dist");
        let mut index = 0;

        while index < args.len() {
            match args[index].as_str() {
                "--output" => {
                    let Some(value) = args.get(index + 1) else {
                        return Err(String::from("missing value for --output"));
                    };

                    output_directory = value.clone();
                    index += 2;
                }
                flag => return Err(format!("unsupported argument: {flag}")),
            }
        }

        Ok(Self { output_directory })
    }
}

struct SiteGenerator {
    content_directory: PathBuf,
    static_directory: PathBuf,
    output_directory: PathBuf,
}

impl SiteGenerator {
    fn new(project_root: PathBuf, output_directory: String) -> Self {
        Self {
            content_directory: project_root.join("content"),
            static_directory: project_root.join("static"),
            output_directory: project_root.join(output_directory),
        }
    }

    fn build(&self) -> Result<(), String> {
        let site = self.load_site()?;

        if self.output_directory.exists() {
            fs::remove_dir_all(&self.output_directory).map_err(|error| error.to_string())?;
        }

        fs::create_dir_all(&self.output_directory).map_err(|error| error.to_string())?;
        self.copy_static_assets()?;
        self.write_file(
            self.output_directory.join("index.html"),
            self.render_standard_page(&site, &site.home_page),
        )?;

        for page in &site.pages {
            if page.slug != "index" {
                self.write_route_page(&page.path, self.render_standard_page(&site, page))?;
            }
        }

        for section in &site.sections {
            self.write_route_page(&section.path, self.render_section_page(&site, section))?;

            for article in &section.articles {
                self.write_route_page(
                    &article.path,
                    self.render_article_page(&site, section, article),
                )?;
            }
        }

        if let Ok(export_path) = env::var("ARTICLE_EXPORT_PATH") {
            self.write_article_export(&site, &export_path)?;
        }

        Ok(())
    }

    fn load_site(&self) -> Result<SiteModel, String> {
        let mut pages = Vec::new();
        let mut sections = Vec::new();

        for entry in fs::read_dir(&self.content_directory).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if file_type.is_file() && file_name.ends_with(".md") && !file_name.starts_with('_') {
                pages.push(self.build_page(entry.path())?);
            }

            if file_type.is_dir() {
                sections.push(self.build_section(entry.path())?);
            }
        }

        pages.sort_by(navigation_cmp_page);
        sections.sort_by(navigation_cmp_section);

        let mut navigation = pages
            .iter()
            .map(|page| NavigationItem {
                title: page.title.clone(),
                path: page.path.clone(),
                order: page.order,
            })
            .collect::<Vec<_>>();

        navigation.extend(sections.iter().map(|section| NavigationItem {
            title: section.title.clone(),
            path: section.path.clone(),
            order: section.order,
        }));

        navigation.sort_by(navigation_cmp_item);

        let home_page = pages
            .iter()
            .find(|page| page.slug == "index")
            .cloned()
            .ok_or_else(|| String::from("expected content/index.md to exist"))?;

        Ok(SiteModel {
            title: home_page.title.clone(),
            description: String::from(
                "A small convention-based blog for pull request review demonstrations.",
            ),
            navigation,
            home_page,
            pages,
            sections,
        })
    }

    fn build_page(&self, file_path: PathBuf) -> Result<PageModel, String> {
        let slug = slug_from_path(&file_path)?;
        let markdown = parse_markdown_file(&file_path)?;
        let path = if slug == "index" {
            String::from("/")
        } else {
            format!("/{slug}/")
        };

        Ok(PageModel {
            slug: slug.clone(),
            path,
            title: markdown
                .frontmatter
                .title
                .unwrap_or_else(|| title_from_slug(&slug)),
            description: markdown.frontmatter.description.unwrap_or_default(),
            order: markdown.frontmatter.order,
            html: render_markdown(&markdown.body),
        })
    }

    fn build_section(&self, directory_path: PathBuf) -> Result<SectionModel, String> {
        let slug = directory_path
            .file_name()
            .ok_or_else(|| String::from("missing section directory name"))?
            .to_string_lossy()
            .to_string();
        let index_path = directory_path.join("_index.md");

        if !index_path.exists() {
            return Err(format!(
                "expected section index file at {}",
                index_path.display()
            ));
        }

        let markdown = parse_markdown_file(&index_path)?;
        let mut articles = Vec::new();

        for entry in fs::read_dir(&directory_path).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if file_type.is_file() && file_name.ends_with(".md") && file_name != "_index.md" {
                articles.push(self.build_article(&slug, entry.path())?);
            }
        }

        articles.sort_by(article_cmp);

        Ok(SectionModel {
            path: format!("/{slug}/"),
            title: markdown
                .frontmatter
                .title
                .unwrap_or_else(|| title_from_slug(&slug)),
            description: markdown.frontmatter.description.unwrap_or_default(),
            order: markdown.frontmatter.order,
            html: render_markdown(&markdown.body),
            articles,
        })
    }

    fn build_article(
        &self,
        section_slug: &str,
        file_path: PathBuf,
    ) -> Result<ArticleModel, String> {
        let slug = slug_from_path(&file_path)?;
        let markdown = parse_markdown_file(&file_path)?;
        let description = markdown.frontmatter.description.unwrap_or_default();
        let summary = markdown
            .frontmatter
            .summary
            .unwrap_or_else(|| description.clone());

        Ok(ArticleModel {
            path: format!("/{section_slug}/{slug}/"),
            title: markdown
                .frontmatter
                .title
                .unwrap_or_else(|| title_from_slug(&slug)),
            description,
            summary,
            date_display: markdown.frontmatter.date.clone(),
            date_sort_key: markdown.frontmatter.date,
            order: markdown.frontmatter.order,
            html: render_markdown(&markdown.body),
        })
    }

    fn copy_static_assets(&self) -> Result<(), String> {
        fs::copy(
            self.static_directory.join("styles.css"),
            self.output_directory.join("styles.css"),
        )
        .map_err(|error| error.to_string())?;

        Ok(())
    }

    fn write_route_page(&self, route_path: &str, html: String) -> Result<(), String> {
        let relative = if route_path == "/" {
            PathBuf::new()
        } else {
            PathBuf::from(route_path.trim_matches('/'))
        };
        let output_directory = self.output_directory.join(relative);
        fs::create_dir_all(&output_directory).map_err(|error| error.to_string())?;
        self.write_file(output_directory.join("index.html"), html)
    }

    fn write_file(&self, file_path: PathBuf, html: String) -> Result<(), String> {
        fs::write(file_path, html).map_err(|error| error.to_string())
    }

    fn write_article_export(&self, site: &SiteModel, export_path: &str) -> Result<(), String> {
        let export = site
            .sections
            .iter()
            .flat_map(|section| section.articles.iter())
            .map(|article| format!("{}\t{}", article.title, article.summary))
            .collect::<Vec<_>>()
            .join("\n");

        eprintln!("writing export to {export_path}: {export}");
        fs::write(export_path, export).map_err(|error| error.to_string())
    }

    fn render_standard_page(&self, site: &SiteModel, page: &PageModel) -> String {
        self.render_document(
            site,
            &page.title,
            &page.description,
            &page.path,
            format!(
                "<article class=\"panel stack-gap\">{}<div class=\"markdown\">{}</div></article>",
                render_panel_description(&page.description),
                page.html
            ),
        )
    }

    fn render_section_page(&self, site: &SiteModel, section: &SectionModel) -> String {
        let article_cards = section
            .articles
            .iter()
            .map(|article| {
                format!(
                    concat!(
                        "<article class=\"article-card\">",
                        "<div class=\"article-card-meta\">{}</div>",
                        "<h2><a href=\"{}\">{}</a></h2>",
                        "<p>{}</p>",
                        "</article>"
                    ),
                    render_article_meta(article),
                    article.path,
                    html_encode(&article.title),
                    html_encode(&article.summary)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let article_section = if section.articles.is_empty() {
            String::new()
        } else {
            format!(
                concat!(
                    "<section class=\"stack-gap\" aria-labelledby=\"articles-heading\">",
                    "<h2 id=\"articles-heading\">Articles</h2>",
                    "<div class=\"article-list\">{}",
                    "</div></section>"
                ),
                article_cards
            )
        };

        self.render_document(
            site,
            &section.title,
            &section.description,
            &section.path,
            format!(
                "<article class=\"panel stack-gap\">{}<div class=\"markdown\">{}</div>{}</article>",
                render_panel_description(&section.description),
                section.html,
                article_section
            ),
        )
    }

    fn render_article_page(
        &self,
        site: &SiteModel,
        section: &SectionModel,
        article: &ArticleModel,
    ) -> String {
        self.render_document(
            site,
            &article.title,
            &article.description,
            &article.path,
            format!(
                concat!(
                    "<article class=\"panel stack-gap\">",
                    "<a class=\"back-link\" href=\"{}\">Back to {}</a>",
                    "{}",
                    "<div class=\"markdown\">{}</div>",
                    "</article>"
                ),
                section.path,
                html_encode(&section.title),
                render_article_header(article),
                article.html
            ),
        )
    }

    fn render_document(
        &self,
        site: &SiteModel,
        page_title: &str,
        description: &str,
        current_path: &str,
        main_content: String,
    ) -> String {
        let full_title = if current_path == "/" {
            site.title.clone()
        } else {
            format!("{page_title} | {}", site.title)
        };
        let navigation = site
            .navigation
            .iter()
            .map(|item| {
                let class_name = if item.path == current_path {
                    "nav-link nav-link-active"
                } else {
                    "nav-link"
                };
                format!(
                    "<a class=\"{class_name}\" href=\"{}\">{}</a>",
                    item.path,
                    html_encode(&item.title)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            concat!(
                "<!DOCTYPE html>",
                "<html lang=\"en\">",
                "<head>",
                "<meta charset=\"utf-8\">",
                "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">",
                "<title>{}</title>",
                "<meta name=\"description\" content=\"{}\">",
                "<link rel=\"stylesheet\" href=\"/styles.css\">",
                "</head>",
                "<body>",
                "<div class=\"app-shell\">",
                "<header class=\"site-header\">",
                "<div>",
                "<a class=\"site-title\" href=\"/\">{}</a>",
                "<p class=\"site-tagline\">{}</p>",
                "</div>",
                "<nav class=\"site-nav\" aria-label=\"Primary navigation\">{}</nav>",
                "</header>",
                "<main>{}</main>",
                "</div>",
                "</body>",
                "</html>"
            ),
            html_encode(&full_title),
            html_encode(description),
            html_encode(&site.title),
            html_encode(&site.description),
            navigation,
            main_content
        )
    }
}

#[derive(Clone)]
struct ParsedMarkdown {
    frontmatter: Frontmatter,
    body: String,
}

#[derive(Clone, Default)]
struct Frontmatter {
    title: Option<String>,
    description: Option<String>,
    summary: Option<String>,
    date: Option<String>,
    order: Option<i32>,
}

#[derive(Clone)]
struct PageModel {
    slug: String,
    path: String,
    title: String,
    description: String,
    order: Option<i32>,
    html: String,
}

#[derive(Clone)]
struct ArticleModel {
    path: String,
    title: String,
    description: String,
    summary: String,
    date_display: Option<String>,
    date_sort_key: Option<String>,
    order: Option<i32>,
    html: String,
}

#[derive(Clone)]
struct SectionModel {
    path: String,
    title: String,
    description: String,
    order: Option<i32>,
    html: String,
    articles: Vec<ArticleModel>,
}

#[derive(Clone)]
struct NavigationItem {
    title: String,
    path: String,
    order: Option<i32>,
}

#[derive(Clone)]
struct SiteModel {
    title: String,
    description: String,
    navigation: Vec<NavigationItem>,
    home_page: PageModel,
    pages: Vec<PageModel>,
    sections: Vec<SectionModel>,
}

fn parse_markdown_file(file_path: &Path) -> Result<ParsedMarkdown, String> {
    let source = fs::read_to_string(file_path).map_err(|error| error.to_string())?;
    let normalized = source.replace("\r\n", "\n");
    let mut lines = normalized.lines();

    if lines.next() != Some("---") {
        return Ok(ParsedMarkdown {
            frontmatter: Frontmatter::default(),
            body: normalized.trim().to_string(),
        });
    }

    let mut frontmatter = Frontmatter::default();
    let mut body_lines = Vec::new();
    let mut in_frontmatter = true;

    for line in normalized.lines().skip(1) {
        if in_frontmatter {
            if line == "---" {
                in_frontmatter = false;
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                apply_frontmatter(&mut frontmatter, key.trim(), value.trim());
            }
        } else {
            body_lines.push(line);
        }
    }

    Ok(ParsedMarkdown {
        frontmatter,
        body: body_lines.join("\n").trim().to_string(),
    })
}

fn apply_frontmatter(frontmatter: &mut Frontmatter, key: &str, value: &str) {
    let cleaned = value.trim_matches('"').trim_matches('\'').to_string();

    match key {
        "title" => frontmatter.title = Some(cleaned),
        "description" => frontmatter.description = Some(cleaned),
        "summary" => frontmatter.summary = Some(cleaned),
        "date" => frontmatter.date = Some(cleaned),
        "order" => frontmatter.order = cleaned.parse::<i32>().ok(),
        _ => {}
    }
}

fn slug_from_path(file_path: &Path) -> Result<String, String> {
    file_path
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .ok_or_else(|| format!("missing file stem for {}", file_path.display()))
}

fn title_from_slug(slug: &str) -> String {
    if slug == "index" {
        return String::from("Home");
    }

    slug.split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let first = chars.next().unwrap().to_ascii_uppercase();
            format!("{first}{}", chars.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_order(value: Option<i32>) -> i32 {
    value.unwrap_or(i32::MAX)
}

fn navigation_cmp_item(left: &NavigationItem, right: &NavigationItem) -> std::cmp::Ordering {
    normalize_order(left.order)
        .cmp(&normalize_order(right.order))
        .then_with(|| left.title.cmp(&right.title))
}

fn navigation_cmp_page(left: &PageModel, right: &PageModel) -> std::cmp::Ordering {
    normalize_order(left.order)
        .cmp(&normalize_order(right.order))
        .then_with(|| left.title.cmp(&right.title))
}

fn navigation_cmp_section(left: &SectionModel, right: &SectionModel) -> std::cmp::Ordering {
    normalize_order(left.order)
        .cmp(&normalize_order(right.order))
        .then_with(|| left.title.cmp(&right.title))
}

fn article_cmp(left: &ArticleModel, right: &ArticleModel) -> std::cmp::Ordering {
    match (&left.date_sort_key, &right.date_sort_key) {
        (Some(left_date), Some(right_date)) if left_date != right_date => right_date.cmp(left_date),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        _ => normalize_order(left.order)
            .cmp(&normalize_order(right.order))
            .then_with(|| left.title.cmp(&right.title)),
    }
}

fn render_panel_description(description: &str) -> String {
    if description.is_empty() {
        String::new()
    } else {
        format!(
            "<header class=\"panel-header\"><p>{}</p></header>",
            html_encode(description)
        )
    }
}

fn render_article_header(article: &ArticleModel) -> String {
    let meta = render_article_meta(article);

    if meta.is_empty() {
        String::new()
    } else {
        format!("<header class=\"panel-header\"><p>{meta}</p></header>")
    }
}

fn render_article_meta(article: &ArticleModel) -> String {
    match (&article.date_display, article.description.is_empty()) {
        (Some(date), false) => format!(
            "<span>{}</span><span>{}</span>",
            html_encode(date),
            html_encode(&article.description)
        ),
        (Some(date), true) => format!("<span>{}</span>", html_encode(date)),
        (None, false) => html_encode(&article.description),
        (None, true) => String::new(),
    }
}

fn render_markdown(markdown: &str) -> String {
    let mut html = String::new();
    let mut paragraph_lines = Vec::new();
    let mut list_items = Vec::new();

    let flush_paragraph = |html: &mut String, paragraph_lines: &mut Vec<String>| {
        if paragraph_lines.is_empty() {
            return;
        }

        let content = paragraph_lines.join(" ");
        html.push_str(&format!("<p>{}</p>\n", render_inline(&content)));
        paragraph_lines.clear();
    };

    let flush_list = |html: &mut String, list_items: &mut Vec<String>| {
        if list_items.is_empty() {
            return;
        }

        html.push_str("<ul>\n");
        for item in list_items.iter() {
            html.push_str(&format!("  <li>{}</li>\n", render_inline(item)));
        }
        html.push_str("</ul>\n");
        list_items.clear();
    };

    for raw_line in markdown.replace("\r\n", "\n").lines() {
        let line = raw_line.trim();

        if line.is_empty() {
            flush_paragraph(&mut html, &mut paragraph_lines);
            flush_list(&mut html, &mut list_items);
            continue;
        }

        if let Some(content) = line.strip_prefix("# ") {
            flush_paragraph(&mut html, &mut paragraph_lines);
            flush_list(&mut html, &mut list_items);
            html.push_str(&format!("<h1>{}</h1>\n", render_inline(content.trim())));
            continue;
        }

        if let Some(content) = line.strip_prefix("- ") {
            flush_paragraph(&mut html, &mut paragraph_lines);
            list_items.push(content.trim().to_string());
            continue;
        }

        flush_list(&mut html, &mut list_items);
        paragraph_lines.push(line.to_string());
    }

    flush_paragraph(&mut html, &mut paragraph_lines);
    flush_list(&mut html, &mut list_items);

    html.trim().to_string()
}

fn render_inline(text: &str) -> String {
    let mut result = String::new();
    let bytes = text.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'`' {
            if let Some(end) = text[index + 1..].find('`') {
                let start = index + 1;
                let finish = start + end;
                result.push_str("<code>");
                result.push_str(&html_encode(&text[start..finish]));
                result.push_str("</code>");
                index = finish + 1;
                continue;
            }
        }

        if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'*') {
            if let Some(end) = text[index + 2..].find("**") {
                let start = index + 2;
                let finish = start + end;
                result.push_str("<strong>");
                result.push_str(&html_encode(&text[start..finish]));
                result.push_str("</strong>");
                index = finish + 2;
                continue;
            }
        }

        let character = text[index..].chars().next().unwrap();
        result.push_str(&html_encode_char(character));
        index += character.len_utf8();
    }

    result
}

fn html_encode(value: &str) -> String {
    value
        .chars()
        .map(html_encode_char)
        .collect::<Vec<_>>()
        .join("")
}

fn html_encode_char(character: char) -> String {
    match character {
        '&' => String::from("&amp;"),
        '<' => String::from("&lt;"),
        '>' => String::from("&gt;"),
        '"' => String::from("&quot;"),
        '\'' => String::from("&#39;"),
        _ => character.to_string(),
    }
}
