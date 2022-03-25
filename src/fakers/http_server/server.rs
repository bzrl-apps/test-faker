use std::net::{SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Write;

//use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_yaml::{Mapping, Value as yamlValue};
use serde_json::{Value as jsonValue, Map};

use anyhow::{anyhow, Result, Error};

use tokio::sync::broadcast::{Sender, Receiver};
//use async_channel::{Sender, Receiver};
//use tokio::runtime::Runtime;
use async_trait::async_trait;

use log::*;

use axum::Router;
use axum::routing::*;
use axum::extract::{Json, Extension, MatchedPath};
use axum::response::IntoResponse;
use axum::http::StatusCode;

use futures::{Future, future};
use std::pin::Pin;


use crate::faker::{Faker, FakerMod};

// Our plugin implementation
#[derive(Default, Debug ,Serialize, Deserialize, Clone, PartialEq)]
struct HttpServer {
    host_addr: String,
    routes: Vec<Route>,
    output_file: Option<String>
}

#[derive(Default, Debug ,Serialize, Deserialize, Clone, PartialEq)]
struct Route {
    path: String,
    method: String,
    status: u16,
    body: Option<String>,
    result: Option<String>,
}

#[async_trait]
impl FakerMod for HttpServer {
    type Future = Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

    fn validate_params(&self) -> Result<()> {

        Ok(())
    }

    fn func(&self, _tx: Sender<bool>, _rx: Receiver<bool>) -> Self::Future {
        let _ =  env_logger::try_init();

        let shared_routes = Arc::new(self.routes.clone());
        let shared_memstore = Arc::new(Mutex::new(Map::new()));

        let mut output = String::new();
        if let Some(o) = &self.output_file {
            output = o.clone();
        }
        let shared_output_file = Arc::new(output);

        // Initialize tracing
        //tracing_subscriber::fmt::init();

        // Build our application with a route
        let mut app = Router::new();
        for r in self.routes.iter() {
            match r.method.as_str() {
                "GET" => app = app.route(r.path.as_str(), get(get_handler)),
                "POST" => app = app.route(r.path.as_str(), post(post_handler)),
                "DELETE" => app = app.route(r.path.as_str(), delete(delete_handler)),
                "PUT" => app = app.route(r.path.as_str(), put(put_handler)),
                _ => return Box::pin(future::err(anyhow!("HTTP Method not supported!"))),
            };

        }
        app = app.layer(Extension(shared_routes))
            .layer(Extension(shared_output_file))
            .layer(Extension(shared_memstore));

        // run our app with hyper
        // `axum::Server` is a re-export of `hyper::Server`
        let addr = match self.host_addr.parse::<SocketAddrV4>() {
            Ok(a) => a,
            Err(e) => return Box::pin(future::err(anyhow!(e))),
        };

        info!("Listening on {}", addr);
        Box::pin(async move {
            if let Err(e) = axum::Server::bind(&SocketAddr::V4(addr))
                .serve(app.into_make_service())
                .await {
                return Err(anyhow!(e));
            }

            Ok(())
        })
    }
}

async fn get_handler(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    matched_path: MatchedPath,
    Extension(routes): Extension<Arc<Vec<Route>>>,
    Extension(output_file): Extension<Arc<String>>,
    Extension(memstore): Extension<Arc<Mutex<Map<String, jsonValue>>>>,
) -> impl IntoResponse {

    let path = matched_path.as_str();
    debug!("GET Path: {}", path);

    let route = routes.iter().find(|r| r.path == path && r.method == "GET");
    if let Some(r) = route.clone() {
        if let Some(res) = &r.result {
            let v_res: jsonValue = serde_json::from_str(&res).unwrap();
            let mut store = memstore.lock().unwrap();

            store.insert(matched_path.as_str().to_string(), v_res.clone());

            write_output_file(output_file, store.clone());

            return (StatusCode::from_u16(r.status).unwrap(), Json(v_res));
        }
    }

    (StatusCode::NOT_FOUND, Json(jsonValue::Null))
}

async fn post_handler(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    matched_path: MatchedPath,
    Json(payload): Json<jsonValue>,
    Extension(routes): Extension<Arc<Vec<Route>>>,
    Extension(output_file): Extension<Arc<String>>,
    Extension(memstore): Extension<Arc<Mutex<Map<String, jsonValue>>>>,
) -> impl IntoResponse {

    let path = matched_path.as_str();
    debug!("POST Path: {}", path);

    let route = routes.iter().find(|r| r.path == path && r.method == "POST" && serde_json::from_str::<jsonValue>(&r.body.as_ref().unwrap()).unwrap() == payload);
    if let Some(r) = route.clone() {
        if let Some(res) = &r.result {
            let v_res: jsonValue = serde_json::from_str(&res).unwrap();
            let mut store = memstore.lock().unwrap();

            store.insert(matched_path.as_str().to_string(), v_res.clone());

            write_output_file(output_file, store.clone());

            return (StatusCode::from_u16(r.status).unwrap(), Json(v_res));
        }
    }

    (StatusCode::NOT_FOUND, Json(jsonValue::Null))
}

async fn delete_handler(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    matched_path: MatchedPath,
    Extension(routes): Extension<Arc<Vec<Route>>>,
    Extension(output_file): Extension<Arc<String>>,
    Extension(memstore): Extension<Arc<Mutex<Map<String, jsonValue>>>>,
) -> impl IntoResponse {

    let path = matched_path.as_str();
    debug!("DELETE Path: {}", path);

    let route = routes.iter().find(|r| r.path == path && r.method == "DELETE");
    if let Some(r) = route.clone() {
        let mut store = memstore.lock().unwrap();

        store.remove(matched_path.as_str());

        write_output_file(output_file, store.clone());

        if let Some(res) = &r.result {
            let v_res: jsonValue = serde_json::from_str(&res).unwrap();

            return (StatusCode::from_u16(r.status).unwrap(), Json(v_res));
        }

        return (StatusCode::from_u16(r.status).unwrap(), Json(jsonValue::Null))
    }

    (StatusCode::NOT_FOUND, Json(jsonValue::Null))
}

async fn put_handler(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    matched_path: MatchedPath,
    Json(payload): Json<jsonValue>,
    Extension(routes): Extension<Arc<Vec<Route>>>,
    Extension(output_file): Extension<Arc<String>>,
    Extension(memstore): Extension<Arc<Mutex<Map<String, jsonValue>>>>,
) -> impl IntoResponse {

    let path = matched_path.as_str();
    debug!("PUT Path: {}", path);

    let route = routes.iter().find(|r| r.path == path && r.method == "PUT" && serde_json::from_str::<jsonValue>(&r.body.as_ref().unwrap()).unwrap() == payload);
    if let Some(r) = route.clone() {
        if let Some(res) = &r.result {
            let v_res: jsonValue = serde_json::from_str(&res).unwrap();
            let mut store = memstore.lock().unwrap();

            store.insert(matched_path.as_str().to_string(), v_res.clone());

            write_output_file(output_file, store.clone());

            return (StatusCode::from_u16(r.status).unwrap(), Json(v_res));
        }
    }

    (StatusCode::NOT_FOUND, Json(jsonValue::Null))
}

fn write_output_file(output_file: Arc<String>, store: Map<String, jsonValue>) {
    if !output_file.is_empty() {
        let mut f = File::create(output_file.as_ref()).unwrap();
        let text = serde_json::to_string_pretty(&store).unwrap();

        f.write_all(text.as_bytes()).unwrap();
        f.write_all(b"\n").unwrap();
    }
}

fn func(params: Mapping, tx: Sender<bool>, rx: Receiver<bool>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> {
    Box::pin(async move {
        let v_params = yamlValue::Mapping(params);

        let server: HttpServer = serde_yaml::from_value(v_params)?;

        server.validate_params()?;
        server.func(tx, rx).await?;

        Ok(())

    })
}

inventory::submit!(Faker {name: "http-server", func: func });

#[cfg(test)]
mod tests {
    use reqwest::header::CONTENT_TYPE;

    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn httpserver_func() {
        let params: Mapping = serde_yaml::from_str(r#"
host_addr: "127.0.0.1:3000"
routes:
- path: /prefix1/suffix1
  method: "GET"
  status: 200
  result: |
    {
      "message": "hello world1"
    }
- path: /prefix1/suffix2
  method: "POST"
  status: 200
  body: |
    {
      "payload": "suffix2"
    }
  result: |
    {
      "message": "hello world2"
    }
- path: /prefix1/suffix1
  method: "DELETE"
  status: 200
- path: /prefix1/suffix2
  method: "PUT"
  status: 200
  body: |
    {
      "payload": "put suffix2"
    }
  result: |
    {
      "message": "put suffix2"
    }
output_file: /tmp/httpserver.json
"#).unwrap();

        let (tx, rx) = tokio::sync::broadcast::channel(16);
        let server: HttpServer = serde_yaml::from_value(yamlValue::Mapping(params)).unwrap();
        server.validate_params().unwrap();

        let server_cloned = server.clone();

        tokio::spawn(async move{
            let _ = server.func(tx, rx).await;
        });

        // Send a post request
        let client = reqwest::Client::new();

        for r in server_cloned.routes.into_iter() {
            if r.method == "GET" {
                let response = client.get(format!("http://localhost:3000{}", r.path).as_str())
                    .header(CONTENT_TYPE, "application/json")
                    .send()
                    .await
                    .unwrap();

                assert_eq!(r.status, response.status().as_u16());
                assert_eq!(serde_json::from_str::<jsonValue>(r.result.as_ref().unwrap()).unwrap(), response.json::<jsonValue>().await.unwrap())
            }

            if r.method == "POST" {
                let response = client.post(format!("http://localhost:3000{}", r.path).as_str())
                    .header(CONTENT_TYPE, "application/json")
                    .body(r.body.clone().unwrap())
                    .send()
                    .await
                    .unwrap();

                assert_eq!(r.status, response.status().as_u16());
                assert_eq!(serde_json::from_str::<jsonValue>(r.result.as_ref().unwrap()).unwrap(), response.json::<jsonValue>().await.unwrap())
            }

            if r.method == "DELETE" {
                let response = client.delete(format!("http://localhost:3000{}", r.path).as_str())
                    .header(CONTENT_TYPE, "application/json")
                    .send()
                    .await
                    .unwrap();

                assert_eq!(r.status, response.status().as_u16());
            }

            if r.method == "PUT" {
                let response = client.put(format!("http://localhost:3000{}", r.path).as_str())
                    .header(CONTENT_TYPE, "application/json")
                    .body(r.body.clone().unwrap())
                    .send()
                    .await
                    .unwrap();

                assert_eq!(r.status, response.status().as_u16());
                assert_eq!(serde_json::from_str::<jsonValue>(r.result.as_ref().unwrap()).unwrap(), response.json::<jsonValue>().await.unwrap())
            }
        }
    }
}
