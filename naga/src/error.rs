use alloc::{borrow::Cow, boxed::Box, string::String};
use core::{error::Error, fmt};

mod sealed {
    pub trait Sealed {}
}

/// An error returned by [Naga](crate)'s compilation or validation.
///
/// Most standard error handling in Rust centers around the [`Error`](core::error::Error) trait.
/// This model exposes a stack of error layers that can each serialize as human-readable strings
/// with no further context. This model fits many types of error reporting well. However, Naga is
/// a _compiler of human-writable languages_; its compiler diagnostics need more than just error
/// information to render properly.
///
/// Errors implementing this trait can be rendered and all necessary context for rendering it as a diagnostic
pub trait RenderableError: sealed::Sealed {
    /// Emits a summary of the error to standard error stream, possibly colorizing the output with
    /// [SGR control
    /// sequences](https://en.wikipedia.org/wiki/ANSI_escape_code#Select_Graphic_Rendition_parameters)
    /// if it is detected to be rendering to a terminal.
    fn emit_to_stderr(&self, ctx: &ErrorRenderingContext<'_>);

    /// Emits a summary of the error to a string.
    fn emit_to_string(&self, ctx: &ErrorRenderingContext<'_>) -> String;

}

/// All information needed to render a [`RenderableError`].
#[derive(Clone, Debug)]
pub struct ErrorRenderingContext<'a> {
    /// The source code of the shader.
    pub source: &'a str,
    pub label: Option<&'a str>,
    pub path: Option<&'a std::path::Path>,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "termcolor")] {
        type DiagnosticBufferInner = codespan_reporting::term::termcolor::NoColor<alloc::vec::Vec<u8>>;
        pub(crate) use codespan_reporting::term::termcolor::WriteColor as _ErrorWrite;
    } else if #[cfg(feature = "stderr")] {
        type DiagnosticBufferInner = alloc::vec::Vec<u8>;
        pub(crate) use std::io::Write as _ErrorWrite;
    } else {
        type DiagnosticBufferInner = String;
        pub(crate) use core::fmt::Write as _ErrorWrite;
    }
}

// Using this indirect export to avoid duplicating the expect(...) for all three cases above.
#[cfg_attr(
    not(any(feature = "spv-in", feature = "glsl-in")),
    expect(
        unused_imports,
        reason = "only need `ErrorWrite` with an appropriate front-end."
    )
)]
pub(crate) use _ErrorWrite as ErrorWrite;

pub(crate) struct DiagnosticBuffer {
    inner: DiagnosticBufferInner,
}

impl DiagnosticBuffer {
    #[cfg_attr(
        not(feature = "termcolor"),
        expect(
            clippy::missing_const_for_fn,
            reason = "`NoColor::new` isn't `const`, but other `inner`s are."
        )
    )]
    pub fn new() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(feature = "termcolor")] {
                let inner = codespan_reporting::term::termcolor::NoColor::new(alloc::vec::Vec::new());
            } else if #[cfg(feature = "stderr")] {
                let inner = alloc::vec::Vec::new();
            } else {
                let inner = String::new();
            }
        };

        Self { inner }
    }

    pub fn inner_mut(&mut self) -> &mut DiagnosticBufferInner {
        &mut self.inner
    }

    pub fn into_string(self) -> String {
        let Self { inner } = self;

        cfg_if::cfg_if! {
            if #[cfg(feature = "termcolor")] {
                String::from_utf8(inner.into_inner()).unwrap()
            } else if #[cfg(feature = "stderr")] {
                String::from_utf8(inner).unwrap()
            } else {
                inner
            }
        }
    }
}
