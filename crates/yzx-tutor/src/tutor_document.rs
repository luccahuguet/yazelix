use crate::cli_render;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

#[derive(Debug, Clone, PartialEq, Eq)]
enum TutorBlock {
    Heading {
        level: u8,
        spans: Vec<TutorSpan>,
    },
    Paragraph(Vec<TutorSpan>),
    List {
        start: Option<u64>,
        items: Vec<Vec<TutorSpan>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TutorSpan {
    Text(String),
    Code(String),
    Emphasis(Vec<TutorSpan>),
    Strong(Vec<TutorSpan>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InlineKind {
    Emphasis,
    Strong,
}

#[derive(Debug, Default)]
struct TutorDocBuilder {
    blocks: Vec<TutorBlock>,
    heading: Option<(u8, Vec<TutorSpan>)>,
    paragraph: Option<Vec<TutorSpan>>,
    list: Option<(Option<u64>, Vec<Vec<TutorSpan>>)>,
    item: Option<Vec<TutorSpan>>,
    inline_stack: Vec<(InlineKind, Vec<TutorSpan>)>,
}

fn parse_tutor_markdown(markdown: &str) -> Result<Vec<TutorBlock>, String> {
    let parser = Parser::new_ext(markdown, Options::empty());
    let mut builder = TutorDocBuilder::default();

    for event in parser {
        match event {
            Event::Start(tag) => builder.start_tag(tag)?,
            Event::End(tag) => builder.end_tag(tag)?,
            Event::Text(text) => builder.push_span(TutorSpan::Text(text.to_string()))?,
            Event::Code(code) => builder.push_span(TutorSpan::Code(code.to_string()))?,
            Event::SoftBreak => builder.push_span(TutorSpan::Text(" ".into()))?,
            Event::HardBreak => builder.push_span(TutorSpan::Text("\n".into()))?,
            Event::Rule => {
                return Err("horizontal rules are not supported in tutor Markdown".into());
            }
            Event::Html(_) | Event::InlineHtml(_) => {
                return Err("HTML is not supported in tutor Markdown".into());
            }
            Event::FootnoteReference(_) => {
                return Err("footnotes are not supported in tutor Markdown".into());
            }
            Event::TaskListMarker(_) => {
                return Err("task lists are not supported in tutor Markdown".into());
            }
            Event::InlineMath(_) | Event::DisplayMath(_) => {
                return Err("math is not supported in tutor Markdown".into());
            }
        }
    }

    if !builder.inline_stack.is_empty() {
        return Err("unclosed inline span in tutor Markdown".into());
    }
    if builder.heading.is_some() || builder.paragraph.is_some() || builder.item.is_some() {
        return Err("unclosed block in tutor Markdown".into());
    }
    if builder.list.is_some() {
        return Err("unclosed list in tutor Markdown".into());
    }

    Ok(builder.blocks)
}

pub(crate) fn render_tutor_markdown(markdown: &str) -> Result<String, String> {
    render_tutor_markdown_with_color(markdown, cli_render::colors_enabled())
}

fn render_tutor_markdown_with_color(markdown: &str, color: bool) -> Result<String, String> {
    let blocks = parse_tutor_markdown(markdown)?;
    Ok(render_tutor_doc(&blocks, color))
}

fn render_tutor_doc(blocks: &[TutorBlock], color: bool) -> String {
    let mut output = String::new();
    let mut previous_was_blank = true;

    for block in blocks {
        if !previous_was_blank {
            output.push('\n');
        }
        match block {
            TutorBlock::Heading { level, spans } => {
                let text = render_spans(spans, color);
                output.push_str(&render_heading(*level, &text, color));
                output.push('\n');
            }
            TutorBlock::Paragraph(spans) => {
                output.push_str(&render_spans(spans, color));
                output.push('\n');
            }
            TutorBlock::List { start, items } => {
                for (index, item) in items.iter().enumerate() {
                    let marker = match start {
                        Some(first) => format!("{}. ", first + index as u64),
                        None => "- ".to_string(),
                    };
                    output.push_str(&render_list_item(&marker, item, color));
                }
            }
        }
        previous_was_blank = false;
    }

    output
}

fn render_list_item(marker: &str, item: &[TutorSpan], color: bool) -> String {
    let rendered = render_spans(item, color);
    let mut lines = rendered.lines();
    let mut output = String::new();

    output.push_str(marker);
    output.push_str(lines.next().unwrap_or_default());
    output.push('\n');

    let continuation_indent = " ".repeat(marker.chars().count());
    for line in lines {
        output.push_str(&continuation_indent);
        output.push_str(line);
        output.push('\n');
    }

    output
}

fn render_heading(level: u8, text: &str, color: bool) -> String {
    if level == 1 {
        cli_render::accent(text, color)
    } else {
        cli_render::section_title(text, color)
    }
}

fn render_spans(spans: &[TutorSpan], color: bool) -> String {
    let mut output = String::new();
    for span in spans {
        match span {
            TutorSpan::Text(text) => output.push_str(text),
            TutorSpan::Code(code) => {
                output.push_str(&render_code_span(code, color));
            }
            TutorSpan::Emphasis(children) => {
                output.push_str(&cli_render::muted(&render_spans(children, color), color));
            }
            TutorSpan::Strong(children) => {
                output.push_str(&cli_render::label(&render_spans(children, color), color));
            }
        }
    }
    output
}

fn render_code_span(code: &str, color: bool) -> String {
    if color {
        cli_render::inline_code(code, true)
    } else {
        format!("`{code}`")
    }
}

impl TutorDocBuilder {
    fn start_tag(&mut self, tag: Tag<'_>) -> Result<(), String> {
        match tag {
            Tag::Paragraph => {
                if self.paragraph.is_some() {
                    return Err("nested paragraphs are not supported in tutor Markdown".into());
                }
                self.paragraph = Some(Vec::new());
            }
            Tag::Heading { level, .. } => {
                if self.heading.is_some() {
                    return Err("nested headings are not supported in tutor Markdown".into());
                }
                self.heading = Some((heading_level(level), Vec::new()));
            }
            Tag::List(start) => {
                if self.list.is_some() {
                    return Err("nested lists are not supported in tutor Markdown".into());
                }
                self.list = Some((start, Vec::new()));
            }
            Tag::Item => {
                if self.list.is_none() {
                    return Err("list item outside list in tutor Markdown".into());
                }
                if self.item.is_some() {
                    return Err("nested list items are not supported in tutor Markdown".into());
                }
                self.item = Some(Vec::new());
            }
            Tag::Emphasis => self.inline_stack.push((InlineKind::Emphasis, Vec::new())),
            Tag::Strong => self.inline_stack.push((InlineKind::Strong, Vec::new())),
            Tag::BlockQuote(_)
            | Tag::CodeBlock(_)
            | Tag::HtmlBlock
            | Tag::Link { .. }
            | Tag::Image { .. }
            | Tag::FootnoteDefinition(_)
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition
            | Tag::Table(_)
            | Tag::TableHead
            | Tag::TableRow
            | Tag::TableCell
            | Tag::Strikethrough
            | Tag::Superscript
            | Tag::Subscript
            | Tag::MetadataBlock(_) => {
                return Err(format!("unsupported tutor Markdown tag: {tag:?}"));
            }
        }
        Ok(())
    }

    fn end_tag(&mut self, tag: TagEnd) -> Result<(), String> {
        match tag {
            TagEnd::Paragraph => {
                let paragraph = self
                    .paragraph
                    .take()
                    .ok_or("paragraph end without paragraph start")?;
                if let Some(item) = &mut self.item {
                    if !item.is_empty() {
                        item.push(TutorSpan::Text(" ".into()));
                    }
                    item.extend(paragraph);
                } else {
                    self.blocks.push(TutorBlock::Paragraph(paragraph));
                }
            }
            TagEnd::Heading(_) => {
                let (level, spans) = self
                    .heading
                    .take()
                    .ok_or("heading end without heading start")?;
                self.blocks.push(TutorBlock::Heading { level, spans });
            }
            TagEnd::List(_) => {
                let (start, items) = self.list.take().ok_or("list end without list start")?;
                self.blocks.push(TutorBlock::List { start, items });
            }
            TagEnd::Item => {
                let item = self.item.take().ok_or("item end without item start")?;
                let (_, items) = self.list.as_mut().ok_or("item end outside list")?;
                items.push(item);
            }
            TagEnd::Emphasis => self.pop_inline(InlineKind::Emphasis)?,
            TagEnd::Strong => self.pop_inline(InlineKind::Strong)?,
            TagEnd::BlockQuote(_)
            | TagEnd::CodeBlock
            | TagEnd::HtmlBlock
            | TagEnd::Link
            | TagEnd::Image
            | TagEnd::FootnoteDefinition
            | TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition
            | TagEnd::Table
            | TagEnd::TableHead
            | TagEnd::TableRow
            | TagEnd::TableCell
            | TagEnd::Strikethrough
            | TagEnd::Superscript
            | TagEnd::Subscript
            | TagEnd::MetadataBlock(_) => {
                return Err(format!("unsupported tutor Markdown end tag: {tag:?}"));
            }
        }
        Ok(())
    }

    fn push_span(&mut self, span: TutorSpan) -> Result<(), String> {
        if let Some((_, spans)) = self.inline_stack.last_mut() {
            spans.push(span);
        } else if let Some((_, spans)) = &mut self.heading {
            spans.push(span);
        } else if let Some(spans) = &mut self.paragraph {
            spans.push(span);
        } else if let Some(item) = &mut self.item {
            item.push(span);
        } else {
            return Err("inline content outside supported tutor Markdown block".into());
        }
        Ok(())
    }

    fn pop_inline(&mut self, kind: InlineKind) -> Result<(), String> {
        let (frame_kind, spans) = self
            .inline_stack
            .pop()
            .ok_or("inline end without inline start")?;
        if frame_kind != kind {
            return Err("mismatched inline span in tutor Markdown".into());
        }
        let span = match kind {
            InlineKind::Emphasis => TutorSpan::Emphasis(spans),
            InlineKind::Strong => TutorSpan::Strong(spans),
        };
        self.push_span(span)?;
        Ok(())
    }
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsupported_markdown_constructs() {
        assert!(parse_tutor_markdown("<span>html</span>").is_err());
        assert!(parse_tutor_markdown("> quoted\n").is_err());
        assert!(parse_tutor_markdown("[link](https://example.com)").is_err());
        assert!(parse_tutor_markdown("```sh\nyzx tutor\n```\n").is_err());
    }

    #[test]
    fn renders_inline_code_with_plain_markers() {
        let output = render_tutor_markdown_with_color(
            "Run `yzx tutor begin` then press `Alt Shift M`.",
            false,
        )
        .unwrap();
        assert_eq!(output, "Run `yzx tutor begin` then press `Alt Shift M`.\n");
    }

    #[test]
    fn renders_inline_code_with_ansi_when_color_is_enabled() {
        let output = render_tutor_markdown_with_color("Run `yzx tutor begin`.", true).unwrap();
        assert!(output.contains("yzx tutor begin"));
        assert!(!output.contains("`yzx tutor begin`"));
        assert!(output.contains("\u{1b}["));
    }

    #[test]
    fn renders_headings_paragraphs_and_ordered_lists() {
        let output = render_tutor_markdown_with_color(
            "# Yazelix tutor\n\nStart here.\n\n1. Run `yzx enter`\n2. Run `yzx keys`\n",
            false,
        )
        .unwrap();
        assert_eq!(
            output,
            "Yazelix tutor\n\nStart here.\n\n1. Run `yzx enter`\n2. Run `yzx keys`\n"
        );
    }

    #[test]
    fn indents_list_item_hard_break_continuations() {
        let output = render_tutor_markdown_with_color(
            "1. `yzx tutor workspace` Workspace roots  \n   Practice the current tab root.\n",
            false,
        )
        .unwrap();
        assert_eq!(
            output,
            "1. `yzx tutor workspace` Workspace roots\n   Practice the current tab root.\n"
        );
    }
}
