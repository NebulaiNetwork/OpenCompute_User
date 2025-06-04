

use crate::thread_ws_send::send_msg_to_ws_server;
use serde::{Deserialize, Serialize};
use public::{encode, decode, build_json, parse_json, rand_u64,  DBG_ERR};
use crate::{G_AUTH_CODE, INIT_CODE_EVNET};

#[derive(Deserialize, Serialize)]
pub struct BaseMsg {
    event_id: u64,

    pub payload: String,

    #[serde(skip)]
    msg_info: MsgInfo,
    #[serde(skip)]
    already_init: bool,
}

impl BaseMsg {
    pub fn new(event_id: u64, msg: MsgInfo) -> Self {
        let payload = encode(&msg, event_id);

        Self {
            event_id: event_id,
            payload: payload,
            already_init: false,
            msg_info: msg,
        }
    }

    pub fn get_msg(&mut self) -> MsgInfo {
        if self.already_init {
            self.msg_info.clone()
        } else {
            let decode_result = decode(&self.payload.clone(), self.event_id);

            self.msg_info = parse_json(&decode_result).unwrap();
            self.already_init = true;
            self.msg_info.clone()
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MsgInfo {
    pub operator_id: u64,
    pub payload: String,
    auth_code: u64,
}

impl MsgInfo {
    pub fn new(operator_id_: u64, payload: String) -> Self {
        Self {
            operator_id: operator_id_,
            payload: payload,
            auth_code: *(G_AUTH_CODE.lock().unwrap()),
        }
    }
}


#[derive(Serialize)]
struct UserInitCode{
    code:   String,
    num:    u32,
}

#[derive(Deserialize)]
pub struct UserInitCodeResult{
    pub succ    : bool,
    pub payload : String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunCodePayload {
    func: String,
    input: String,
    output: i16,
}




pub fn direct_send_error_msg(other_msg: String) {
    let error_msg_s = BaseMsg::new(0, MsgInfo::new(0, other_msg));

    let json_str = build_json(&error_msg_s).unwrap();
    
    send_msg_to_ws_server("error".to_string(), json_str);
}

pub fn send_msg_to_verifier(route: String, other_msg: String){
    let msg_s = BaseMsg::new(rand_u64(), MsgInfo::new(0, other_msg));

    let json_str = build_json(&msg_s).unwrap();

    send_msg_to_ws_server(route, json_str);
}

pub fn send_msg_to_verifier_by_event_id(event_id: u64, route: String, other_msg: String){
    let msg_s = BaseMsg::new(event_id, MsgInfo::new(0, other_msg));

    let json_str = build_json(&msg_s).unwrap();

    send_msg_to_ws_server(route, json_str);
}

pub fn send_msg_to_verifier_by_event_id_op_id(route: String, event_id: u64, op_id: u64, other_msg: String){
    let msg_s = BaseMsg::new(event_id, MsgInfo::new(op_id, other_msg));

    let json_str = build_json(&msg_s).unwrap();

    send_msg_to_ws_server(route, json_str);
}



pub fn user_hello(){
    send_msg_to_verifier("user/hello".to_string(), "".to_string());
}

pub fn user_init(code: String, max_workers: u32){
    let init_code_event_id = INIT_CODE_EVNET.lock().unwrap();

    let user_init_code = UserInitCode{
        code: code,
        num: max_workers,
    };

    match build_json(&user_init_code) {
        Ok(payload) => send_msg_to_verifier_by_event_id(*init_code_event_id, "user/init".to_string(), payload),
        Err(e) => {
            direct_send_error_msg(e.to_string());
            DBG_ERR!("build json error");
        },
    };
}

pub fn user_run(event_id: u64, op_id: u64, call_func: String, input: String, output: i16){
    let tmp_payload = RunCodePayload {
        func: call_func,
        input: input,
        output: output,
    };

    let run_code_payload = build_json(&tmp_payload).unwrap();
    send_msg_to_verifier_by_event_id_op_id(
        "user/run".to_string(),
        event_id,
        op_id,
        run_code_payload,
    );
}

pub fn user_close(event_id: u64){
    send_msg_to_verifier_by_event_id_op_id("user/close".to_string(), event_id, 0, "".to_string());
}