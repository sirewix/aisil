//! A few built-in API and implementor combinators.

#[doc(hidden)]
pub mod compose;
#[doc(hidden)]
pub use compose::*;

pub mod ignore;
pub use ignore::*;

pub mod err_into;
pub use err_into::*;

pub mod with_err;
pub use with_err::*;

#[cfg(feature = "tokio")]
pub mod fork_and_forget;
#[cfg(feature = "tokio")]
pub use fork_and_forget::*;

#[cfg(feature = "tracing")]
pub mod tracing;
#[cfg(feature = "tracing")]
pub use tracing::*;
