use std::ffi::{CStr, CString};
use std::sync::Mutex;

use lazy_static::lazy_static;
use libc::{self, c_int, c_void};

lazy_static! {
    static ref STARTUP_HOOK_CALLBACK: Mutex<Option<Box<dyn Fn() -> i32 + Send>>> = Default::default();
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    Nul,
    Eof,
    InvalidUtf8,
}

mod ffi {
    use libc::{c_char, c_int};

    #[allow(non_camel_case_types)]
    pub type rl_hook_func_t = extern "C" fn() -> c_int;

    #[link(name = "readline")]
    extern "C" {
        pub static mut rl_startup_hook: rl_hook_func_t;

        pub fn readline(prompt: *const c_char) -> *const c_char;
        pub fn rl_insert_text(text: *const c_char) -> c_int;
    }
}

pub fn readline(prompt: &str) -> Result<String, Error> {
    let c_prompt = CString::new(prompt).or(Err(Error::Nul))?;
    let line_ptr = unsafe { ffi::readline(c_prompt.as_ptr()) };

    if line_ptr.is_null() {
        return Err(Error::Eof);
    }

    unsafe {
        let line = CStr::from_ptr(line_ptr)
            .to_str()
            .map(|s| s.to_owned())
            .or(Err(Error::InvalidUtf8));

        libc::free(line_ptr as *mut c_void);

        line
    }
}

pub fn editline(prompt: &str, text: &str) -> Result<String, Error> {
    let text = text.to_owned();

    let cb = Box::new(move || {
        insert_text(&text);
        0
    });

    *STARTUP_HOOK_CALLBACK.lock().unwrap() = Some(cb);

    unsafe { ffi::rl_startup_hook = startup_hook_once; }

    readline(prompt)
}

extern "C" fn startup_hook_once() -> c_int {
    let mut guard = STARTUP_HOOK_CALLBACK.lock().unwrap();

    let result = match *guard {
        Some(ref ctx) => ctx(),
        None => 0,
    };

    *guard = None;

    result
}

fn insert_text(text: &str) -> i32 {
    let c_text = CString::new(text).unwrap();
    unsafe { ffi::rl_insert_text(c_text.as_ptr()) }
}
