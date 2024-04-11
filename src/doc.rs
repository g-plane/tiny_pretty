use std::{borrow::Cow, cell::RefCell, rc::Rc};

#[derive(Clone)]
/// The data structure that describes about pretty printing.
///
/// You should avoid using variants on this enum;
/// instead, use helper functions on this enum.
pub enum Doc<'a> {
    #[doc(hidden)]
    Nil,

    #[doc(hidden)]
    /// The first component is for "flat" mode;
    /// the second component is for "break" mode.
    Alt(Rc<Doc<'a>>, Rc<Doc<'a>>),

    #[doc(hidden)]
    /// Try printing the first doc.
    /// If it exceeds the width limitation, print the second doc.
    Union(Rc<Doc<'a>>, Rc<Doc<'a>>),

    #[doc(hidden)]
    Nest(usize, Rc<Doc<'a>>),

    #[doc(hidden)]
    Text(Cow<'a, str>),

    #[doc(hidden)]
    NewLine,

    #[doc(hidden)]
    EmptyLine,

    #[doc(hidden)]
    /// The first component is the number of spaces if it can be put on a single line;
    /// the second component is the number of offset if it will be broken into different lines.
    Break(usize, usize),

    #[doc(hidden)]
    Group(Vec<Rc<Doc<'a>>>),

    #[doc(hidden)]
    List(Vec<Rc<Doc<'a>>>),

    #[doc(hidden)]
    Column(Rc<RefCell<dyn FnMut(usize) -> Doc<'a> + 'a>>),
}

impl<'a> Doc<'a> {
    #[inline]
    /// Insert a piece of text. It **must not** contain line breaks.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::text("code");
    /// assert_eq!("code", &print(doc, &Default::default()));
    ///
    /// let doc = Doc::text(String::from("code"));
    /// assert_eq!("code", &print(doc, &Default::default()));
    /// ```
    pub fn text(s: impl Into<Cow<'a, str>>) -> Doc<'a> {
        Doc::Text(s.into())
    }

    #[inline]
    /// Empty doc, which does nothing.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::nil();
    /// assert!(print(doc, &Default::default()).is_empty());
    /// ```
    pub fn nil() -> Doc<'a> {
        Doc::Nil
    }

    #[inline]
    /// Just a space. This is just short for calling [`text`](Doc::text) with a space.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::space();
    /// assert_eq!(" ", &print(doc, &Default::default()));
    /// ```
    pub fn space() -> Doc<'a> {
        Doc::Text(" ".into())
    }

    #[inline]
    /// Force to print a line break.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc, LineBreak, PrintOptions};
    ///
    /// let doc = Doc::hard_line();
    /// assert_eq!("\n", &print(doc.clone(), &Default::default()));
    /// assert_eq!("\r\n", &print(doc.clone(), &PrintOptions {
    ///     line_break: LineBreak::Crlf,
    ///     ..Default::default()
    /// }));
    ///
    /// // There's a `hard_line` call inside a group,
    /// // so the group always breaks even it doesn't exceed the width limitation.
    /// let doc = Doc::text("fn(")
    ///     .append(Doc::line_or_space())
    ///     .append(Doc::hard_line())
    ///     .group();
    /// assert_eq!("fn(\n\n", &print(doc, &Default::default()));
    /// ```
    pub fn hard_line() -> Doc<'a> {
        Doc::NewLine
    }

    #[inline]
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
    ///         Doc::list(vec![
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
    ///         Doc::list(vec![
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
    pub fn soft_line() -> Doc<'a> {
        Doc::Group(vec![Rc::new(Doc::Break(1, 0))])
    }

    #[inline]
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
    ///         Doc::empty_line().nest(1),
    ///         &Default::default(),
    ///     ),
    /// );
    /// assert_eq!(
    ///     "\n ",
    ///     &print(
    ///         Doc::hard_line().nest(1),
    ///         &Default::default(),
    ///     ),
    /// );
    /// ```
    pub fn empty_line() -> Doc<'a> {
        Doc::EmptyLine
    }

    #[inline]
    /// Create a list of docs.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::list(vec![Doc::text("a"), Doc::text("b"), Doc::text("c")]);
    /// assert_eq!("abc", &print(doc, &Default::default()));
    /// ```
    pub fn list(docs: Vec<Doc<'a>>) -> Doc<'a> {
        Doc::List(docs.into_iter().map(Rc::new).collect())
    }

    #[inline]
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
    ///         Doc::list(vec![
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
    ///         Doc::list(vec![
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
    ///         Doc::list(vec![
    ///             Doc::text("a"),
    ///             Doc::line_or_space(),
    ///             Doc::text("b"),
    ///         ]), // <-- no grouping here
    ///         &options,
    ///     ),
    /// );
    /// ```
    pub fn line_or_space() -> Doc<'a> {
        Doc::Break(1, 0)
    }

    #[inline]
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
    ///         Doc::list(vec![
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
    ///         Doc::list(vec![
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
    ///         Doc::list(vec![
    ///             Doc::text("f("),
    ///             Doc::line_or_nil(),
    ///             Doc::text("arg"),
    ///         ]), // <-- no grouping here
    ///         &options,
    ///     ),
    /// );
    /// ```
    pub fn line_or_nil() -> Doc<'a> {
        Doc::Break(0, 0)
    }

    #[inline]
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
    /// assert_eq!("function(\narg,\n)", &print(doc.clone(), &PrintOptions {
    ///     width: 10,
    ///     ..Default::default()
    /// }));
    ///
    /// assert_eq!("function(arg)", &print(doc.clone(), &PrintOptions {
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
    /// assert_eq!("function(\narg,\n)", &print(doc, &PrintOptions {
    ///     width: 20,
    ///     ..Default::default()
    /// }));
    /// ```
    pub fn flat_or_break(doc_flat: Doc<'a>, doc_break: Doc<'a>) -> Doc<'a> {
        Doc::Alt(Rc::new(doc_flat), Rc::new(doc_break))
    }

    #[inline]
    /// Apply the doc returned by a closure that accepts current width as parameter.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc, PrintOptions};
    ///
    /// let dot = ".";
    /// let doc = Doc::text("column after some text: ")
    ///     .append(Doc::column(|column| Doc::text(column.to_string()).append(Doc::text(dot))));
    ///
    /// assert_eq!("column after some text: 24.", &print(doc, &Default::default()));
    /// ```
    pub fn column<F>(f: F) -> Doc<'a>
    where
        F: FnMut(usize) -> Doc<'a> + 'a,
    {
        Doc::Column(Rc::new(RefCell::new(f)))
    }

    #[inline]
    /// Try applying the current doc. If it exceeds the width limitation, apply the `alternate` doc.
    ///
    /// This looks similar to [`flat_or_break`](Doc::flat_or_break),
    /// but you should use [`flat_or_break`](Doc::flat_or_break) with [`group`](Doc::group) as possible.
    ///
    /// Only consider using this if there're some [`hard_line`](Doc::hard_line) calls in your doc,
    /// since [`hard_line`](Doc::hard_line) will always break in a group.
    /// ```
    /// use tiny_pretty::{print, Doc, PrintOptions};
    ///
    /// let closure = Doc::list(vec![
    ///     Doc::text("|| {"),
    ///     Doc::hard_line()
    ///         .append(
    ///             Doc::text("call2(|| {")
    ///                 .append(Doc::hard_line().append(Doc::text("value")).nest(4))
    ///                 .append(Doc::hard_line())
    ///                 .append(Doc::text("})"))
    ///         )
    ///         .nest(4),
    ///     Doc::hard_line(),
    ///     Doc::text("}"),
    /// ]);
    ///
    /// let doc = Doc::text("fn main() {")
    ///     .append(
    ///         Doc::hard_line()
    ///             .append(
    ///                 Doc::list(vec![
    ///                     Doc::text("call1("),
    ///                     Doc::nil()
    ///                         .append(Doc::text("very_long_arg"))
    ///                         .append(Doc::text(","))
    ///                         .append(Doc::space())
    ///                         .append(closure.clone())
    ///                         .nest(0),
    ///                     Doc::text(")"),
    ///                 ]).union(Doc::list(vec![
    ///                     Doc::text("call1("),
    ///                         Doc::hard_line()
    ///                             .append(Doc::text("very_long_arg"))
    ///                             .append(Doc::text(","))
    ///                             .append(Doc::hard_line())
    ///                             .append(closure)
    ///                             .nest(4),
    ///                         Doc::hard_line(),
    ///                         Doc::text(")"),
    ///                 ])),
    ///             )
    ///             .nest(4)
    ///     )
    ///     .append(Doc::hard_line())
    ///     .append(Doc::text("}"));
    ///
    /// assert_eq!("fn main() {
    ///     call1(
    ///         very_long_arg,
    ///         || {
    ///             call2(|| {
    ///                 value
    ///             })
    ///         }
    ///     )
    /// }", &print(doc.clone(), &PrintOptions {
    ///     width: 10,
    ///     ..Default::default()
    /// }));
    ///
    /// assert_eq!("fn main() {
    ///     call1(very_long_arg, || {
    ///         call2(|| {
    ///             value
    ///         })
    ///     })
    /// }", &print(doc.clone(), &PrintOptions {
    ///     width: 30,
    ///     ..Default::default()
    /// }));
    /// ```
    pub fn union(self, alternate: Doc<'a>) -> Doc<'a> {
        Doc::Union(Rc::new(self), Rc::new(alternate))
    }

    #[inline]
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
    /// assert_eq!("code", &print(doc, &Default::default()));
    /// ```
    pub fn group(self) -> Doc<'a> {
        match self {
            Doc::List(list) => Doc::Group(list),
            Doc::Group(..) => self,
            doc => Doc::Group(vec![Rc::new(doc)]),
        }
    }

    #[inline]
    /// Join two docs.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::text("a").append(Doc::text("b")).append(Doc::text("c"));
    /// assert_eq!("abc", &print(doc, &Default::default()));
    /// ```
    pub fn append(self, other: Doc<'a>) -> Doc<'a> {
        let mut current = if let Doc::List(docs) = self {
            docs
        } else {
            vec![Rc::new(self)]
        };
        match other {
            Doc::List(mut docs) => current.append(&mut docs),
            _ => current.push(Rc::new(other)),
        }
        Doc::List(current)
    }

    #[inline]
    /// Concatenate an iterator whose items are docs.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::text("a").concat(vec![Doc::text("b"), Doc::text("c")].into_iter());
    /// assert_eq!("abc", &print(doc, &Default::default()));
    /// ```
    pub fn concat(self, iter: impl Iterator<Item = Doc<'a>>) -> Doc<'a> {
        let mut current = if let Doc::List(docs) = self {
            docs
        } else {
            vec![Rc::new(self)]
        };
        current.extend(iter.map(Rc::new));
        Doc::List(current)
    }

    #[inline]
    /// Increase indentation level. Usually this method should be called on group
    /// or line break. Calling this on text won't take any effects.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc};
    ///
    /// let doc = Doc::hard_line().nest(2);
    /// assert_eq!("\n  ", &print(doc, &Default::default()));
    ///
    /// let doc = Doc::text("code").nest(2);
    /// assert_eq!("code", &print(doc, &Default::default()));
    /// ```
    pub fn nest(mut self, size: usize) -> Doc<'a> {
        if let Doc::Break(_, offset) = &mut self {
            *offset += size;
            self
        } else {
            Doc::Nest(size, Rc::new(self))
        }
    }
}
