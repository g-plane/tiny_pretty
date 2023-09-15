//! Tiny implementation of [Wadler-style](http://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf)
//! pretty printing algorithm.
//!
//! ## Basic Usage
//!
//! Supposed we're going to print a code snippet of function calls,
//! and we already have data structure defined as:
//!
//! ```
//! struct FunctionCall {
//!     name: String,
//!     args: Vec<FunctionCall>,
//! }
//! ```
//!
//! We may have a function call that is very very long,
//! so we need to pretty print it for better readability.
//! Our function call may behave like:
//!
//! ```
//! # struct FunctionCall {
//! #     name: String,
//! #     args: Vec<FunctionCall>,
//! # }
//! let fn_call = FunctionCall {
//!     name: "foo".into(),
//!     args: vec![
//!         FunctionCall { name: "really_long_arg".into(), args: vec![] },
//!         FunctionCall { name: "omg_so_many_parameters".into(), args: vec![] },
//!         FunctionCall { name: "we_should_refactor_this".into(), args: vec![] },
//!         FunctionCall { name: "is_there_seriously_another_one".into(), args: vec![] },
//!     ],
//! };
//! ```
//!
//! (This example is copied from [Prettier](https://github.com/prettier/prettier/blob/411ef345a2b8b424d93aed80e28db862f3341c8f/README.md?plain=1#L69) with modifications.)
//!
//! Now we're going to implement about building [`Doc`] from the data structure above.
//! We expect arguments should be placed on a single line as possible.
//! If they're too long to fit, we insert line break with indentation:
//!
//! - When being on a single line, there're no spaces after left paren and before right paren,
//! and there must be a space after each argument comma.
//! - When being splitted into different lines, there must be indentation when printing arguments,
//! and there must be a line break between arguments.
//!
//! So, we can build [`Doc`] like this:
//!
//! ```
//! # struct FunctionCall {
//! #     name: String,
//! #     args: Vec<FunctionCall>,
//! # }
//! use itertools::Itertools;
//! use tiny_pretty::Doc;
//!
//! fn build_doc(fn_call: &FunctionCall) -> Doc {
//!     Doc::text(&fn_call.name)
//!         .append(Doc::text("("))
//!         .append(
//!             Doc::line_or_nil()
//!                 .append(Doc::list(Itertools::intersperse(
//!                     fn_call.args.iter().map(build_doc),
//!                     Doc::text(",").append(Doc::line_or_space())
//!                 ).collect()))
//!                 .nest(2)
//!                 .append(Doc::line_or_nil())
//!                 .group()
//!         )
//!         .append(Doc::text(")"))
//! }
//! ```
//!
//! Once we have a [`Doc`], we can pretty print it:
//!
//! ```
//! # struct FunctionCall {
//! #     name: String,
//! #     args: Vec<FunctionCall>,
//! # }
//! # use itertools::Itertools;
//! # use tiny_pretty::Doc;
//! # let fn_call = FunctionCall {
//! #     name: "foo".into(),
//! #     args: vec![
//! #         FunctionCall { name: "really_long_arg".into(), args: vec![] },
//! #         FunctionCall { name: "omg_so_many_parameters".into(), args: vec![] },
//! #         FunctionCall { name: "we_should_refactor_this".into(), args: vec![] },
//! #         FunctionCall { name: "is_there_seriously_another_one".into(), args: vec![] },
//! #     ],
//! # };
//! #
//! # fn build_doc(fn_call: &FunctionCall) -> Doc {
//! #     Doc::text(&fn_call.name)
//! #         .append(Doc::text("("))
//! #         .append(
//! #             Doc::line_or_nil()
//! #                 .append(Doc::list(Itertools::intersperse(
//! #                     fn_call.args.iter().map(build_doc),
//! #                     Doc::text(",").append(Doc::line_or_space())
//! #                 ).collect()))
//! #                 .nest(2)
//! #                 .append(Doc::line_or_nil())
//! #                 .group()
//! #         )
//! #         .append(Doc::text(")"))
//! # }
//! use tiny_pretty::{print, PrintOptions};
//!
//! assert_eq!(r#"
//! foo(
//!   really_long_arg(),
//!   omg_so_many_parameters(),
//!   we_should_refactor_this(),
//!   is_there_seriously_another_one()
//! )"#.trim(), &print(&build_doc(&fn_call), &PrintOptions::default()));
//! ```
//!
//! Besides, if we have a function call which is short enough to fit on single line:
//!
//! ```
//! # struct FunctionCall {
//! #     name: String,
//! #     args: Vec<FunctionCall>,
//! # }
//! # use itertools::Itertools;
//! # use tiny_pretty::Doc;
//! use tiny_pretty::{print, PrintOptions};
//!
//! let fn_call = FunctionCall {
//!     name: "foo".into(),
//!     args: vec![
//!         FunctionCall { name: "a".into(), args: vec![] },
//!         FunctionCall { name: "b".into(), args: vec![] },
//!         FunctionCall { name: "c".into(), args: vec![] },
//!         FunctionCall { name: "d".into(), args: vec![] },
//!     ],
//! };
//!
//! # fn build_doc(fn_call: &FunctionCall) -> Doc {
//! #     Doc::text(&fn_call.name)
//! #         .append(Doc::text("("))
//! #         .append(
//! #             Doc::line_or_nil()
//! #                 .append(Doc::list(Itertools::intersperse(
//! #                     fn_call.args.iter().map(build_doc),
//! #                     Doc::text(",").append(Doc::line_or_space())
//! #                 ).collect()))
//! #                 .nest(2)
//! #                 .append(Doc::line_or_nil())
//! #                 .group()
//! #         )
//! #         .append(Doc::text(")"))
//! # }
//! #
//! assert_eq!(
//!     "foo(a(), b(), c(), d())",
//!     &print(&build_doc(&fn_call), &PrintOptions::default()),
//! );
//! ```
//!
//! You can specify advanced printing options, such as controlling line break and
//! indentation kind. See [`PrintOptions`] for details.
//!
//! ## Text Width Measurement
//!
//! By default, text width is measured as "visual width".
//! This strategy makes it satisfy the width limitation visually.
//!
//! But sometimes for some Unicode characters, you may want the column to
//! be close to width limitation as possible, though it will exceed visually.
//! To achieve that, please enable the `unicode-width` feature gate.

mod doc;
mod options;
mod print;

pub use doc::Doc;
pub use options::*;
pub use print::print;
