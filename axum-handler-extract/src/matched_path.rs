use axum::{
    extract::{rejection::MatchedPathRejection, MatchedPath},
    http::{Extensions as HttpExtensions, Request as HttpRequest},
};

use crate::{extensions_extract_from_request, extract_from_extensions};

//
pub async fn matched_path_from_request<B>(
    req: HttpRequest<B>,
) -> Result<(Option<MatchedPath>, HttpRequest<B>), (MatchedPathRejection, HttpRequest<B>)> {
    let (matched_path, req) = extensions_extract_from_request(req).await;
    match matched_path {
        Ok(x) => Ok((Some(x), req)),
        Err(MatchedPathRejection::MatchedPathMissing(_)) => Ok((None, req)),
        Err(err) => Err((err, req)),
    }
}

//
pub async fn matched_path_from_extensions(
    extensions: HttpExtensions,
) -> Result<(Option<MatchedPath>, HttpExtensions), (MatchedPathRejection, HttpExtensions)> {
    let (matched_path, extensions) = extract_from_extensions(extensions).await;
    match matched_path {
        Ok(x) => Ok((Some(x), extensions)),
        Err(MatchedPathRejection::MatchedPathMissing(_)) => Ok((None, extensions)),
        Err(err) => Err((err, extensions)),
    }
}
