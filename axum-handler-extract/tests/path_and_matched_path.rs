use core::{future::Future, pin::Pin};
use std::net::SocketAddr;

use async_trait::async_trait;
use axum::{
    handler::Handler, http::Request as HttpRequest, response::Response as AxumResponse,
    routing::get, Router, Server,
};
use hyper::Body as HyperBody;

use axum_handler_extract::{matched_path::matched_path_from_request, path::path_from_request};

#[tokio::test]
async fn simple() -> Result<(), Box<dyn std::error::Error>> {
    //
    let listen_addr = SocketAddr::from(([127, 0, 0, 1], portpicker::pick_unused_port().unwrap()));
    println!("listen_addr {listen_addr:?}");

    //
    let server_task = tokio::task::spawn(async move {
        let app = Router::new().route("/path_params/:key", get(MyHandler));

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
struct MyHandler;

#[async_trait]
impl<S, B> Handler<(), S, B> for MyHandler
where
    S: Clone + 'static,
    B: Send + 'static,
{
    type Future = Pin<Box<dyn Future<Output = AxumResponse> + Send + 'static>>;

    fn call(self, req: HttpRequest<B>, _state: S) -> Self::Future {
        Box::pin(async move {
            assert_eq!(req.uri(), "/path_params/foo");

            let req = match path_from_request::<Vec<String>, B>(req).await {
                Ok((path, req)) => {
                    assert_eq!(path.unwrap().0, vec!["foo"]);

                    req
                }
                Err(err) => {
                    panic!("{}", err);
                }
            };

            match matched_path_from_request(req).await {
                Ok((matched_path, _req)) => {
                    assert_eq!(matched_path.unwrap().as_str(), "/path_params/:key");
                }
                Err(err) => {
                    panic!("{}", err);
                }
            }

            let body = HyperBody::empty();
            let mut res = AxumResponse::new(axum::body::boxed(body));
            res.headers_mut().insert("x", 1.into());

            res
        })
    }
}
