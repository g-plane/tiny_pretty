use crate::{
    options::{LineBreak, PrintOptions},
    Doc, IndentKind,
};
use std::fmt::{self, Write};

#[derive(Clone, Copy, Debug)]
enum Mode {
    Flat,
    Break,
}

/// Pretty print a doc.
///
/// ## Panics
///
/// Panics if `options.tab_size` is `0`.
pub fn print(doc: &Doc, options: &PrintOptions) -> Result<String, fmt::Error> {
    assert!(options.tab_size > 0);

    let line_break = match options.line_break {
        LineBreak::Lf => "\n",
        LineBreak::Crlf => "\r\n",
    };

    let mut out = String::with_capacity(1024);
    let mut cols = 0;
    let mut actions = Vec::with_capacity(128);
    actions.push((0, Mode::Break, doc));

    while let Some((indent, mode, doc)) = actions.pop() {
        match doc {
            Doc::Nil => {}
            Doc::Alt(doc_flat, doc_break) => match mode {
                Mode::Flat => actions.push((indent, mode, doc_flat)),
                Mode::Break => actions.push((indent, mode, doc_break)),
            },
            Doc::Nest(offset, doc) => {
                actions.push((indent + offset, mode, doc));
            }
            Doc::Text(text) => {
                cols += measure_text_width(text);
                out.push_str(text);
            }
            Doc::NewLine => {
                cols = indent;
                match options.indent_kind {
                    IndentKind::Space => write!(out, "{line_break}{:indent$}", "")?,
                    IndentKind::Tab => write!(
                        out,
                        "{line_break}{0:\t<tabs$}{0:<spaces$}",
                        "",
                        tabs = indent / options.tab_size,
                        spaces = indent % options.tab_size
                    )?,
                }
            }
            Doc::EmptyLine => {
                out.push_str(line_break);
            }
            Doc::Break(spaces, offset) => match mode {
                Mode::Flat => {
                    cols += spaces;
                    write!(out, "{:spaces$}", "")?;
                }
                Mode::Break => {
                    cols = indent + offset;
                    match options.indent_kind {
                        IndentKind::Space => write!(out, "{line_break}{:cols$}", "")?,
                        IndentKind::Tab => write!(
                            out,
                            "{line_break}{0:\t<tabs$}{0:<spaces$}",
                            "",
                            tabs = cols / options.tab_size,
                            spaces = cols % options.tab_size
                        )?,
                    }
                }
            },
            Doc::Group(docs) => match mode {
                Mode::Flat => {
                    actions.extend(docs.iter().map(|doc| (indent, Mode::Flat, doc)).rev());
                }
                Mode::Break => {
                    let fitting_actions = actions
                        .iter()
                        .cloned()
                        .chain(docs.iter().map(|doc| (indent, Mode::Flat, doc)).rev())
                        .collect();
                    let mode = if fitting(fitting_actions, cols, options.width) {
                        Mode::Flat
                    } else {
                        Mode::Break
                    };
                    actions.extend(docs.iter().map(|doc| (indent, mode, doc)).rev());
                }
            },
            Doc::List(docs) => {
                actions.extend(docs.iter().map(|doc| (indent, mode, doc)).rev());
            }
        }
    }

    Ok(out)
}

/// Check if a group can be placed on single line.
///
/// There's no magic here:
/// it just simply attempts to put the whole group and the rest actions into current line.
/// After that, if current column is still less than width limitation,
/// we can feel sure that this group can be put on current line without line breaks.
fn fitting(mut actions: Vec<(usize, Mode, &Doc)>, mut cols: usize, width: usize) -> bool {
    let mut fit = true;
    while let Some((indent, mode, doc)) = actions.pop() {
        match doc {
            Doc::Nil => {}
            Doc::Alt(doc_flat, doc_break) => match mode {
                Mode::Flat => actions.push((indent, mode, doc_flat)),
                Mode::Break => actions.push((indent, mode, doc_break)),
            },
            Doc::Nest(offset, doc) => {
                actions.push((indent + offset, mode, doc));
            }
            Doc::Text(text) => {
                cols += measure_text_width(text);
            }
            Doc::Break(spaces, _) => match mode {
                Mode::Flat => cols += spaces,
                Mode::Break => {
                    fit = true;
                    cols = indent;
                }
            },
            Doc::NewLine => {
                fit = true;
                cols = indent;
            }
            Doc::EmptyLine => {}
            Doc::Group(docs) | Doc::List(docs) => {
                actions.extend(docs.iter().map(|doc| (indent, mode, doc)).rev());
            }
        }
        if cols > width {
            return false;
        }
    }
    fit
}

#[cfg(not(feature = "unicode-width"))]
fn measure_text_width(text: &str) -> usize {
    text.len()
}

#[cfg(feature = "unicode-width")]
fn measure_text_width(text: &str) -> usize {
    use unicode_width::UnicodeWidthStr;
    text.width()
}
