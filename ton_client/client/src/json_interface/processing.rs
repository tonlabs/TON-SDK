/*
 * Copyright 2018-2020 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 *
 */

use crate::client::{ClientContext};
use crate::error::ClientResult;
use crate::processing::{ParamsOfProcessMessage, ProcessingEvent, ProcessingResponseType, ResultOfProcessMessage, ParamsOfSendMessage, ResultOfSendMessage, ParamsOfWaitForTransaction};
use std::sync::Arc;
use super::request::Request;


/// Creates message, sends it to the network and monitors its processing.
/// 
/// Creates ABI-compatible message,
/// sends it to the network and monitors for the result transaction.
/// Decodes the output messages's bodies.
/// 
/// If ABI supports `pragma expire` then
/// SDK implements retries in case of unsuccessful message delivery within the expiration
/// timeout. SDK recreates the message, sends it and processes it again. 
/// The appropriate events are controlled by `send_events` flag 
/// and logged into the supplied callback function.
/// The retry configuration parameters are defined in config:
/// <add correct config params here>
/// pub const DEFAULT_EXPIRATION_RETRIES_LIMIT: i8 = 3; - max number of retries
/// pub const DEFAULT_EXPIRATION_TIMEOUT: u32 = 40000;  - message expiration timeout in ms.
/// pub const DEFAULT_....expiration_timeout_grow_factor... = 1.5 - factor that increases the expiration timeout for each retry
/// 
/// If ABI DOES NOT support `pragma expire`
/// then if no transacrion is found within the network timeout (see config parameter ), exits with error.
/// 
/// `send_events` enables/disables intermediate events, useful for logging, such as "will send",
/// "did send", "will fetch next block", etc.
#[api_function]
pub(crate) async fn process_message(
    context: Arc<ClientContext>,
    params: ParamsOfProcessMessage,
    request: std::sync::Arc<Request>,
) -> ClientResult<ResultOfProcessMessage> {
    let callback = move |event: ProcessingEvent| {
        request.response(event, ProcessingResponseType::ProcessingEvent as u32);
        futures::future::ready(())
    };
    crate::processing::process_message(context, params, callback).await
}


/// Sends message to the network
/// 
/// Sends message to the network and returns the last generated shard block of the destination account
/// before the messafe was sent. It will be required later for message processing.
#[api_function]
pub(crate) async fn send_message(
    context: Arc<ClientContext>,
    params: ParamsOfSendMessage,
    callback: std::sync::Arc<Request>,
) -> ClientResult<ResultOfSendMessage> {
    let callback = move |result: ProcessingEvent| {
        callback.response(result, ProcessingResponseType::ProcessingEvent as u32);
        futures::future::ready(())
    };

    crate::processing::send_message::send_message(context, params, callback).await
}

/// Performs monitoring of the network for for the result transaction
/// of the external inbound message processing.
/// 
/// `send_events` enables intermediate events, such as `new shard block received` event will be triggered 
/// each time.
///
/// Note that presence of the `abi` parameter is critical for ABI
/// compliant contracts. Message processing uses drastically
/// different strategy for processing message with an ABI expire
/// pragma.
///
/// When the ABI header `expire` is present, the processing uses
/// `message expiration` strategy:
/// - The maximum block gen time is set to
///   `message_expiration_time + transaction_wait_timeout`.
/// - When maximum block gen time is reached the processing will
///   be finished with `MessageExpired` error.
///
/// When the ABI header `expire` isn't present or `abi` parameter
/// isn't specified, the processing uses `transaction waiting`
/// strategy:
/// - The maximum block gen time is set to
///   `now() + transaction_wait_timeout`.
/// 
/// - If maximum block gen time is reached and no result transaction is found 
/// the processing will exit with an error.
#[api_function]
pub(crate) async fn wait_for_transaction(
    context: Arc<ClientContext>,
    params: ParamsOfWaitForTransaction,
    callback: std::sync::Arc<Request>,
) -> ClientResult<ResultOfProcessMessage> {
    let callback = move |result: ProcessingEvent| {
        callback.response(result, ProcessingResponseType::ProcessingEvent as u32);
        futures::future::ready(())
    };
    crate::processing::wait_for_transaction(context, params, callback).await
}
