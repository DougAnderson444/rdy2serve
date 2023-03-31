// C:\Users\douga\Documents2\code\RUST-projects\str0m-chat\js-libp2p-webrtc\node_modules\@libp2p\webrtc\dist\index.min.js
use wasm_bindgen::prelude::*;

// lifted from the `console_log` example
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn run() {
    log(&format!("Hello webRTC!"));
}
