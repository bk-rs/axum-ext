use core::fmt;

use axum::extract::rejection::BodyAlreadyExtracted;

//
pub enum Error<T> {
    Rejection(T),
    BodyAlreadyExtracted(BodyAlreadyExtracted),
}

impl<T> fmt::Debug for Error<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rejection(err) => f.debug_tuple("Error::Rejection").field(err).finish(),
            Self::BodyAlreadyExtracted(err) => f
                .debug_tuple("Error::BodyAlreadyExtracted")
                .field(err)
                .finish(),
        }
    }
}

impl<T> fmt::Display for Error<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<T> std::error::Error for Error<T> where T: fmt::Debug {}
