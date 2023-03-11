use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use axum::{
    body::{Body as AxumBody, StreamBody as AxumStreamBody},
    http::Request as HttpRequest,
    response::Response as AxumResponse,
};
use futures_util::{stream::unfold, TryStreamExt as _};
use isahc::{
    AsyncBody as IsahcAsyncBody, AsyncReadResponseExt as _, Error as IsahcError, HttpClient,
};

//
pub async fn send(
    client: &HttpClient,
    http_request: HttpRequest<AxumBody>,
) -> Result<AxumResponse, IsahcError> {
    let isahc_request = {
        let (parts, body) = http_request.into_parts();
        let body = body
            .map_ok(|x| x.to_vec())
            .map_err(|err| IoError::new(IoErrorKind::Other, err));
        let body = IsahcAsyncBody::from_reader(body.into_async_read());
        HttpRequest::from_parts(parts, body)
    };
    let isahc_response = client.send_async(isahc_request).await?;
    let http_response = {
        let mut response = AxumResponse::new(());
        *response.status_mut() = isahc_response.status();
        *response.version_mut() = isahc_response.version();
        *response.headers_mut() = isahc_response.headers().to_owned();

        let buf = vec![];
        let body_stream = unfold(
            (isahc_response, buf),
            |(mut isahc_response, mut buf)| async {
                match isahc_response.copy_to(&mut buf).await {
                    Ok(n) if n == 0 => None,
                    Ok(n) => Some((Ok(buf[..n as usize].to_vec()), (isahc_response, buf))),
                    Err(err) => Some((Err(err), (isahc_response, buf))),
                }
            },
        );

        let body = AxumStreamBody::new(body_stream);

        let (parts, _) = response.into_parts();
        AxumResponse::from_parts(parts, axum::body::boxed(body))
    };
    Ok(http_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::net::SocketAddr;

    use axum::{routing::get, Router, Server};

    #[tokio::test]
    async fn test_send() -> Result<(), Box<dyn std::error::Error>> {
        //
        let backend_listen_addr = SocketAddr::from((
            [127, 0, 0, 1],
            portpicker::pick_unused_port().expect("No ports free"),
        ));
        let server_listen_addr = SocketAddr::from((
            [127, 0, 0, 1],
            portpicker::pick_unused_port().expect("No ports free"),
        ));

        //
        let backend_task = tokio::task::spawn(async move {
            let app = Router::new().route("/", get(|| async { "backend" }));

            let server = Server::bind(&backend_listen_addr).serve(app.into_make_service());

            server.await.expect("backend start failed");
        });

        //
        let server_task = tokio::task::spawn(async move {
            use axum::{body::Body, http::Request};

            let app = Router::new().route(
                "/",
                get(move |mut request: Request<Body>| async move {
                    *request.uri_mut() = format!("http://{}{}", backend_listen_addr, "/")
                        .parse()
                        .unwrap();
                    let client = isahc::HttpClient::new().unwrap();
                    send(&client, request).await.unwrap()
                }),
            );

            let server = Server::bind(&server_listen_addr).serve(app.into_make_service());

            server.await.expect("server start failed");
        });

        //
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        //
        let mut resp = isahc::get_async(format!("http://{}{}", server_listen_addr, "/")).await?;
        assert!(resp.status().is_success());
        assert_eq!(resp.bytes().await.unwrap(), b"backend");

        //
        server_task.abort();
        assert!(server_task.await.unwrap_err().is_cancelled());

        backend_task.abort();
        assert!(backend_task.await.unwrap_err().is_cancelled());

        Ok(())
    }
}
