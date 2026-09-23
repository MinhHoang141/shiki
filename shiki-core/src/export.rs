//! Renders a notebook's notes into a single self-contained HTML or Markdown
//! bundle. Shared by `shiki export` (CLI) and the TUI's own export action
//! (notes-scope, or leader-bound) so there's exactly one rendering
//! implementation, not two that could drift — same reasoning as
//! `git_status_color`/`git_status_suffix` being shared between the footer
//! and the drawer.

use pulldown_cmark::{html, Options, Parser};

use crate::Note;

/// Which of the two bundle shapes to produce — the CLI's own `ExportFormat`
/// (a `clap::ValueEnum`, which this crate deliberately has no dependency on)
/// converts into this at the call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// A single self-contained HTML file with a small embedded stylesheet —
    /// readable on its own with no external assets to ship alongside it.
    Html,
    /// A single plain-Markdown bundle — every note concatenated in order,
    /// each preceded by its title/date/tags as a heading.
    Md,
}

/// Renders every note in `notes` (expected to already be sorted by the
/// caller — both existing callers use `(date, title)`) into one bundle.
pub fn render(notebook: &str, notes: &[Note], format: Format) -> String {
    match format {
        Format::Html => render_html(notebook, notes),
        Format::Md => render_markdown(notebook, notes),
    }
}

/// Renders one note as a standalone document rather than a one-item notebook
/// bundle. Markdown preserves shiki's canonical YAML-frontmatter file shape;
/// HTML reuses the same metadata/body renderer and stylesheet as notebook
/// export, with the note itself as the document title.
pub fn render_note(note: &Note, format: Format) -> crate::Result<String> {
    match format {
        Format::Html => Ok(render_note_html(note)),
        Format::Md => note.to_file_contents(),
    }
}

fn render_markdown(notebook: &str, notes: &[Note]) -> String {
    let mut buf = format!("# {notebook}\n\n");
    for note in notes {
        buf.push_str(&format!("## {}\n\n", note.frontmatter.title));
        buf.push_str(&format!("*{}*", note.frontmatter.date));
        if !note.frontmatter.tags.is_empty() {
            buf.push_str(&format!(
                " \u{2014} tags: {}",
                note.frontmatter.tags.join(", ")
            ));
        }
        buf.push_str("\n\n");
        buf.push_str(&note.body);
        buf.push_str("\n\n---\n\n");
    }
    buf
}

/// Bare `&`/`<`/`>` escaping for the metadata (title/tags) that gets
/// interpolated straight into the HTML shell rather than run through
/// `pulldown-cmark` — the note body itself doesn't need this, since
/// `html::push_html` already escapes it as part of normal Markdown
/// rendering.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn render_article(note: &Note, include_heading: bool) -> String {
    let mut article = String::from("<article>\n");
    if include_heading {
        article.push_str("<h2>");
        article.push_str(&escape_html(&note.frontmatter.title));
        article.push_str("</h2>\n");
    }
    article.push_str("<p class=\"meta\">");
    article.push_str(&note.frontmatter.date.to_string());
    if !note.frontmatter.tags.is_empty() {
        article.push_str(" &mdash; ");
        article.push_str(
            &note
                .frontmatter
                .tags
                .iter()
                .map(|t| format!("<span class=\"tag\">{}</span>", escape_html(t)))
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    article.push_str("</p>\n");
    let mut body_html = String::new();
    html::push_html(&mut body_html, Parser::new_ext(&note.body, Options::all()));
    article.push_str(&body_html);
    article.push_str("</article>\n");
    article
}

fn render_html(notebook: &str, notes: &[Note]) -> String {
    let mut articles = String::new();
    for note in notes {
        articles.push_str(&render_article(note, true));
        articles.push_str("<hr>\n");
    }
    render_html_shell(notebook, &articles)
}

fn render_note_html(note: &Note) -> String {
    let article = render_article(note, false);
    render_html_shell(&note.frontmatter.title, &article)
}

fn render_html_shell(title: &str, articles: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{title}</title>
<style>
  body {{ max-width: 46rem; margin: 2rem auto; padding: 0 1rem;
          font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
          line-height: 1.6; color: #222; }}
  h1 {{ border-bottom: 2px solid #ccc; padding-bottom: 0.3rem; }}
  article h2 {{ margin-bottom: 0.2rem; }}
  .meta {{ color: #777; font-size: 0.85rem; margin-top: 0; }}
  .tag {{ background: #eee; border-radius: 0.3rem; padding: 0.1rem 0.4rem; font-size: 0.8rem; }}
  pre {{ background: #f5f5f5; padding: 0.8rem; overflow-x: auto; border-radius: 0.3rem; }}
  code {{ background: #f5f5f5; padding: 0.1rem 0.3rem; border-radius: 0.2rem; }}
  pre code {{ background: none; padding: 0; }}
  blockquote {{ border-left: 3px solid #ccc; margin-left: 0; padding-left: 1rem; color: #555; }}
  hr {{ border: none; border-top: 1px solid #eee; margin: 2rem 0; }}
  @media (prefers-color-scheme: dark) {{
    body {{ background: #1a1a1a; color: #ddd; }}
    h1 {{ border-color: #444; }}
    .meta {{ color: #999; }}
    .tag {{ background: #333; }}
    pre, code {{ background: #262626; }}
    blockquote {{ border-color: #444; color: #aaa; }}
    hr {{ border-color: #333; }}
  }}
</style>
</head>
<body>
<h1>{title}</h1>
{articles}
</body>
</html>
"#,
        title = escape_html(title),
        articles = articles,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn note(title: &str, tags: &[&str], body: &str) -> Note {
        let mut frontmatter = crate::Frontmatter::new(title, "personal");
        frontmatter.date = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
        frontmatter.tags = tags.iter().map(|t| t.to_string()).collect();
        Note::new(
            std::path::PathBuf::from("note.md"),
            frontmatter,
            body.into(),
        )
    }

    #[test]
    fn markdown_bundle_has_heading_meta_body_and_separator() {
        let out = render(
            "personal",
            &[
                note("First", &["a", "b"], "hello"),
                note("Second", &[], "bye"),
            ],
            Format::Md,
        );
        assert!(out.starts_with("# personal\n\n## First\n\n"));
        assert!(out.contains("*2026-01-02* \u{2014} tags: a, b\n\nhello\n\n---\n\n"));
        // A tag-less note omits the em-dash segment entirely.
        assert!(out.contains("## Second\n\n*2026-01-02*\n\nbye"));
    }

    #[test]
    fn html_bundle_escapes_metadata_and_renders_body_markdown() {
        let out = render(
            "book & club",
            &[note("A <B> & C", &["x<y"], "**bold**")],
            Format::Html,
        );
        // Metadata interpolated directly into the shell is escaped...
        assert!(out.contains("<title>book &amp; club</title>"));
        assert!(out.contains("<h1>book &amp; club</h1>"));
        assert!(out.contains("<h2>A &lt;B&gt; &amp; C</h2>"));
        assert!(out.contains("<span class=\"tag\">x&lt;y</span>"));
        assert!(!out.contains("A <B> & C"));
        // ...while the body goes through pulldown-cmark as real Markdown.
        assert!(out.contains("<strong>bold</strong>"));
    }

    #[test]
    fn standalone_markdown_preserves_frontmatter_and_body() {
        let note = note("Standalone", &["one", "two"], "# Body\n\nhello");
        let out = render_note(&note, Format::Md).unwrap();

        assert!(out.starts_with("---\n"));
        assert!(out.contains("title: Standalone"));
        assert!(out.contains("tags:\n- one\n- two"));
        assert!(out.contains("---\n\n# Body\n\nhello"));
        assert!(!out.contains("# personal\n"));
    }

    #[test]
    fn standalone_html_uses_note_title_as_document_heading() {
        let note = note("A <B> & C", &["x<y"], "**bold**");
        let out = render_note(&note, Format::Html).unwrap();

        assert!(out.contains("<title>A &lt;B&gt; &amp; C</title>"));
        assert!(out.contains("<h1>A &lt;B&gt; &amp; C</h1>"));
        assert!(!out.contains("<h2>A &lt;B&gt; &amp; C</h2>"));
        assert!(out.contains("<span class=\"tag\">x&lt;y</span>"));
        assert!(out.contains("<strong>bold</strong>"));
    }

    #[test]
    fn escape_html_covers_all_three_bare_characters() {
        assert_eq!(escape_html("a & b < c > d"), "a &amp; b &lt; c &gt; d");
        assert_eq!(escape_html("plain"), "plain");
    }
}
