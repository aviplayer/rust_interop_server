use std::ffi::{CString};
use std::os::raw::{c_char};
use hyper::StatusCode;
use lazy_static::lazy_static;
use crate::context::{Context};
use crate::{Response, utils};
use crate::router::Handler;

static  mut SIMPLE_CACHE: Vec<String> = vec![];

pub fn create_param_handler(callback: extern "C" fn(*mut c_char) -> *mut c_char) ->
Box<dyn Handler> {
    let fun = move |ctx: Context| async move {
        let param = match ctx.params.find("param") {
            Some(v) => v,
            None => "empty",
        };
        let str_bytes = param.as_bytes().to_vec();
        let c_string = unsafe {
            CString::from_vec_unchecked(str_bytes)
        };
        let res = callback(c_string.into_raw());
        let res_c_str = unsafe {
            CString::from_raw(res)
        };
        match res_c_str.into_string() {
            Ok(res_str) => unsafe {
                println!("Get response is {}", res_str);
                let res = res_str.clone();
                SIMPLE_CACHE.push( res_str);
                res
            },
            Err(err) => {
                eprint!("Got an error while parsing res {} ", err);
                panic!("{}", err);
            }
        }
    };
    Box::new(fun)
}

pub fn create_body_handler(callback: extern "C" fn(*mut c_char) -> *mut c_char) ->
Box<dyn Handler> {
    let fun = move |mut ctx: Context| async move {
        let body: String = match ctx.body_json().await {
            Ok(res_json) => res_json,
            Err(e) => {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(format!("could not parse JSON: {}", e).into())
                    .unwrap();
            }
        };
        process_response(body, callback)
    };
    Box::new(fun)
}

fn process_response(response: String, callback: extern "C" fn(*mut c_char) -> *mut c_char) ->
hyper::Response<hyper::Body> {
    let response_raw = unsafe {
        CString::from_vec_unchecked(response.as_bytes().to_vec()).into_raw()
    };
    let res_raw = callback(response_raw);
    let res = utils::c_chars_to_string(res_raw);
    let res_to_return = res.clone();
    unsafe {
        SIMPLE_CACHE.push(res);
    }
    Response::builder()
        .status(StatusCode::OK)
        .body(res_to_return.into())
        .unwrap()
}
