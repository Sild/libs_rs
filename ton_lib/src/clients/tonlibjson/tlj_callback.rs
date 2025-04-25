// use std::sync::Arc;
// use std::time::Duration;
//
// use crate::clients::tonlibjson::client_raw::tl_request::TLRequest;
// use crate::clients::tonlibjson::client_raw::tl_response::TLResponse;
// use crate::errors::TonlibError;
//
// /// The callback methods invoked by TonConnection
// pub trait TLJCallback: Send + Sync {
//     /// called **before** invoking tonlib.
//     fn before_exec(&self, tag: &str, req_id: u32, req: &TLRequest) {}
//
//     /// Method `on_invoke_result` gets called in two scenarios:
//     ///
//     /// - **after** receiving invoke result from tonlib and **before** sending result to the caller.
//     /// - **after** failed attempt to invoke tonlib (this situation might occur only because of
//     ///   serialization error).
//     fn on_result(&self, tag: &str, request_id: u32, method: &str, duration: &Duration, result: &Result<TLResponse, TonlibError>) {}
//
//     /// Method `on_cancelled_invoke` gets called when attempt to send an invoke result is failed
//     ///
//     /// Typically this happens when the corresponding future (async fn invoke_on_connection) is cancelled
//     fn on_cancel(&self, tag: &str, request_id: u32, method: &str, duration: &Duration) {}
//
//     /// Method `on_notification` gets called upon receiving valid notification from tonlib.
//     ///
//     /// A tonlib notification doesn't have corresponding request and thus no `request_id`.
//     fn on_notify(&self, tag: &str, notification: &TonNotification) {}
//
//     /// Method `on_ton_result_parse_error` gets called upon receiving message from tonlib
//     /// that couldn't be parsed.
//     ///
//     /// Reception of `on_ton_result_parse_error` means that not all tonlib message get parsed
//     /// and undefined behaviour is very likely.
//     fn on_parser_response_error(&self, tag: &str, req_extra: Option<&str>, result: &TLResponse) {}
//
//     /// Method `on_idle` gets called when polling tonlib returns `None`.
//     fn on_idle(&self, tag: &str) {}
//
//     /// Method `on_connection_loop_start` gets called when new connection loop starts
//     fn on_conn_loop_start(&self, tag: &str) {}
//
//     /// Method `on_connection_loop_exit` gets called when new connection loop stops and connection is dropped
//     fn on_conn_loop_end(&self, tag: &str) {}
// }
//
// /// An implementation of TonConnectionCallback that does nothing
// pub struct TLJCallbackNoop {}
// impl TLJCallback for TLJCallbackNoop {}
//
// /// An implementation of TonConnectionCallback that does default logging
// pub struct TLJCallbackTrace {}
// impl TLJCallback for TLJCallbackTrace {
//     fn on_result(
//         &self,
//         tag: &str,
//         request_id: u32,
//         method: &str,
//         duration: &Duration,
//         result: &Result<TonResult, TonClientError>,
//     ) {
//         match result {
//             Ok(r) => {
//                 log::trace!(
//                     "[{}] Invoke successful, request_id: {}, method: {}, elapsed: {:?}: {}",
//                     tag,
//                     request_id,
//                     method,
//                     duration,
//                     r.to_string()
//                 );
//             }
//             Err(e) => {
//                 log::warn!(
//                     "[{}] Invocation error: request_id: {:?}, method: {}, elapsed: {:?}: {}",
//                     tag,
//                     request_id,
//                     method,
//                     duration,
//                     e
//                 );
//             }
//         }
//     }
//
//     fn on_cancel(&self, tag: &str, request_id: u32, method: &str, duration: &Duration) {
//         log::warn!(
//             "[{}] Error sending invoke result, receiver already closed. method: {} request_id: {}, elapsed: {:?}",
//             tag,
//             method,
//             request_id,
//             duration,
//        );
//     }
//
//     fn on_notify(&self, tag: &str, notification: &TonNotification) {
//         log::trace!("[{}] Sending notification: {:?}", tag, notification);
//     }
//
//     fn on_parser_response_error(
//         &self,
//         tag: &str,
//         request_extra: Option<&str>,
//         result: &TonResult,
//     ) {
//         log::error!(
//             "[{}] Error parsing result: request_extra: {:?}: {}",
//             tag,
//             request_extra,
//             result
//         );
//     }
//
//     fn on_conn_loop_start(&self, tag: &str) {
//         log::info!("[{}] Starting event loop", tag);
//     }
//
//     fn on_conn_loop_end(&self, tag: &str) {
//         log::info!("[{}] Exiting event loop", tag);
//     }
// }
