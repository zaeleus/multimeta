use std::ffi::{CStr, CString};

use libc::{self, c_void};

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    Nul,
    Eof,
    InvalidUtf8,
}

mod ffi {
    use libc::c_char;

	#[link(name = "readline")]
	extern {
		pub fn readline(prompt: *const c_char) -> *const c_char;
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
