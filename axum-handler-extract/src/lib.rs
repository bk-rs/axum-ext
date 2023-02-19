use axum::{
    extract::FromRequestParts,
    http::{
        request::Parts as HttpRequestParts, Extensions as HttpExtensions, Request as HttpRequest,
    },
};

//
pub async fn extensions_extract_from_request<T: FromRequestParts<()>, B>(
    req: HttpRequest<B>,
) -> (Result<T, T::Rejection>, HttpRequest<B>) {
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
    let (extract, extensions) = extract_from_extensions(extensions).await;

    //
    parts.extensions = extensions;
    let req = HttpRequest::from_parts(parts, body);

    //
    (extract, req)
}

//
pub async fn extract_from_extensions<T: FromRequestParts<()>>(
    extensions: HttpExtensions,
) -> (Result<T, T::Rejection>, HttpExtensions) {
    //
    let mut parts = {
        let (mut parts, _) = HttpRequest::new(()).into_parts();
        parts.extensions = extensions;
        parts
    };

    //
    let extract = T::from_request_parts(&mut parts, &()).await;

    //
    let extensions = parts.extensions;

    //
    (extract, extensions)
}

//
pub mod matched_path;
pub mod path;
