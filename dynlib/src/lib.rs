mod router;
mod context;
mod handlers;
mod utils;

use std::os::raw::{c_char, c_int, c_short};
use async_ffi::FutureExt;
use lazy_static::lazy_static;
use tokio::runtime::Runtime;
use std::sync::{Arc};
use hyper::{Response};
use hyper::service::{make_service_fn, service_fn};
use hyper::server::Server;
use crate::context::{AppState, Error};
use crate::router::{Router};
use futures::lock::Mutex;
use crate::handlers::{create_body_handler, create_param_handler, create_test_handler};


lazy_static! {
    static ref RUNTIME: Runtime =  Runtime::new().unwrap();
    static ref ROUTER: Arc<Mutex<Router>> = Arc::new(Mutex::new( Router::new()));
}

#[no_mangle]
pub extern "C" fn start_rust_server(raw_addr: *mut c_char) {
    let body = async move {
        let addr_string = utils::c_chars_to_string(raw_addr);
        let addr = addr_string.parse().expect(
            format!("wrong serer address {}", addr_string).as_str()
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
    }.into_local_ffi();
    println!("Starting tokio runtime ...");
    RUNTIME.block_on(body.into_local_ffi());
}

#[no_mangle]
pub extern "C" fn get(uri_raw: *mut c_char,
                      callback: extern "C" fn(*mut c_char) -> *mut c_char) {
    let async_block = async move {
        let uri = utils::c_chars_to_string(uri_raw);
        let handler_box = create_param_handler(callback);
        ROUTER.lock().await.get(uri.as_str(), handler_box);
        println!("Handler added for {}", uri)
    };
    RUNTIME.block_on(async_block);
}

#[no_mangle]
pub extern "C" fn post(uri_raw: *mut c_char,
                       callback: extern "C" fn(*mut c_char) -> *mut c_char) {
    let async_block = async move {
        let uri = utils::c_chars_to_string(uri_raw);
        let handler_box = create_body_handler(callback);
        ROUTER.lock().await.post(uri.as_str(), handler_box);
        println!("Handler added for {}", uri)
    };
    RUNTIME.block_on(async_block);
}
