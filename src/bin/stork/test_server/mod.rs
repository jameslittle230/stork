extern crate stork_search as stork;
use stork::LatestVersion::structs::Index;

#[cfg(not(feature = "test-server"))]
pub fn serve(_index: &Index, _port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("Stork was not compiled with test server support. Rebuild the crate with default features to enable the test server.\nIf you don't expect to see this, file a bug: https://jil.im/storkbug\n");
    panic!()
}

#[cfg(feature = "test-server")]
pub fn serve(index: &Index, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Request, Response, Server, StatusCode};
    use std::convert::Infallible;
    use tokio::runtime::Runtime;

    let rt = Runtime::new()?;
    let index_bytes: Vec<u8> = index.to_bytes();

    rt.block_on(async {
        // For every connection, we must make a `Service` to handle all
        // incoming HTTP requests on said connection.
        let make_svc = make_service_fn(|_conn| {
            // This is the `Service` that will handle the connection.
            // `service_fn` is a helper to convert a function that
            // returns a Response into a `Service`.
            let bytes = index_bytes.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |request: Request<Body>| {
                    let bytes_2 = bytes.clone();
                    async move {
                        Ok::<_, Infallible>(match request.uri().to_string().as_str() {
                            "/" => {
                                let index_html = include_str!("index.html");
                                Response::new(Body::from(index_html))
                            }

                            "/test.st" => Response::new(Body::from(bytes_2)),

                            _ => Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Body::from("404: Not found."))
                                .unwrap(),
                        })
                    }
                }))
            }
        });

        let addr = ([127, 0, 0, 1], port).into();
        let server = Server::bind(&addr).serve(make_svc);
        let graceful = server.with_graceful_shutdown(shutdown_signal());

        println!("Open <http://{}> in your web browser to visit the test page.\nPress ctrl-C to stop the server.", addr);

        if let Err(e) = graceful.await {
            eprintln!("server error: {}", e);
        }
        Ok(())
    })
}

#[cfg(feature = "test-server")]
async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
