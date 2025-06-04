
use public::{DBG_LOG, sleep_ms};

use crate::protocol::{user_hello, user_init};

pub fn thread_keep_alive(code: String, max_worker: u32){

	DBG_LOG!("init code:", code);

	sleep_ms(1000);

	user_hello();

	sleep_ms(1000);

	user_init(code.clone(), max_worker);

	loop{
		sleep_ms(60000);
		user_hello();
	}
}
