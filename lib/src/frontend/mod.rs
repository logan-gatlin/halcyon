mod logging;
//mod span;

pub use logging::*;
//pub use span::*;
//
pub type LResult<T> = std::result::Result<T, Log>;
