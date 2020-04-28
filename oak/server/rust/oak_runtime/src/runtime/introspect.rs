//
// Copyright 2020 The Project Oak Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Error, Method, Request, Response, Server, StatusCode,
};
use log::info;
use regex::Regex;
use std::{net::SocketAddr, sync::Arc};

use crate::runtime::Runtime;

// TODO(#672): Shift to a templating library.
fn dot_wrap(title: &str, graph: &str) -> String {
    format!(
        r###"
<!DOCTYPE html>
<meta charset="utf-8">
<title>{}</title>
<body>
<script src="https://d3js.org/d3.v5.min.js"
        integrity="sha384-M06Cb6r/Yrkprjr7ngOrJlzgekrkkdmGZDES/SUnkUpUol0/qjsaQWQfLzq0mcfg"
        crossorigin="anonymous"></script>
<script src="https://unpkg.com/@hpcc-js/wasm@0.3.6/dist/index.min.js"
        integrity="sha384-y8QckRXESCyo6DMRfBy8RKSRf6q/D6U6D8XUbUU5poU9144yTw98vyhHVjky6jCY"
        crossorigin="anonymous"></script>
<script src="https://unpkg.com/d3-graphviz@3.0.0/build/d3-graphviz.js"
        integrity="sha384-rhFJRiWvj+zkVQouZi5hc+vHuqhp1N8AnncxPSaKQ+PUhODpXC3Pgy4EcxjYCIYA"
        crossorigin="anonymous"></script>
<div id="graph" style="text-align: center;"></div>
<script>d3.select("#graph").graphviz().renderDot(`
{}
`);</script>
"###,
        title, graph,
    )
}

fn html_wrap(title: &str, body: &str) -> String {
    format!(
        r###"
<!DOCTYPE html>
<meta charset="utf-8">
<title>{}</title>
<body>
{}
"###,
        title, body
    )
}

// Look for a path like "/{kind}/{id}" and return the id value.
fn find_id(path: &str, kind: &str) -> Option<u64> {
    Regex::new(&format!(r"^/{}/(?P<id>\d+)/?$", kind))
        .unwrap()
        .captures(path)?
        .name("id")
        .unwrap()
        .as_str()
        .parse::<u64>()
        .ok()
}

fn handle_request(
    req: Request<Body>,
    runtime: Arc<Runtime>,
) -> Result<Response<Body>, hyper::Error> {
    if req.method() != Method::GET {
        let mut not_found = Response::default();
        *not_found.status_mut() = StatusCode::NOT_FOUND;
        return Ok(not_found);
    }
    // TODO(#672): Shift to a framework.
    let path = req.uri().path();
    if path == "/" {
        return Ok(Response::new(Body::from(html_wrap(
            "Data Structures",
            &runtime.html(),
        ))));
    } else if path == "/graph" {
        return Ok(Response::new(Body::from(dot_wrap(
            "Runtime Graph",
            &runtime.graph(),
        ))));
    } else if let Some(node_id) = find_id(path, "node") {
        if let Some(body) = runtime.html_for_node(node_id) {
            return Ok(Response::new(Body::from(html_wrap(
                &format!("Node {}", node_id),
                &body,
            ))));
        }
    } else if let Some(channel_id) = find_id(path, "channel") {
        if let Some(body) = runtime.html_for_channel(channel_id) {
            return Ok(Response::new(Body::from(html_wrap(
                &format!("Channel {}", channel_id),
                &body,
            ))));
        }
    } else if let Some(half_id) = find_id(path, "half") {
        if let Some(body) = runtime.html_for_half(half_id) {
            return Ok(Response::new(Body::from(html_wrap(
                &format!("Channel Half {}", half_id),
                &body,
            ))));
        }
    }
    let mut not_found = Response::default();
    *not_found.status_mut() = StatusCode::NOT_FOUND;
    Ok(not_found)
}

async fn make_server(port: u16, runtime: Arc<Runtime>) {
    // Construct our SocketAddr to listen on...
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("starting introspection server on {:?}", addr);

    // And a MakeService to handle each connection...
    let make_service = make_service_fn(move |_| {
        // The `Arc<Runtime>` is moved into this closure, but needs to be cloned
        // because this closure is called for every connection.
        let runtime = runtime.clone();

        async move {
            Ok::<_, Error>(service_fn(move |req| {
                let runtime = runtime.clone();
                async move { handle_request(req, runtime) }
            }))
        }
    });

    // Then bind and serve...
    let server = Server::bind(&addr).serve(make_service);

    // And run forever...
    let result = server.await;
    info!("introspection server terminated with {:?}", result);
}

// TODO(#908): add a termination mechanism so that this `Arc<Runtime>` doesn't
// force the `Runtime` instance to be undroppable.
pub fn serve(port: u16, runtime: Arc<Runtime>) {
    let mut tokio_runtime = tokio::runtime::Runtime::new().expect("Couldn't create Tokio runtime");
    tokio_runtime.block_on(make_server(port, runtime))
}