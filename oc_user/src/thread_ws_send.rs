
use route_websocket_client::{WsClient};
use crate::{config, DBG_ERR, client_process};

use thread_manager::{send_msg, recv_msg};

use tokio::runtime::{Builder};
use serde::{Deserialize, Serialize};


// pub static TOKIO: Lazy<Runtime> = Lazy::new(|| {
//     tokio::runtime::Builder::new_multi_thread()
//         .enable_all()
//         .build()
//         .unwrap()
// });

#[derive(Debug, Deserialize, Serialize)]
struct WsClientMsg{
    route:  String,
    payload:  String,
    big_payload: String,
}

pub fn send_msg_to_ws_server(route: String, payload: String, big_payload: String){
    let tmp = WsClientMsg{
        route: route,
        payload: payload,
        big_payload: big_payload,
    };

    send_msg::<WsClientMsg>(config::THREAD_WS_SEND, tmp);
}

pub fn thread_ws_send(token: String, ip: String){

    let rt = Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    rt.block_on(async move {

        let ws = WsClient::new(token, ip);

        ws.route_ws("user/hello", client_process::user_hello);
        ws.route_ws("user/init", client_process::user_init);
        ws.route_ws_big_payload("user/run", client_process::user_run);
        ws.route_ws("user/close", client_process::user_close);

        ws.start_ws();

        loop {
            if let Some(msg) = recv_msg::<WsClientMsg>(config::THREAD_WS_SEND) {

                // DBG_LOG!("send route[", msg.route, "] payload size[", msg.payload.len(), "]");
                
                ws.send_big_payload(msg.route, msg.payload, msg.big_payload).await;
            } else {
                DBG_ERR!("thread_ws_send error");
                break;
            }
        }

    });

    //hang this thread to wait close sig
    std::thread::park();
}