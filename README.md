This crate allows communication (message passing) of code in WASM instances, invoked by 3NWeb platform(s).

## Message passing, version 1 (`wasm_mp1`)

Module `wasm_mp1` does message passing between a WASM instance and its embedding in accordance with version 1 of api.

Embedder provides imports a callback `_3nweb_mp1_send_out_msg` in `env` namespace. WASM code writes message bytes into its own memory and calls `_3nweb_mp1_send_out_msg` with memory pointer and message length. Embedder must copy message content during this call, cause message exchange memory area is recycled and there are no guarantees about its content afterwards.

WASM exports functions `_3nweb_mp1_get_buffer` and `_3nweb_mp1_accept_msg`. These are used by embedder to send messages to WASM instance. Embedder sends message to WASM instance with following steps:
 - asks to allocate memory for message in WASM with `_3nweb_mp1_get_buffer` call,
 - copies message into provided memory area,
 - calls `_3nweb_mp1_accept_msg` to let WASM instance know about the message.

All three steps must be done without letting WASM instance to do anything else. This ensures that memory allocated in the first step won't be re-purposed, messing up everything.


## License
LGPL-3.0 or greater version(s).

