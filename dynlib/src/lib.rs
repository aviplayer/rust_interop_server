mod router;
mod context;
mod handlers;
mod utils;
mod uri_handler;

use std::ffi::CString;
use std::os::raw::{c_char};
use async_ffi::FutureExt;
use lazy_static::lazy_static;
use tokio::runtime::Runtime;
use std::sync::{Arc};
use hyper::{Response};
use hyper::service::{make_service_fn, service_fn};
use hyper::server::Server;
use crate::context::{AppState, Error};
use crate::router::{Handler, Router};
use futures::lock::Mutex;
use serde_json::Value::Array;
use crate::handlers::{create_body_handler, create_param_handler};
use unicase::UniCase;
use crate::uri_handler::UriHandler;


lazy_static! {
    static ref RUNTIME: Runtime =  Runtime::new().unwrap();
    static ref ROUTER: Arc<Mutex<Router>> = Arc::new(Mutex::new( Router::new()));
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct KRouter {
    pub method: *mut c_char,
    pub uri: *mut c_char,
    pub handler: extern "C" fn(*mut c_char) -> *mut c_char,
}

#[no_mangle]
pub extern "C" fn start_rust_server(raw_addr: *mut c_char, kroutes: *mut KRouter, length: u32) {
    println!("Num of routes {}", length);
    let routes: &mut [KRouter] = unsafe {
        assert!(!kroutes.is_null());
        std::slice::from_raw_parts_mut(kroutes, length as usize)
    };
    let post_pattern: UniCase<&str> = UniCase::new("post");
    let get_pattern: UniCase<&str> = UniCase::new("get");

    let body = async move {
        let routes_vec = routes.to_vec();
        let mut r: Option<&KRouter> = None;
        let mut method: UniCase<&str> = UniCase::new("");
        let mut m: Vec<String> = vec![];
        let mut string_cache: Vec<String> = vec![];

        for i in 0..routes_vec.len() {
            r = Some(routes_vec.get(i).unwrap());
            m.push(utils::c_chars_to_string(r.unwrap().method));
            method = UniCase::new(
                m[i].as_str()
            );

            let uri_h = if method == post_pattern {
                post(r.unwrap().uri, r.unwrap().handler).await
            } else if method == get_pattern {
                get(r.unwrap().uri, r.unwrap().handler).await
            } else {
                eprintln!("unsupported method {}", method.into_inner());
                panic!("Method not supported");
            };

            if method == post_pattern {
                ROUTER.lock().await.post(uri_h.uri.as_str(), uri_h.handler.unwrap());
            } else {
                ROUTER.lock().await.get(uri_h.uri.as_str(), uri_h.handler.unwrap());
            }
            println!("Added handler {} for {}", m[i], uri_h.uri.as_str());
            string_cache.push(uri_h.uri);
        }

        let addr_string = utils::c_chars_to_string(raw_addr);
        let addr = addr_string.parse().expect(
            format!("wrong server address {}", addr_string).as_str()
        );
        let some_state = "state".to_string();
        let new_service = make_service_fn(move |_| {
            let app_state = AppState {
                state_thing: some_state.clone(),
            };
            async {
                Ok::<_, Error>(service_fn(move |req| {
                    router::route(req, app_state.clone())
                }))
            }
        });


        let server = Server::bind(&addr)
            .serve(new_service);
        println!("Server started {} ...", addr);
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    };
    RUNTIME.block_on(body.into_local_ffi());
}

async fn get(uri_raw: *mut c_char,
             callback: extern "C" fn(*mut c_char) -> *mut c_char) -> UriHandler {
    let uri = utils::c_chars_to_string(uri_raw);
    UriHandler {
        uri,
        handler: Some(create_param_handler(callback))
    }
}

async fn post(uri_raw: *mut c_char,
              callback: extern "C" fn(*mut c_char) -> *mut c_char) -> UriHandler {
    let uri = utils::c_chars_to_string(uri_raw);
    UriHandler {
        uri,
        handler: Some(create_body_handler(callback))
    }
}

