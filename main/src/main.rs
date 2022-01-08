use std::ffi::CString;
use std::os::raw::c_char;
use lib_server::*;

fn main() {
    unsafe {
        let uri_get = CString::from_vec_unchecked(b"/param/:param"
            .to_vec()).into_raw();
        let uri_post = CString::from_vec_unchecked(b"/body"
            .to_vec()).into_raw();

        extern "C" fn callback_get(name: *mut c_char) -> *mut c_char {
            let c_name =
                unsafe {
                    CString::from_raw(name)
                };
            let res = format!("Hello from {:?}", c_name).as_bytes().to_vec();
            unsafe {
                CString::from_vec_unchecked(res).into_raw()
            }
        }

        extern "C" fn callback_post(json_raw: *mut c_char) -> *mut c_char {
            let json =
                unsafe {
                    CString::from_raw(json_raw)
                };
            println!("Got json: \n{:?}", json);
            let res = "{\"response\":\"hello\"}".as_bytes().to_vec();
            unsafe {
                CString::from_vec_unchecked(res).into_raw()
            }
        }

        get(uri_get, callback_get);
        post(uri_post, callback_post);
        println!("server is starting ....");
        let addr_bytes = b"127.0.0.1:8087".to_vec();
        let c_ctring = CString::from_vec_unchecked(addr_bytes);
        start_rust_server(c_ctring.into_raw())
    }
}
