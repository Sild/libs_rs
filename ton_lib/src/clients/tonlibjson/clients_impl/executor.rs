use crate::clients::tonlibjson::client_raw::tl_request::TLRequest;
use crate::clients::tonlibjson::client_raw::tl_response::TLResponse;
use crate::errors::TonlibError;
use tonlib_sys::{
    tonlib_client_json_create, tonlib_client_json_destroy, tonlib_client_json_receive, tonlib_client_json_send,
};
// Wrapper around ton client with support for TL data types

pub(crate) struct TLClientRaw {
    client_ptr: *mut ::std::os::raw::c_void,
    tag: String,
}

impl TLClientRaw {
    pub fn new(tag: String) -> TLClientRaw {
        unsafe {
            let client_ptr = tonlib_client_json_create();
            TLClientRaw { client_ptr, tag }
        }
    }

    pub fn get_tag(&self) -> &str { self.tag.as_str() }

    pub fn send(&self, req: &TLRequest, extra: &str) -> Result<(), TonlibError> {
        let c_str = req.to_c_str_json(extra)?;
        log::trace!("[{}] send: {c_str:?}", self.tag);
        unsafe { tonlib_client_json_send(self.client_ptr, c_str.as_ptr()) };
        Ok(())
    }

    pub fn receive(&self, timeout: f64) -> Result<Option<(TLResponse, Option<String>)>, TonlibError> {
        let c_str = unsafe { tonlib_client_json_receive(self.client_ptr, timeout) };
        if c_str.is_null() {
            return Ok(None);
        }
        unsafe { TLResponse::from_c_str_json(c_str).map(Some) }
    }

    pub fn set_log_verbosity_level(verbosity_level: u32) {
        unsafe { tonlib_sys::tonlib_client_set_verbosity_level(verbosity_level) }
    }
}

impl Drop for TLClientRaw {
    fn drop(&mut self) { unsafe { tonlib_client_json_destroy(self.client_ptr) } }
}

unsafe impl Send for TLClientRaw {}
unsafe impl Sync for TLClientRaw {}

#[cfg(test)]
mod tests {
    use crate::clients::tonlibjson::client_raw::tl_request::TLRequest;
    use crate::clients::tonlibjson::clients_impl::executor::TLClientRaw;

    #[test]
    fn set_log_verbosity_level_works() -> anyhow::Result<()> {
        let level = 1;
        TLClientRaw::set_log_verbosity_level(level);
        Ok(())
    }

    #[test]
    fn it_executes_functions() -> anyhow::Result<()> {
        let client = TLClientRaw::new("test".to_string());
        client.send(&TLRequest::GetLogVerbosityLevel {}, "test2")?;
        Ok(())
    }
}
