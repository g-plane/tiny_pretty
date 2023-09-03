#[derive(Clone, Debug, Default)]
pub enum LineBreak {
    #[default]
    Lf,
    Crlf,
}

#[derive(Clone, Debug, Default)]
pub enum IndentKind {
    #[default]
    Space,
    Tab,
}

/// Print control options, such as line break and indentation kind.
#[derive(Clone, Debug)]
pub struct PrintOptions {
    /// Line break for each line. It can be "\n" (LF) or "\r\n" (CRLF).
    ///
    /// Default value is LF.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc, LineBreak, PrintOptions};
    ///
    /// let doc = Doc::list(vec![Doc::text("a"), Doc::hardline(), Doc::text("b")]);
    ///
    /// assert_eq!("a\nb", &print(&doc, &PrintOptions {
    ///     line_break: LineBreak::Lf,
    ///     ..Default::default()
    /// }).unwrap());
    ///
    /// assert_eq!("a\r\nb", &print(&doc, &PrintOptions {
    ///     line_break: LineBreak::Crlf,
    ///     ..Default::default()
    /// }).unwrap());
    /// ```
    pub line_break: LineBreak,

    /// To use space or tab for indentation.
    ///
    /// Note that when using tabs and calling [`nest`](crate::Doc::nest) with offset,
    /// it doesn't mean it will print tabs with the number of offset.
    /// Of course, it will print tabs as possible, however if `indent % tab_size != 0`,
    /// it will print tabs first and then fill with spaces to match indentation.
    /// Specifically, it prints `indent / tab_size` times tabs
    /// then prints `indent % tab_size` times spaces.
    /// See the documentation and examples of the [`tab_size`](PrintOptions::tab_size) option below.
    ///
    /// Default value is space.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc, IndentKind, PrintOptions};
    ///
    /// let doc = Doc::list(vec![Doc::text("a"), Doc::hardline().nest(2), Doc::text("b")]);
    ///
    /// assert_eq!("a\n  b", &print(&doc, &PrintOptions {
    ///     indent_kind: IndentKind::Space,
    ///     ..Default::default()
    /// }).unwrap());
    ///
    /// assert_eq!("a\n\tb", &print(&doc, &PrintOptions {
    ///     indent_kind: IndentKind::Tab,
    ///     ..Default::default()
    /// }).unwrap());
    /// ```
    pub indent_kind: IndentKind,

    /// The limitation that pretty printer should *(but not must)* avoid columns exceeding.
    /// Pretty printer will try its best to keep column width less than this value,
    /// but it may exceed for some cases, for example, a very very long single word.
    ///
    /// Default value is 80.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc, PrintOptions};
    ///
    /// let doc = Doc::list(vec![Doc::text("aaaa"), Doc::line_or_space(), Doc::text("bbbb")]).group();
    /// assert_eq!("aaaa\nbbbb", &print(&doc, &PrintOptions {
    ///     width: 5,
    ///     ..Default::default()
    /// }).unwrap());
    ///
    /// assert_eq!("aaaa bbbb", &print(&doc, &PrintOptions {
    ///     width: 20,
    ///     ..Default::default()
    /// }).unwrap());
    ///
    /// let doc = Doc::list(vec![Doc::text("aaaaaaaa"), Doc::line_or_space(), Doc::text("bbbbbbbb")])
    ///     .group();
    /// assert_eq!("aaaaaaaa\nbbbbbbbb", &print(&doc, &PrintOptions {
    ///     width: 5,
    ///     ..Default::default()
    /// }).unwrap());
    /// ```
    pub width: usize,

    /// Tab size is not indentation size.
    /// If `indent_kind` is set to `Tab`, when indentation level satisfies the `tab_size`,
    /// it will convert those spaces to tabs.
    ///
    /// If you're implementing a high-level formatter or pretty printer,
    /// it's highly recommended to set this value as same as indentation size of your
    /// formatter or pretty printer.
    ///
    /// Default value is 2. It can't be zero.
    /// This option will be ignored when `indent_kind` is `Space`.
    ///
    /// ```
    /// use tiny_pretty::{print, Doc, IndentKind, PrintOptions};
    ///
    /// let doc = Doc::list(vec![Doc::text("aaaa"), Doc::hardline(), Doc::text("bbbb")])
    ///     .group()
    ///     .nest(8);
    ///
    /// assert_eq!("aaaa\n\t   bbbb", &print(&doc, &PrintOptions {
    ///     indent_kind: IndentKind::Tab,
    ///     tab_size: 5,
    ///     ..Default::default()
    /// }).unwrap());
    ///
    /// assert_eq!("aaaa\n\t\tbbbb", &print(&doc, &PrintOptions {
    ///     indent_kind: IndentKind::Tab,
    ///     tab_size: 4,
    ///     ..Default::default()
    /// }).unwrap());
    ///
    /// assert_eq!("aaaa\n        bbbb", &print(&doc, &PrintOptions {
    ///     indent_kind: IndentKind::Space,
    ///     tab_size: 5,
    ///     ..Default::default()
    /// }).unwrap());
    /// ```
    pub tab_size: usize,
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self {
            line_break: Default::default(),
            indent_kind: Default::default(),
            width: 80,
            tab_size: 2,
        }
    }
}
