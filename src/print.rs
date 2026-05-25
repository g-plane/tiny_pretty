use crate::{
    IndentKind,
    doc::{Doc, Nest},
    options::{LineBreak, PrintOptions},
};

#[derive(Clone, Copy)]
enum Mode {
    Flat,
    Break,
}

type Action<'a, 'b> = (usize, Mode, &'b Doc<'a>);

/// Pretty print a doc.
///
/// ## Panics
///
/// Panics if `options.tab_size` is `0`.
pub fn print<'a>(doc: &Doc<'a>, options: &'a PrintOptions) -> String {
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

    fn print_to<'b>(&mut self, init_action: Action<'a, 'b>, out: &mut String) -> bool {
        let line_break = match self.options.line_break {
            LineBreak::Lf => "\n",
            LineBreak::Crlf => "\r\n",
        };

        let mut actions = Vec::with_capacity(128);
        actions.push(init_action);

        let mut fitting_actions = Vec::with_capacity(128);
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
                    if self.print_to((indent, mode, attempt), &mut buf) {
                        // SAFETY: Both are `String`s.
                        unsafe {
                            out.as_mut_vec().append(buf.as_mut_vec());
                        }
                    } else {
                        self.cols = original_cols;
                        actions.push((indent, mode, alternate));
                    }
                }
                Doc::Nest(offset, nest) => match nest {
                    Nest::Vec(docs) => {
                        actions.extend(docs.iter().map(|doc| (indent + offset, mode, doc)).rev());
                    }
                    Nest::Slice(docs) => {
                        actions.extend(docs.iter().map(|doc| (indent + offset, mode, doc)).rev());
                    }
                    Nest::Box(doc) => actions.push((indent + offset, mode, &**doc)),
                },
                Doc::Text(text) => {
                    self.cols += measure_text_width(text);
                    out.push_str(text);
                    fits &= self.cols <= self.options.width;
                }
                Doc::NewLine => {
                    self.cols = indent;
                    out.reserve(indent + 2);
                    out.push_str(line_break);
                    match self.options.indent_kind {
                        IndentKind::Space => {
                            for _ in 0..indent {
                                out.push(' ');
                            }
                        }
                        IndentKind::Tab => {
                            for _ in 0..indent / self.options.tab_size {
                                out.push('\t');
                            }
                            for _ in 0..indent % self.options.tab_size {
                                out.push(' ');
                            }
                        }
                    }
                    fits &= self.cols <= self.options.width;
                }
                Doc::EmptyLine => {
                    out.push_str(line_break);
                    self.cols = 0;
                }
                Doc::Break(spaces, offset) => {
                    match mode {
                        Mode::Flat => {
                            self.cols += spaces;
                            out.reserve(*spaces);
                            for _ in 0..*spaces {
                                out.push(' ');
                            }
                        }
                        Mode::Break => {
                            self.cols = indent + offset;
                            out.push_str(line_break);
                            out.reserve(self.cols);
                            match self.options.indent_kind {
                                IndentKind::Space => {
                                    for _ in 0..self.cols {
                                        out.push(' ');
                                    }
                                }
                                IndentKind::Tab => {
                                    for _ in 0..self.cols / self.options.tab_size {
                                        out.push('\t');
                                    }
                                    for _ in 0..self.cols % self.options.tab_size {
                                        out.push(' ');
                                    }
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
                        fitting_actions
                            .extend(docs.iter().map(|doc| (indent, Mode::Flat, doc)).rev());
                        let mode = if fitting(
                            &mut fitting_actions,
                            actions.iter().copied().rev(),
                            self.cols,
                            self.options.width,
                        ) {
                            Mode::Flat
                        } else {
                            Mode::Break
                        };
                        fitting_actions.clear();
                        actions.extend(docs.iter().map(|doc| (indent, mode, doc)).rev());
                    }
                },
                Doc::List(docs) => {
                    actions.extend(docs.iter().map(|doc| (indent, mode, doc)).rev());
                }
                Doc::Slice(docs) => {
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
fn fitting<'a, 'b>(
    fitting_actions: &mut Vec<Action<'a, 'b>>,
    mut best_actions: impl Iterator<Item = Action<'a, 'b>>,
    mut cols: usize,
    width: usize,
) -> bool {
    while let Some((indent, mode, doc)) = fitting_actions.pop().or_else(|| best_actions.next()) {
        match doc {
            Doc::Nil => {}
            Doc::Alt(doc_flat, doc_break) => match mode {
                Mode::Flat => fitting_actions.push((indent, mode, doc_flat)),
                Mode::Break => fitting_actions.push((indent, mode, doc_break)),
            },
            Doc::Union(attempt, alternate) => match mode {
                Mode::Flat => fitting_actions.push((indent, mode, attempt)),
                Mode::Break => fitting_actions.push((indent, mode, alternate)),
            },
            Doc::Nest(offset, nest) => match nest {
                Nest::Vec(docs) => {
                    fitting_actions
                        .extend(docs.iter().map(|doc| (indent + offset, mode, doc)).rev());
                }
                Nest::Slice(docs) => {
                    fitting_actions
                        .extend(docs.iter().map(|doc| (indent + offset, mode, doc)).rev());
                }
                Nest::Box(doc) => fitting_actions.push((indent + offset, mode, doc)),
            },
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
            Doc::EmptyLine => return true,
            Doc::Group(docs) => {
                fitting_actions.extend(docs.iter().map(|doc| (indent, mode, doc)).rev());
            }
            Doc::List(docs) => {
                fitting_actions.extend(docs.iter().map(|doc| (indent, mode, doc)).rev());
            }
            Doc::Slice(docs) => {
                fitting_actions.extend(docs.iter().map(|doc| (indent, mode, doc)).rev());
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
