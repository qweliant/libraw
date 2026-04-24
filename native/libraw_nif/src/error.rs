use rustler::{Encoder, Env, NifResult, Term};

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
    /// The processed image has an unexpected type (not LIBRAW_IMAGE_BITMAP)
    UnexpectedImageType,
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
            LibRawError::UnexpectedImageType => {
                write!(f, "libraw returned an unexpected image type (not bitmap)")
            }
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

/// Convenience: encode a `Result<Term, LibRawError>` into an Elixir term.
pub fn result_to_term<'a>(env: Env<'a>, result: Result<Term<'a>, LibRawError>) -> Term<'a> {
    match result {
        Ok(t) => t,
        Err(e) => encode_error(env, e),
    }
}

/// Allow `?` to propagate `LibRawError` inside functions that return `NifResult`.
impl From<LibRawError> for rustler::Error {
    fn from(e: LibRawError) -> rustler::Error {
        rustler::Error::Term(Box::new(e.to_string()))
    }
}
