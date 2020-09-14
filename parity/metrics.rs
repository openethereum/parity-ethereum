use std::{sync::Arc, time::Instant};

use crate::{futures::Future, rpc, rpc_apis};

use parking_lot::Mutex;

use hyper::{service::service_fn_ok, Body, Method, Request, Response, Server, StatusCode};

use stats::{
    prometheus::{self, Encoder},
    prometheus_gauge, PrometheusMetrics,
};

#[derive(Debug, Clone, PartialEq)]
pub struct MetricsConfiguration {
    /// Are metrics enabled (default is false)?
    pub enabled: bool,
    /// The IP of the network interface used (default is 127.0.0.1).
    pub interface: String,
    /// The network port (default is 3000).
    pub port: u16,
}

impl Default for MetricsConfiguration {
    fn default() -> Self {
        MetricsConfiguration {
            enabled: false,
            interface: "127.0.0.1".into(),
            port: 3000,
        }
    }
}

struct State {
    rpc_apis: Arc<rpc_apis::FullDependencies>,
}

fn handle_request(req: Request<Body>, state: Arc<Mutex<State>>) -> Response<Body> {
    let (parts, _body) = req.into_parts();
    match (parts.method, parts.uri.path()) {
        (Method::GET, "/metrics") => {
            let start = Instant::now();

            let mut reg = prometheus::Registry::new();
            let state = state.lock();
            state.rpc_apis.client.prometheus_metrics(&mut reg);
            state.rpc_apis.sync.prometheus_metrics(&mut reg);
            let elapsed = start.elapsed();
            prometheus_gauge(
                &mut reg,
                "metrics_time",
                "Time to perform rpc metrics",
                elapsed.as_millis() as i64,
            );

            let mut buffer = vec![];
            let encoder = prometheus::TextEncoder::new();
            let metric_families = reg.gather();

            encoder
                .encode(&metric_families, &mut buffer)
                .expect("all source of metrics are static; qed");
            let text = String::from_utf8(buffer).expect("metrics encoding is ASCII; qed");

            Response::new(Body::from(text))
        }
        (_, _) => {
            let mut res = Response::new(Body::from("not found"));
            *res.status_mut() = StatusCode::NOT_FOUND;
            res
        }
    }
}

/// Start the prometheus metrics server accessible via GET <host>:<port>/metrics
pub fn start_prometheus_metrics(
    conf: &MetricsConfiguration,
    deps: &rpc::Dependencies<rpc_apis::FullDependencies>,
) -> Result<(), String> {
    if !conf.enabled {
        return Ok(());
    }

    let addr = format!("{}:{}", conf.interface, conf.port);
    let addr = addr
        .parse()
        .map_err(|err| format!("Failed to parse address '{}': {}", addr, err))?;

    let state = State {
        rpc_apis: deps.apis.clone(),
    };
    let state = Arc::new(Mutex::new(state));

    let server = Server::bind(&addr)
        .serve(move || {
            // This is the `Service` that will handle the connection.
            // `service_fn_ok` is a helper to convert a function that
            // returns a Response into a `Service`.
            let state = state.clone();
            service_fn_ok(move |req: Request<Body>| handle_request(req, state.clone()))
        })
        .map_err(|e| eprintln!("server error: {}", e));
    println!("Listening on http://{}", addr);

    deps.executor.spawn(server);

    Ok(())
}
