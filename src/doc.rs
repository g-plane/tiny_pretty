use std::{borrow::Cow, rc::Rc};

/// The data structure that describes about pretty printing.
///
/// You should avoid using variants on this enum;
/// instead, use helper functions on this enum.
#[derive(Clone, Debug)]
pub enum Doc<'a> {
    #[doc(hidden)]
    Nil,

    /// The first component is for "flat" mode;
    /// the second component is for "break" mode.
    #[doc(hidden)]
    Alt(Rc<Doc<'a>>, Rc<Doc<'a>>),

    #[doc(hidden)]
    Nest(usize, Rc<Doc<'a>>),

    #[doc(hidden)]
    Text(Cow<'a, str>),

    #[doc(hidden)]
    NewLine,

    #[doc(hidden)]
    EmptyLine,

    /// The first component is the number of spaces if it can be put on a single line;
    /// the second component is the number of offset if it will be broken into different lines.
    #[doc(hidden)]
    Break(usize, usize),

    #[doc(hidden)]
    Group(Vec<Doc<'a>>),

    #[doc(hidden)]
    List(Vec<Doc<'a>>),
}

impl<'a> Doc<'a> {
    /// Insert a piece of text. It **must not** contain line breaks.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::text("code");
    /// assert_eq!("code", &print(&doc, &Default::default()));
    ///
    /// let doc = Doc::text(String::from("code"));
    /// assert_eq!("code", &print(&doc, &Default::default()));
    /// ```
    #[inline]
    pub fn text(s: impl Into<Cow<'a, str>>) -> Doc<'a> {
        Doc::Text(s.into())
    }

    /// Empty doc, which does nothing.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::nil();
    /// assert!(print(&doc, &Default::default()).is_empty());
    /// ```
    #[inline]
    pub fn nil() -> Doc<'a> {
        Doc::Nil
    }

    /// Just a space. This is just short for calling [`text`](Doc::text) with a space.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::space();
    /// assert_eq!(" ", &print(&doc, &Default::default()));
    /// ```
    #[inline]
    pub fn space() -> Doc<'a> {
        Doc::Text(" ".into())
    }

    /// Force to print a line break.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc, LineBreak, PrintOptions};
    ///
    /// let doc = Doc::hard_line();
    /// assert_eq!("\n", &print(&doc, &Default::default()));
    /// assert_eq!("\r\n", &print(&doc, &PrintOptions {
    ///     line_break: LineBreak::Crlf,
    ///     ..Default::default()
    /// }));
    /// ```
    #[inline]
    pub fn hard_line() -> Doc<'a> {
        Doc::NewLine
    }

    /// "Soft line" allows you to put docs on a single line as many as possible.
    /// Once it's going to exceed the width limitation, it will insert a line break,
    /// but before that it will insert spaces instead of line break.
    ///
    /// This is different from [`line_or_space`](Doc::line_or_space). See the examples below.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc, PrintOptions};
    ///
    /// let options = PrintOptions { width: 10, ..Default::default() };
    /// assert_eq!(
    ///     "aaaa bbbb\ncccc",
    ///     &print(
    ///         &Doc::list(vec![
    ///             Doc::text("aaaa"),
    ///             Doc::soft_line(),
    ///             Doc::text("bbbb"),
    ///             Doc::soft_line(),
    ///             Doc::text("cccc"),
    ///         ]).group(),
    ///         &options,
    ///     ),
    /// );
    /// assert_eq!(
    ///     "aaaa\nbbbb\ncccc",
    ///     &print(
    ///         &Doc::list(vec![
    ///             Doc::text("aaaa"),
    ///             Doc::line_or_space(),
    ///             Doc::text("bbbb"),
    ///             Doc::line_or_space(),
    ///             Doc::text("cccc"),
    ///         ]).group(),
    ///         &options,
    ///     ),
    /// );
    /// ```
    #[inline]
    pub fn soft_line() -> Doc<'a> {
        Doc::Group(vec![Doc::Break(1, 0)])
    }

    /// "Empty line" is simliar to [`hard_line`](Doc::hard_line) but it won't be
    /// affected by indentation. That is, it always prints an empty line without
    /// spaces or tabs indented.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc, PrintOptions};
    ///
    /// assert_eq!(
    ///     "\n",
    ///     &print(
    ///         &Doc::empty_line().nest(1),
    ///         &Default::default(),
    ///     ),
    /// );
    /// assert_eq!(
    ///     "\n ",
    ///     &print(
    ///         &Doc::hard_line().nest(1),
    ///         &Default::default(),
    ///     ),
    /// );
    /// ```
    #[inline]
    pub fn empty_line() -> Doc<'a> {
        Doc::EmptyLine
    }

    /// Create a list of docs.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::list(vec![Doc::text("a"), Doc::text("b"), Doc::text("c")]);
    /// assert_eq!("abc", &print(&doc, &Default::default()));
    /// ```
    #[inline]
    pub fn list(l: Vec<Doc<'a>>) -> Doc<'a> {
        Doc::List(l)
    }

    /// Print a space if doc can be put on a single line, otherwise print a line break.
    ///
    /// *This won't take any effects if used outside a group: it will just print a line break.*
    ///
    /// ```
    /// use tiny_pretty::{print, Doc, PrintOptions};
    ///
    /// let options = PrintOptions { width: 10, ..Default::default() };
    /// assert_eq!(
    ///     "aaaa\nbbbb\ncccc",
    ///     &print(
    ///         &Doc::list(vec![
    ///             Doc::text("aaaa"),
    ///             Doc::line_or_space(),
    ///             Doc::text("bbbb"),
    ///             Doc::line_or_space(),
    ///             Doc::text("cccc"),
    ///         ]).group(),
    ///         &options,
    ///     ),
    /// );
    /// assert_eq!(
    ///     "a b",
    ///     &print(
    ///         &Doc::list(vec![
    ///             Doc::text("a"),
    ///             Doc::line_or_space(),
    ///             Doc::text("b"),
    ///         ]).group(),
    ///         &options,
    ///     ),
    /// );
    ///
    /// assert_eq!(
    ///     "a\nb",
    ///     &print(
    ///         &Doc::list(vec![
    ///             Doc::text("a"),
    ///             Doc::line_or_space(),
    ///             Doc::text("b"),
    ///         ]), // <-- no grouping here
    ///         &options,
    ///     ),
    /// );
    /// ```
    #[inline]
    pub fn line_or_space() -> Doc<'a> {
        Doc::Break(1, 0)
    }

    /// Print nothing if doc can be put on a single line, otherwise print a line break.
    ///
    /// *This won't take any effects if used outside a group: it will just print a line break.*
    ///
    /// ```
    /// use tiny_pretty::{print, Doc, PrintOptions};
    ///
    /// let options = PrintOptions { width: 5, ..Default::default() };
    /// assert_eq!(
    ///     "func(\narg",
    ///     &print(
    ///         &Doc::list(vec![
    ///             Doc::text("func("),
    ///             Doc::line_or_nil(),
    ///             Doc::text("arg"),
    ///         ]).group(),
    ///         &options,
    ///     ),
    /// );
    /// assert_eq!(
    ///     "f(arg",
    ///     &print(
    ///         &Doc::list(vec![
    ///             Doc::text("f("),
    ///             Doc::line_or_nil(),
    ///             Doc::text("arg"),
    ///         ]).group(),
    ///         &options,
    ///     ),
    /// );
    ///
    /// assert_eq!(
    ///     "f(\narg",
    ///     &print(
    ///         &Doc::list(vec![
    ///             Doc::text("f("),
    ///             Doc::line_or_nil(),
    ///             Doc::text("arg"),
    ///         ]), // <-- no grouping here
    ///         &options,
    ///     ),
    /// );
    /// ```
    #[inline]
    pub fn line_or_nil() -> Doc<'a> {
        Doc::Break(0, 0)
    }

    /// Apply `doc_flat` when it can be put on a single line,
    /// otherwise apply `doc_break`.
    ///
    /// *This won't take any effects if used outside a group: it will just apply `doc_break`.*
    ///
    /// ```
    /// use tiny_pretty::{print, Doc, PrintOptions};
    ///
    /// let doc = Doc::list(vec![
    ///     Doc::text("function("),
    ///     Doc::line_or_nil(),
    ///     Doc::text("arg"),
    ///     Doc::flat_or_break(Doc::nil(), Doc::text(",")),
    ///     Doc::line_or_nil(),
    ///     Doc::text(")"),
    /// ]).group();
    ///
    /// assert_eq!("function(\narg,\n)", &print(&doc, &PrintOptions {
    ///     width: 10,
    ///     ..Default::default()
    /// }));
    ///
    /// assert_eq!("function(arg)", &print(&doc, &PrintOptions {
    ///     width: 20,
    ///     ..Default::default()
    /// }));
    ///
    ///
    ///
    /// let doc = Doc::list(vec![
    ///     Doc::text("function("),
    ///     Doc::line_or_nil(),
    ///     Doc::text("arg"),
    ///     Doc::flat_or_break(Doc::nil(), Doc::text(",")),
    ///     Doc::line_or_nil(),
    ///     Doc::text(")"),
    /// ]); // <-- no grouping here
    ///
    /// assert_eq!("function(\narg,\n)", &print(&doc, &PrintOptions {
    ///     width: 20,
    ///     ..Default::default()
    /// }));
    /// ```
    #[inline]
    pub fn flat_or_break(doc_flat: Doc<'a>, doc_break: Doc<'a>) -> Doc<'a> {
        Doc::Alt(Rc::new(doc_flat), Rc::new(doc_break))
    }

    /// Mark the docs as a group.
    ///
    /// For a group of docs, when printing,
    /// they will be checked if those docs can be put on a single line.
    /// If they can't, it may insert line breaks according to the
    /// [`line_or_space`](Doc::line_or_space), [`line_or_nil`](Doc::line_or_nil)
    /// or [`soft_line`](Doc::soft_line) calls in the group.
    /// (Also, please read examples of those functions for usage of `group`.)
    ///
    /// Calling this on text won’t take any effects.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::text("code").group();
    /// assert_eq!("code", &print(&doc, &Default::default()));
    /// ```
    #[inline]
    pub fn group(self) -> Doc<'a> {
        match self {
            Doc::List(list) => Doc::Group(list),
            Doc::Group(..) => self,
            doc => Doc::Group(vec![doc]),
        }
    }

    /// Join two docs.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::text("a").append(Doc::text("b")).append(Doc::text("c"));
    /// assert_eq!("abc", &print(&doc, &Default::default()));
    /// ```
    #[inline]
    pub fn append(self, other: Doc<'a>) -> Doc<'a> {
        let mut current = if let Doc::List(docs) = self {
            docs
        } else {
            vec![self]
        };
        match other {
            Doc::List(mut docs) => current.append(&mut docs),
            _ => current.push(other),
        }
        Doc::List(current)
    }

    /// Concatenate an iterator whose items are docs.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::text("a").concat(vec![Doc::text("b"), Doc::text("c")].into_iter());
    /// assert_eq!("abc", &print(&doc, &Default::default()));
    /// ```
    #[inline]
    pub fn concat(self, iter: impl Iterator<Item = Doc<'a>>) -> Doc<'a> {
        let mut current = if let Doc::List(docs) = self {
            docs
        } else {
            vec![self]
        };
        current.extend(iter);
        Doc::List(current)
    }

    /// Increase indentation level. Usually this method should be called on group
    /// or line break. Calling this on text won't take any effects.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::hard_line().nest(2);
    /// assert_eq!("\n  ", &print(&doc, &Default::default()));
    ///
    /// let doc = Doc::text("code").nest(2);
    /// assert_eq!("code", &print(&doc, &Default::default()));
    /// ```
    #[inline]
    pub fn nest(mut self, size: usize) -> Doc<'a> {
        match &mut self {
            Doc::Break(_, offset) => {
                *offset += size;
                self
            }
            Doc::Group(docs) => {
                if let [Doc::Break(_, offset)] = &mut docs[..] {
                    *offset += size;
                }
                self
            }
            _ => Doc::Nest(size, Rc::new(self)),
        }
    }
}
