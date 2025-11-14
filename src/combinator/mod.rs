//! A few built-in API and implementor combinators.

#[doc(hidden)]
pub mod compose;
#[doc(hidden)]
pub use compose::*;

mod ignore;
pub use ignore::*;

mod err_into;
pub use err_into::*;

mod with_err;
pub use with_err::*;

#[cfg(feature = "tokio")]
mod fork_and_forget;
#[cfg(feature = "tokio")]
pub use fork_and_forget::*;

#[cfg(feature = "tracing")]
pub mod tracing;
//#[cfg(feature = "tracing")]
//pub use tracing::*;
