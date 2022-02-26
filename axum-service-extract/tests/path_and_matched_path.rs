use core::{convert::Infallible, task::Poll};
use std::{error, net::SocketAddr};

use async_trait::async_trait;
use axum::{http::Request as HttpRequest, response::Response as AxumResponse, Router, Server};
use futures_util::{future::BoxFuture, FutureExt as _, TryFutureExt as _};
use hyper::Body as HyperBody;
use tower_service::Service;

use axum_service_extract::{matched_path::matched_path_from_request, path::path_from_request};

#[tokio::test]
async fn test() -> Result<(), Box<dyn error::Error>> {
    //
    let listen_addr = SocketAddr::from(([127, 0, 0, 1], portpicker::pick_unused_port().unwrap()));
    println!("listen_addr {:?}", listen_addr);

    //
    let server_task = tokio::task::spawn(async move {
        let app = Router::new().route("/path_params/:key", MyService);

        let server = Server::bind(&listen_addr).serve(app.into_make_service());

        server.await.expect("server error");
    });

    //
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    //
    let res = isahc::get_async(format!("http://{}{}", listen_addr, "/path_params/foo")).await?;
    assert!(res.status().is_success());
    assert_eq!(res.headers().get("x").unwrap(), "1");

    //
    server_task.abort();
    assert!(server_task.await.unwrap_err().is_cancelled());

    Ok(())
}

//
#[derive(Clone)]
struct MyService;

#[async_trait]
impl<B> Service<HttpRequest<B>> for MyService
where
    B: Send + 'static,
{
    type Response = AxumResponse;

    type Error = Infallible;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: HttpRequest<B>) -> Self::Future {
        assert_eq!(req.uri(), "/path_params/foo");

        path_from_request::<Vec<String>, B>(req)
            .and_then(|(path, req)| {
                assert_eq!(path.unwrap().0, vec!["foo"]);

                matched_path_from_request(req)
                    .and_then(|(matched_path, _req)| {
                        async move {
                            assert_eq!(matched_path.unwrap().as_str(), "/path_params/:key");

                            let body = HyperBody::empty();
                            let mut res = AxumResponse::new(axum::body::boxed(body));
                            res.headers_mut().insert("x", 1.into());

                            res
                        }
                        .map(Ok)
                        .boxed()
                    })
                    .or_else(|err| {
                        async move {
                            panic!("{}", err);
                        }
                        .map(Ok)
                        .boxed()
                    })
                    .boxed()
            })
            .or_else(|err| {
                async move {
                    panic!("{}", err);
                }
                .map(Ok)
                .boxed()
            })
            .boxed()
    }
}
