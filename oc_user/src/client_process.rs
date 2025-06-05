
use crate::{protocol::{BaseMsg, UserInitCodeResult}, thread_task_manager::{finish_task, TaskInfo}, DBG_ERR, DBG_LOG};
use public::{parse_json, decode};
// use thread_manager::{send_msg};

pub async fn user_hello(code: i16, payload: String){
	DBG_LOG!("code[", code, "] payload[", payload, "]");
}

pub async fn user_init(_code: i16, payload: String){
	// DBG_LOG!("code[", code, "] payload[", payload, "]");

	match parse_json::<BaseMsg>(&payload){
		Ok(mut base_msg) => {
			let msg_info = base_msg.get_msg();

			match parse_json::<UserInitCodeResult>(&msg_info.payload){
				Ok(init_code_result) =>{
					if init_code_result.succ{
						//send_msg::<bool>(config::WAIT_INIT, true);
					}else{
						DBG_ERR!("init code failed[", init_code_result.payload, "]");
					}
				},
				Err(e) =>{
					DBG_ERR!("parse base msg error:", e.to_string());
				}
			}
		},
		Err(e) => DBG_ERR!("parse base msg error:", e.to_string()),
	};
}

pub async fn user_run(_code: i16, payload: String, big_payload: String){
	// DBG_LOG!("code[", code, "] payload[", payload, "]");
	
	match parse_json::<BaseMsg>(&payload){
		Ok(base_msg) => {
			let big_payload = decode(&big_payload, base_msg.event_id);

			match parse_json::<TaskInfo>(&big_payload){
				Ok(task_info) => {
					finish_task(task_info);
				},
				Err(e) => DBG_ERR!("parse task info error:", e.to_string()),
			};
		},
		Err(e) => DBG_ERR!("parse task info error:", e.to_string()),
	};
}

pub async fn user_close(code: i16, payload: String){
	DBG_LOG!("code[", code, "] payload[", payload, "]");
}

