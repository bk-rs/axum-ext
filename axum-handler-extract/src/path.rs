use axum::{
    extract::{rejection::PathRejection, Path},
    http::{Extensions as HttpExtensions, Request as HttpRequest},
};
use serde::de::DeserializeOwned;

use crate::{extensions_extract_from_request, extract_from_extensions};

//
pub async fn path_from_request<T, B>(
    req: HttpRequest<B>,
) -> Result<(Option<Path<T>>, HttpRequest<B>), (PathRejection, HttpRequest<B>)>
where
    T: DeserializeOwned + Send,
{
    let (path, req) = extensions_extract_from_request(req).await;
    match path {
        Ok(x) => Ok((Some(x), req)),
        Err(PathRejection::MissingPathParams(_)) => Ok((None, req)),
        Err(err) => Err((err, req)),
    }
}

//
pub async fn path_from_extensions<T>(
    extensions: HttpExtensions,
) -> Result<(Option<Path<T>>, HttpExtensions), (PathRejection, HttpExtensions)>
where
    T: DeserializeOwned + Send,
{
    let (path, extensions) = extract_from_extensions(extensions).await;
    match path {
        Ok(x) => Ok((Some(x), extensions)),
        Err(PathRejection::MissingPathParams(_)) => Ok((None, extensions)),
        Err(err) => Err((err, extensions)),
    }
}
