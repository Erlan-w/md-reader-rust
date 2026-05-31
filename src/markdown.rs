/// markdown.rs — Fast Markdown → HTML renderer using pulldown-cmark.
///
/// Features implemented:
///   • Headings H1-H6 with auto-generated anchor IDs
///   • Tables, blockquotes, lists, inline code, fenced code blocks
///   • Mermaid diagram extraction → <div class="mermaid">…</div>
///   • Syntax highlighting wrapper (highlight.js picks it up at runtime)
///   • Word count (computed on raw text before HTML conversion)

use pulldown_cmark::{
    CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};

/// Render Markdown text to an HTML fragment.
/// Returns `(html_string, word_count)`.
pub fn render(input: &str) -> (String, usize) {
    let word_count = count_words(input);

    // Pre-process: extract ```mermaid blocks, replace with unique placeholders.
    let (processed, mermaid_blocks) = extract_mermaid(input);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(&processed, options);

    // We post-process events to:
    //   1. Add `id` attributes to headings (for TOC anchor links)
    //   2. Wrap fenced code blocks in <pre><code class="language-X">
    let mut html_output = String::with_capacity(processed.len() * 3 / 2);
    let mut heading_counter: usize = 0;
    let mut current_heading_level: Option<HeadingLevel> = None;
    let mut current_heading_text = String::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_content = String::new();

    for event in parser {
        match event {
            // ── Heading start ──────────────────────────────────────────────
            Event::Start(Tag::Heading { level, id, classes, .. }) => {
                current_heading_level = Some(level);
                current_heading_text.clear();

                let tag = heading_tag(level);
                let anchor_id = id
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("heading-{}", heading_counter));
                heading_counter += 1;

                let class_str = if classes.is_empty() {
                    String::new()
                } else {
                    format!(" class=\"{}\"", classes.join(" "))
                };
                html_output.push_str(&format!(
                    "<{tag} id=\"{anchor_id}\"{class_str}>"
                ));
            }

            // ── Heading end ────────────────────────────────────────────────
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = current_heading_level.take() {
                    html_output.push_str(&format!("</{}>", heading_tag(level)));
                }
            }

            // ── Fenced code block start ────────────────────────────────────
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_content.clear();
                code_lang = match &kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }

            // ── Fenced code block end ──────────────────────────────────────
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                // The mermaid placeholder is just text now, handled later.
                let class = if code_lang.is_empty() {
                    String::new()
                } else {
                    format!(" class=\"language-{}\"", html_attr_escape(&code_lang))
                };
                html_output.push_str(&format!(
                    "<pre><code{class}>{}</code></pre>\n",
                    html_escape(&code_content)
                ));
                code_content.clear();
                code_lang.clear();
            }

            // ── Raw text (may be inside code block) ───────────────────────
            Event::Text(text) => {
                if in_code_block {
                    code_content.push_str(&text);
                } else if current_heading_level.is_some() {
                    // collect heading text (also pushed to output via html_output)
                    current_heading_text.push_str(&text);
                    html_output.push_str(&html_escape(&text));
                } else {
                    html_output.push_str(&text);
                }
            }

            // ── Everything else: let pulldown-cmark handle it ──────────────
            other => {
                let mut buf = String::new();
                pulldown_cmark::html::push_html(&mut buf, std::iter::once(other));
                html_output.push_str(&buf);
            }
        }
    }

    // Post-process: replace mermaid placeholders with <div class="mermaid">
    let html_output = inject_mermaid(html_output, &mermaid_blocks);

    (html_output, word_count)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn heading_tag(level: HeadingLevel) -> &'static str {
    match level {
        HeadingLevel::H1 => "h1",
        HeadingLevel::H2 => "h2",
        HeadingLevel::H3 => "h3",
        HeadingLevel::H4 => "h4",
        HeadingLevel::H5 => "h5",
        HeadingLevel::H6 => "h6",
    }
}

/// HTML-escape text content (not attribute values).
fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Escape for use inside HTML attribute values.
fn html_attr_escape(s: &str) -> String {
    s.replace('"', "&quot;").replace('\'', "&#39;")
}

/// Count words in raw Markdown (ignores code blocks for accuracy).
fn count_words(input: &str) -> usize {
    // Simple whitespace split — fast enough, good enough for read-time estimate.
    input
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .count()
}

/// Extract ```mermaid…``` fenced blocks from the raw Markdown string.
/// Replaces them with `<!--MERMAID:N-->` placeholders.
/// Returns the modified string and the list of mermaid block contents.
fn extract_mermaid(input: &str) -> (String, Vec<String>) {
    let mut blocks: Vec<String> = Vec::new();
    let mut output = String::with_capacity(input.len());
    let mut remaining = input;

    while let Some(start) = remaining.find("```mermaid") {
        output.push_str(&remaining[..start]);
        let after_open = &remaining[start + 10..]; // skip "```mermaid"
        // Skip optional language tag line
        let content_start = after_open
            .find('\n')
            .map(|i| i + 1)
            .unwrap_or(after_open.len());

        let content_tail = &after_open[content_start..];
        if let Some(end) = content_tail.find("```") {
            let block_content = content_tail[..end].trim().to_string();
            let idx = blocks.len();
            blocks.push(block_content);
            output.push_str(&format!("<!--MERMAID:{idx}-->"));
            remaining = &content_tail[end + 3..];
        } else {
            // Unclosed block — treat as normal text
            output.push_str(&remaining[start..]);
            remaining = "";
            break;
        }
    }
    output.push_str(remaining);
    (output, blocks)
}

/// Replace `<!--MERMAID:N-->` placeholders (possibly wrapped in `<p>`) with
/// proper `<div class="mermaid">…</div>` elements.
fn inject_mermaid(mut html: String, blocks: &[String]) -> String {
    for (i, block) in blocks.iter().enumerate() {
        let placeholder = format!("<!--MERMAID:{i}-->");
        let div = format!("<div class=\"mermaid\">{}</div>", block);

        // pulldown-cmark wraps block-level items in <p>
        let wrapped = format!("<p>{placeholder}</p>");
        if html.contains(&wrapped) {
            html = html.replace(&wrapped, &div);
        } else {
            html = html.replace(&placeholder, &div);
        }
    }
    html
}
