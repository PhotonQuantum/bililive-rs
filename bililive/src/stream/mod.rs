//! Bilibili live stream.
pub use codec::CodecStream;

pub(crate) mod retry;

mod codec;
#[cfg(test)]
mod tests;
