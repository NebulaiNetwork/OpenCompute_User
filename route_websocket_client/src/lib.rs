use futures_util::future::BoxFuture;
use futures_util::{SinkExt, StreamExt};
use public::DBG_ERR;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, Sender};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

type RouteCallback = Arc<dyn Fn(i16, String) -> BoxFuture<'static, ()> + Send + Sync>;
type RouteBigPayloadCallback = Arc<dyn Fn(i16, String, String) -> BoxFuture<'static, ()> + Send + Sync>;

#[derive(Clone)]
pub struct WsClient {
    inner: Arc<Mutex<WsClientInner>>,
}

struct WsClientInner {
    uid: String,
    url: String,
    routes: HashMap<String, Arc<dyn Fn(i16, String) -> BoxFuture<'static, ()> + Send + Sync>>,
    routes_big_payload: HashMap<String, Arc<dyn Fn(i16, String, String) -> BoxFuture<'static, ()> + Send + Sync>>,
    
    tx: Sender<String>,
}

#[derive(Serialize)]
struct WsRequest {
    t: String,
    r: String,
    p: String,
}

#[derive(Deserialize)]
struct WsResponse {
    c: i16,
    p: String,
    r: String,
}

impl WsClient {
    pub fn new(uid: String, url: String) -> Self {
        let (tx, _rx) = mpsc::channel::<String>(100);
        let inner = WsClientInner {
            uid,
            url,
            routes: HashMap::new(),
            routes_big_payload: HashMap::new(),
            tx,
        };
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub fn route_ws<F, Fut>(&self, api: &str, callback: F)
    where
        F: Fn(i16, String) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let cb: RouteCallback = Arc::new(move |code, payload| {
            let fut = callback(code, payload);
            Box::pin(fut) as BoxFuture<'static, ()>
        });

        let mut inner = self.inner.lock().unwrap();
        inner.routes.insert(api.to_string(), cb);
    }

    pub fn route_ws_big_payload<F, Fut>(&self, api: &str, callback: F)
    where
        F: Fn(i16, String, String) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let cb: RouteBigPayloadCallback = Arc::new(move |code, payload, big_payload| {
            let fut = callback(code, payload, big_payload);
            Box::pin(fut) as BoxFuture<'static, ()>
        });

        let mut inner = self.inner.lock().unwrap();
        inner.routes_big_payload.insert(api.to_string(), cb);
    }

    pub fn start_ws(&self) {
        let inner = self.inner.clone();
        let (msg_tx, mut msg_rx) = mpsc::channel::<String>(100);

        {
            let mut locked = inner.lock().unwrap();
            locked.tx = msg_tx.clone();
        }

        tokio::spawn(async move {
            loop {
                let url = {
                    let locked = inner.lock().unwrap();
                    locked.url.clone()
                };

                match connect_async(&url).await {
                    Ok((mut ws_stream, _)) => {
                        println!("WebSocket connected to {}", &url);

                        loop {
                            tokio::select! {
                                Some(Ok(msg)) = ws_stream.next() => {
                                    if let Message::Text(text) = msg {

                                        if text.len() < 4 {
                                            continue;
                                        }

                                        let hex_len_str = &text[..4];
                                        if let Ok(json_len) = u32::from_str_radix(hex_len_str, 16) {
                                            let json_end = 4 + json_len as usize;
                                            if text.len() < json_end {
                                                continue;
                                            }
                                            let json_str = &text[4..json_end];

                                            if let Ok(parsed) = serde_json::from_str::<WsResponse>(json_str) {
                                                let big_payload = &text[json_end..];

                                                if big_payload.len() > 0{
                                                    let cb_opt = {
                                                        let locked = inner.lock().unwrap();
                                                        locked.routes_big_payload.get(&parsed.r).cloned()
                                                    };

                                                    if let Some(cb) = cb_opt {
                                                        cb(parsed.c, parsed.p, big_payload.to_string()).await;
                                                    }
                                                }else{
                                                    let cb_opt = {
                                                        let locked = inner.lock().unwrap();
                                                        locked.routes.get(&parsed.r).cloned()
                                                    };

                                                    if let Some(cb) = cb_opt {
                                                        cb(parsed.c, parsed.p).await;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Some(msg) = msg_rx.recv() => {
                                    if let Err(e) = ws_stream.send(Message::Text(msg.clone().into())).await {
                                        eprintln!("Send error: {}", e);

                                        let _ = msg_tx.send(msg).await;
                                        break; 
                                    }
                                }
                                else => break,
                            }
                        }

                        DBG_ERR!("WebSocket disconnected, will attempt to reconnect...");
                    }
                    Err(e) => {
                        DBG_ERR!("Failed to connect to ", url, ": ", e);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        });
    }

    pub async fn send(&self, route: String, payload: String) {
        self.send_big_payload(route, payload, "".to_string()).await;
    }

    pub async fn send_big_payload(&self, route: String, payload: String, big_payload: String) {
        let (msg, tx) = {
            let locked: std::sync::MutexGuard<'_, WsClientInner> = self.inner.lock().unwrap();
            let req = WsRequest {
                t: locked.uid.clone(),
                r: route,
                p: payload,
            };
            let json_str = serde_json::to_string(&req).unwrap();

            let hex_len = {
                let mut s = format!("{:x}", json_str.len());
                if s.len() < 4 {
                    s = format!("{:0>4}", s);
                }
                s
            };

            let final_msg = hex_len + &json_str + &big_payload;

            (final_msg, locked.tx.clone())
        };

        let _ = tx.send(msg).await;
    }
}
