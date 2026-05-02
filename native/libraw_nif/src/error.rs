use rustler::{Encoder, Env, Term};

/// Maps a libraw integer error code to a human-readable atom/string pair
/// returned to Elixir as `{:error, reason}`.
#[derive(Debug)]
pub enum LibRawError {
    /// libraw returned a non-zero error code
    LibRaw(i32),
    /// Path contains a null byte and cannot be passed to C
    InvalidPath,
    /// libraw returned a null pointer where one was not expected
    NullPointer,
}

impl LibRawError {
    pub fn from_code(code: i32) -> Self {
        LibRawError::LibRaw(code)
    }
}

impl std::fmt::Display for LibRawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LibRawError::LibRaw(code) => write!(f, "libraw error code {}", code),
            LibRawError::InvalidPath => write!(f, "invalid path: contains null byte"),
            LibRawError::NullPointer => write!(f, "libraw returned a null pointer"),
        }
    }
}

/// Helper to turn a libraw integer result into a Rust Result.
pub fn check(code: i32) -> Result<(), LibRawError> {
    if code == 0 {
        Ok(())
    } else {
        Err(LibRawError::from_code(code))
    }
}

/// Encode a `LibRawError` as an Elixir `{:error, reason}` term.
pub fn encode_error<'a>(env: Env<'a>, err: LibRawError) -> Term<'a> {
    let reason = err.to_string().encode(env);
    (rustler::types::atom::error(), reason).encode(env)
}

