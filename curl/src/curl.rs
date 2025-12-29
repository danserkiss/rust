use crate::curl_sys::{self, CURL_OK, CURLOPT_URL, CURLOPT_WRITEDATA, CURLOPT_WRITEFUNCTION};
use std::ffi::{CString, c_char, c_void};

#[repr(transparent)]
pub struct CURL(curl_sys::CURL);

impl CURL {
    pub fn new() -> Self {
        unsafe { Self(curl_sys::curl_easy_init()) }
    }

    pub fn set_url(&mut self, url: &str) -> Result<(), String> {
        let c_url = CString::new(url).map_err(|_| "Invalid URL")?;
        let res =
            unsafe { curl_sys::curl_easy_setopt(self.0.clone(), CURLOPT_URL, c_url.as_ptr()) };
        self.check_res(res)
    }

    pub fn set_write_to_string(&mut self, dest: &mut String) -> Result<(), String> {
        unsafe {
            self.check_res(curl_sys::curl_easy_setopt(
                self.0.clone(),
                CURLOPT_WRITEFUNCTION,
                write_to_string
                    as extern "C" fn(
                        ptr: *mut c_char,
                        size: usize,
                        nmemb: usize,
                        userdata: *mut c_void,
                    ) -> usize,
            ))?;
            self.check_res(curl_sys::curl_easy_setopt(
                self.0.clone(),
                CURLOPT_WRITEDATA,
                dest as *mut String as *mut c_void,
            ))?;
        }
        Ok(())
    }

    pub fn perform(&mut self) -> Result<(), String> {
        let res = unsafe { curl_sys::curl_easy_perform(self.0.clone()) };
        self.check_res(res)
    }

    fn check_res(&self, res: curl_sys::CURLcode) -> Result<(), String> {
        if res == CURL_OK {
            Ok(())
        } else {
            Err(format!("CURL error: {}", res))
        }
    }
}

impl Drop for CURL {
    fn drop(&mut self) {
        unsafe { curl_sys::curl_easy_cleanup(self.0.clone()) };
    }
}

extern "C" fn write_to_string(
    ptr: *mut c_char,
    size: usize,
    nmemb: usize,
    userdata: *mut c_void,
) -> usize {
    let s = unsafe { &mut *(userdata as *mut String) };
    let data = unsafe { std::slice::from_raw_parts(ptr as *const u8, nmemb) };

    if let Ok(text) = std::str::from_utf8(data) {
        s.push_str(text);
        size * nmemb
    } else {
        0
    }
}
