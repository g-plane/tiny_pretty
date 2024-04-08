use crate::{
    options::{LineBreak, PrintOptions},
    Doc, IndentKind,
};

#[derive(Clone, Copy)]
enum Mode {
    Flat,
    Break,
}

type Action<'a> = (usize, Mode, &'a Doc<'a>);

/// Pretty print a doc.
///
/// ## Panics
///
/// Panics if `options.tab_size` is `0`.
pub fn print(doc: &Doc, options: &PrintOptions) -> String {
    assert!(options.tab_size > 0);

    let mut printer = Printer::new(options);
    let mut out = String::with_capacity(1024);
    printer.print_to((0, Mode::Break, doc), &mut out);
    out
}

struct Printer<'a> {
    options: &'a PrintOptions,
    cols: usize,
}

impl<'a> Printer<'a> {
    fn new(options: &'a PrintOptions) -> Self {
        Self { options, cols: 0 }
    }

    fn print_to(&mut self, init_action: Action<'a>, out: &mut String) -> bool {
        let line_break = match self.options.line_break {
            LineBreak::Lf => "\n",
            LineBreak::Crlf => "\r\n",
        };

        let mut actions = Vec::with_capacity(128);
        actions.push(init_action);

        let mut fits = true;

        while let Some((indent, mode, doc)) = actions.pop() {
            match doc {
                Doc::Nil => {}
                Doc::Alt(doc_flat, doc_break) => match mode {
                    Mode::Flat => actions.push((indent, mode, doc_flat)),
                    Mode::Break => actions.push((indent, mode, doc_break)),
                },
                Doc::Union(attempt, alternate) => {
                    let original_cols = self.cols;

                    let mut buf = String::new();
                    if self.print_to((indent, mode, &attempt), &mut buf) {
                        // SAFETY: Both are `String`s.
                        unsafe {
                            out.as_mut_vec().append(buf.as_mut_vec());
                        }
                    } else {
                        self.cols = original_cols;
                        actions.push((indent, mode, alternate));
                    }
                }
                Doc::Nest(offset, doc) => {
                    actions.push((indent + offset, mode, doc));
                }
                Doc::Text(text) => {
                    self.cols += measure_text_width(text);
                    out.push_str(text);
                    fits &= self.cols <= self.options.width;
                }
                Doc::NewLine => {
                    self.cols = indent;
                    out.push_str(line_break);
                    match self.options.indent_kind {
                        IndentKind::Space => {
                            out.push_str(&" ".repeat(indent));
                        }
                        IndentKind::Tab => {
                            out.push_str(&"\t".repeat(indent / self.options.tab_size));
                            out.push_str(&" ".repeat(indent % self.options.tab_size));
                        }
                    }
                    fits &= self.cols <= self.options.width;
                }
                Doc::EmptyLine => {
                    out.push_str(line_break);
                }
                Doc::Break(spaces, offset) => {
                    match mode {
                        Mode::Flat => {
                            self.cols += spaces;
                            out.push_str(&" ".repeat(*spaces));
                        }
                        Mode::Break => {
                            self.cols = indent + offset;
                            out.push_str(line_break);
                            match self.options.indent_kind {
                                IndentKind::Space => {
                                    out.push_str(&" ".repeat(self.cols));
                                }
                                IndentKind::Tab => {
                                    out.push_str(&"\t".repeat(self.cols / self.options.tab_size));
                                    out.push_str(&" ".repeat(self.cols % self.options.tab_size));
                                }
                            }
                        }
                    };
                    fits &= self.cols <= self.options.width;
                }
                Doc::Group(docs) => match mode {
                    Mode::Flat => {
                        actions.extend(docs.iter().map(|doc| (indent, Mode::Flat, doc)).rev());
                    }
                    Mode::Break => {
                        let fitting_actions = docs
                            .iter()
                            .map(|doc| (indent, Mode::Flat, doc))
                            .rev()
                            .collect();
                        let mode = if fitting(
                            fitting_actions,
                            actions.iter().rev(),
                            self.cols,
                            self.options.width,
                        ) {
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

        fits
    }
}

/// Check if a group can be placed on single line.
///
/// There's no magic here:
/// it just simply attempts to put the whole group and the rest actions into current line.
/// After that, if current column is still less than width limitation,
/// we can feel sure that this group can be put on current line without line breaks.
fn fitting<'a>(
    mut actions: Vec<Action<'a>>,
    mut best_actions: impl Iterator<Item = &'a Action<'a>>,
    mut cols: usize,
    width: usize,
) -> bool {
    while let Some((indent, mode, doc)) = actions.pop().or_else(|| best_actions.next().copied()) {
        match doc {
            Doc::Nil => {}
            Doc::Alt(doc_flat, doc_break) => match mode {
                Mode::Flat => actions.push((indent, mode, doc_flat)),
                Mode::Break => actions.push((indent, mode, doc_break)),
            },
            Doc::Union(attempt, alternate) => match mode {
                Mode::Flat => actions.push((indent, mode, attempt)),
                Mode::Break => actions.push((indent, mode, alternate)),
            },
            Doc::Nest(offset, doc) => {
                actions.push((indent + offset, mode, doc));
            }
            Doc::Text(text) => {
                cols += measure_text_width(text);
            }
            Doc::Break(spaces, _) => match mode {
                Mode::Flat => cols += spaces,
                Mode::Break => return true,
            },
            Doc::NewLine => {
                // https://github.com/Marwes/pretty.rs/blob/83021205d557d77731d404cd40b37b105ab762c7/src/render.rs#L381
                return matches!(mode, Mode::Break);
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
    true
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
