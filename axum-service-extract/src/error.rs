use axum::extract::rejection::BodyAlreadyExtracted;

//
pub enum Error<T> {
    Rejection(T),
    BodyAlreadyExtracted(BodyAlreadyExtracted),
}

impl<T> core::fmt::Debug for Error<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Rejection(err) => f.debug_tuple("Error::Rejection").field(err).finish(),
            Self::BodyAlreadyExtracted(err) => f
                .debug_tuple("Error::BodyAlreadyExtracted")
                .field(err)
                .finish(),
        }
    }
}

impl<T> core::fmt::Display for Error<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<T> std::error::Error for Error<T> where T: core::fmt::Debug {}
