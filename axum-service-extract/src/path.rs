use axum::{
    extract::{rejection::PathRejection, FromRequest as _, Path, RequestParts},
    http::{
        request::Parts as HttpRequestParts, Extensions as HttpExtensions, Request as HttpRequest,
    },
};
use serde::de::DeserializeOwned;

use crate::error::Error;

//
pub async fn path_from_request<T, B>(
    req: HttpRequest<B>,
) -> Result<(Option<Path<T>>, HttpRequest<B>), Error<PathRejection>>
where
    T: DeserializeOwned + Send,
{
    //
    let (mut parts, extensions, body) = {
        let (
            HttpRequestParts {
                method,
                uri,
                version,
                headers,
                extensions,
                ..
            },
            body,
        ) = req.into_parts();
        let (mut parts, _) = HttpRequest::new(()).into_parts();
        parts.method = method;
        parts.uri = uri;
        parts.version = version;
        parts.headers = headers;

        (parts, extensions, body)
    };

    //
    let (path, extensions) = path_from_extensions(extensions).await?;

    //
    parts.extensions = extensions;
    let req = HttpRequest::from_parts(parts, body);

    //
    Ok((path, req))
}

//
pub async fn path_from_extensions<T>(
    extensions: HttpExtensions,
) -> Result<(Option<Path<T>>, HttpExtensions), Error<PathRejection>>
where
    T: DeserializeOwned + Send,
{
    //
    let mut request_parts = {
        let (mut parts, body) = HttpRequest::new(()).into_parts();
        parts.extensions = extensions;
        let req = HttpRequest::from_parts(parts, body);
        RequestParts::new(req)
    };

    //
    let path = match Path::<T>::from_request(&mut request_parts).await {
        Ok(path) => Some(path),
        Err(PathRejection::MissingPathParams(_)) => None,
        Err(err) => {
            return Err(Error::Rejection(err));
        }
    };

    //
    let req = request_parts
        .try_into_request()
        .map_err(Error::BodyAlreadyExtracted)?;
    let (parts, _) = req.into_parts();
    let extensions = parts.extensions;

    //
    Ok((path, extensions))
}
