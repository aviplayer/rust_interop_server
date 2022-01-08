use std::ffi::CString;
use std::os::raw::c_char;

pub fn c_chars_to_string(chars: *mut c_char) -> String {
    let c_str =
        unsafe {
            CString::from_raw(chars)
        };
    match c_str.into_string() {
        Ok(add) => add,
        Err(err) => {
            eprintln!("Error while parsing URL {}", err);
            panic!("{}", err)
        }
    }
}
