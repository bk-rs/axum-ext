use axum::{
    extract::{rejection::PathRejection, FromRequestParts as _, Path},
    http::{
        request::Parts as HttpRequestParts, Extensions as HttpExtensions, Request as HttpRequest,
    },
};
use serde::de::DeserializeOwned;

//
pub async fn path_from_request<T, B>(
    req: HttpRequest<B>,
) -> Result<(Option<Path<T>>, HttpRequest<B>), PathRejection>
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
) -> Result<(Option<Path<T>>, HttpExtensions), PathRejection>
where
    T: DeserializeOwned + Send,
{
    //
    let mut parts = {
        let (mut parts, _) = HttpRequest::new(()).into_parts();
        parts.extensions = extensions;
        parts
    };

    //
    let path = match Path::<T>::from_request_parts(&mut parts, &()).await {
        Ok(path) => Some(path),
        Err(PathRejection::MissingPathParams(_)) => None,
        Err(err) => {
            return Err(err);
        }
    };

    //
    let extensions = parts.extensions;

    //
    Ok((path, extensions))
}
