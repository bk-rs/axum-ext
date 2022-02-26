use axum::{
    extract::{rejection::MatchedPathRejection, FromRequest as _, MatchedPath, RequestParts},
    http::{
        request::Parts as HttpRequestParts, Extensions as HttpExtensions, Request as HttpRequest,
    },
};

use crate::error::Error;

//
pub async fn matched_path_from_request<B>(
    req: HttpRequest<B>,
) -> Result<(Option<MatchedPath>, HttpRequest<B>), Error<MatchedPathRejection>> {
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
    let (matched_path, extensions) = matched_path_from_extensions(extensions).await?;

    //
    parts.extensions = extensions;
    let req = HttpRequest::from_parts(parts, body);

    //
    Ok((matched_path, req))
}

//
pub async fn matched_path_from_extensions(
    extensions: HttpExtensions,
) -> Result<(Option<MatchedPath>, HttpExtensions), Error<MatchedPathRejection>> {
    //
    let mut request_parts = {
        let (mut parts, body) = HttpRequest::new(()).into_parts();
        parts.extensions = extensions;
        let req = HttpRequest::from_parts(parts, body);
        RequestParts::new(req)
    };

    //
    let matched_path = match MatchedPath::from_request(&mut request_parts).await {
        Ok(path) => Some(path),
        Err(MatchedPathRejection::MatchedPathMissing(_)) => None,
        Err(err) => {
            return Err(Error::Rejection(err));
        }
    };

    //
    let req = request_parts.try_into_request().map_err(Error::AxumError)?;
    let (parts, _) = req.into_parts();
    let extensions = parts.extensions;

    //
    Ok((matched_path, extensions))
}

//
//
//
pub async fn matched_path_from_request_without_from_request<B>(
    req: &HttpRequest<B>,
) -> Option<MatchedPath> {
    matched_path_from_extensions_without_from_request(req.extensions())
}

pub fn matched_path_from_extensions_without_from_request(
    extensions: &HttpExtensions,
) -> Option<MatchedPath> {
    extensions.get::<MatchedPath>().cloned()
}
