
use crate::{DBG_LOG, DBG_ERR, protocol::BaseMsg, protocol::UserInitCodeResult, thread_task_manager::finish_task};
use public::{parse_json};
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

pub async fn user_run(_code: i16, payload: String){
	// DBG_LOG!("code[", code, "] payload[", payload, "]");
	
	match parse_json::<BaseMsg>(&payload){
		Ok(mut base_msg) => {
			let msg_info = base_msg.get_msg();

			finish_task(msg_info);
		},
		Err(e) => DBG_ERR!("parse base msg error:", e.to_string()),
	};
}

pub async fn user_close(code: i16, payload: String){
	DBG_LOG!("code[", code, "] payload[", payload, "]");
}

