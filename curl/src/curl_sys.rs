use std::ffi::c_int;
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct CURL(*const ());
pub type CURLcode = c_int;
pub type CURLoption = c_int;

#[link(name = "curl")]
unsafe extern "C" {
    pub fn curl_easy_init() -> CURL;
    pub fn curl_easy_setopt(handle: CURL, option: CURLoption, ...) -> CURLcode;
    pub fn curl_easy_perform(handle: CURL) -> CURLcode;
    pub fn curl_easy_cleanup(handle: CURL);
}

pub const CURLOPT_URL: CURLcode = 10002;
pub const CURLOPT_WRITEFUNCTION: CURLcode = 20011;
pub const CURLOPT_WRITEDATA: CURLcode = 10001;
pub const CURL_OK: CURLcode = 0;
