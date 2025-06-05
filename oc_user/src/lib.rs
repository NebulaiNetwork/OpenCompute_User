use pyo3::exceptions::PyRuntimeError;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyAnyMethods, PyList, PyListMethods, PyTuple, PyTupleMethods};
use serde_json::Value;
use std::collections::HashMap;

use public::{define_global, rand_u64, read_tiny_file, DBG_ERR, DBG_LOG};

use thread_manager::ThreadManager;

mod client_process;
mod config;
mod protocol;
mod thread_keep_alive;
mod thread_task_manager;
mod thread_ws_send;

define_global!(USER_TOKEN, String, String::new());
define_global!(G_AUTH_CODE, u64, 0x24420251131_u64);
define_global!(INIT_CODE_EVNET, u64, 0x0_u64);

#[pyclass]
struct OcUser {
    pub task_array: Vec<u64>,
    pub run_code_event: u64,
    pub op_id: u64,
    pub expect_results: HashMap<u64, String>,
}

#[pymethods]
impl OcUser {
    #[new]
    pub fn new(token_: String, rune_file_path: String, max_worker: u32) -> PyResult<Self> {
        let mut run_code_id = INIT_CODE_EVNET.lock().unwrap();
        *run_code_id = rand_u64();
        let run_code_event = *run_code_id;

        let mut token = USER_TOKEN.lock().unwrap();
        *token = token_.clone();

        let code = match read_tiny_file(rune_file_path) {
            Ok(content) => content,
            Err(e) => return Err(PyRuntimeError::new_err(format!("File read error: {}", e))),
        };

        let task_list: Vec<Box<dyn FnOnce() + Send>> = vec![
            Box::new(move || thread_keep_alive::thread_keep_alive(code, max_worker)),
            Box::new(|| {
                thread_ws_send::thread_ws_send(token_, config::WS_SERVER_URL.to_string().clone())
            }),
        ];

        ThreadManager::new(task_list);

        let user = Self {
            task_array: Vec::new(),
            run_code_event: run_code_event,
            op_id: 0,
            expect_results: HashMap::new(),
        };

        Ok(user)
    }

    pub fn run_code(
        &mut self,
        call_func: String,
        args: &Bound<'_, PyTuple>,
        output: String,
    ) -> PyResult<u64> {
        //let start_time = now_time_ms();

        match normalize_input(args) {
            Ok(args_string) => {

				//let end_time = now_time_ms();
				//DBG_LOG!("normalize_input use[", end_time - start_time, "]");

                let expect_output_type = match output.as_str() {
                    "i32" => 1_i16,
                    "f32" => 2_i16,
                    "String" => 3_i16,
                    "Vec<i32>" => 4_i16,
                    "Vec<f32>" => 5_i16,
                    "Vec<Vec<i32>>" => 6_i16,
                    "Vec<Vec<f32>>" => 7_i16,
                    _ => {
                        return Err(PyTypeError::new_err(format!(
                            "{:?}",
                            "no support this output type"
                        )))
                    }
                };

				//let start_time = now_time_ms();

                protocol::user_run(
                    self.run_code_event,
                    self.op_id,
                    call_func,
                    args_string,
                    expect_output_type,
                );

				//let end_time = now_time_ms();
				//DBG_LOG!("user_run use[", end_time - start_time, "]");

                self.task_array.push(self.op_id);
                self.expect_results.insert(self.op_id, output);
                self.op_id += 1;
                Ok(self.op_id - 1)
            }
            Err(e) => Err(PyTypeError::new_err(format!("{:?}", e.to_string()))),
        }
    }

    // pub fn run_code(&self, call_func: String, input: String, output: String) -> u64{
    // 	0_u64
    // }

    pub fn wait_task(
        &self,
        py: Python<'_>,
        py_list: &Bound<'_, PyList>,
    ) -> PyResult<Vec<PyObject>> {
        let wait_task_list = py_list.extract::<Vec<u64>>().expect("Expected list of u64");
        let ret_list = thread_task_manager::query_task_list_result(
            py,
            &wait_task_list,
            self.expect_results.clone(),
        );
        Ok(ret_list)
    }

    fn close(&self) {
        protocol::user_close(self.run_code_event);
    }
}

impl Drop for OcUser {
    fn drop(&mut self) {
        self.close();
    }
}

fn convert_any(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if let Ok(v) = obj.extract::<i32>() {
        Ok(Value::from(v))
    } else if let Ok(v) = obj.extract::<f32>() {
        Ok(Value::from(v))
    } else if let Ok(v) = obj.extract::<Vec<f32>>() {
        Ok(Value::from(v))
    } else if let Ok(v) = obj.extract::<Vec<Vec<f32>>>() {
        Ok(Value::from(v))
    } else if let Ok(seq) = obj.downcast::<PyList>() {
        let mut vec = Vec::new();
        for item in seq.iter() {
            vec.push(convert_any(&item)?);
        }
        Ok(Value::from(vec))
    } else if let Ok(seq) = obj.downcast::<PyTuple>() {
        let mut vec = Vec::new();
        for item in seq.iter() {
            vec.push(convert_any(&item)?);
        }
        Ok(Value::from(vec))
    } else {
        Err(PyTypeError::new_err(format!(
            "Unsupported type: {:?}",
            obj.get_type().name()?
        )))
    }
}

// #[pyfunction]
fn normalize_input(args: &Bound<'_, PyTuple>) -> PyResult<String> {

	// DBG_LOG!("args:", args);

    let mut list = Vec::new();
    for item in args.iter() {
        list.push(convert_any(&item)?);
    }

	// DBG_LOG!("list:", list);

    let value = Value::from(list);

	// DBG_LOG!("value:", value);

    match serde_json::to_string(&value) {
        Ok(val) => Ok(val),
        Err(e) => Err(PyTypeError::new_err(format!("err: {:?}", e.to_string()))),
    }
}

// #[pyfunction]
// fn test(py: Python<'_>, type_i: u32) -> PyObject {
//     match type_i {
//         0 => {
//             let val = 123u64;
//             val.into_py(py)
//         }
//         1 => {
//             let val = "hello".to_string();
//             val.into_py(py)
//         }
//         2 => {
//             let val = vec![1u64, 2, 3];
//             val.into_py(py)
//         }
//         3 => {
//             let val = vec![vec![1u64, 2], vec![3, 4]];
//             val.into_py(py)
//         }
//         _ => py.None(),
//     }
// }

#[pymodule]
fn oc_user(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(test, m)?)?;

    // m.add_function(wrap_pyfunction!(format_args, m)?)?;
    // m.add_function(wrap_pyfunction!(normalize_input, m)?)?;
    m.add_class::<OcUser>()?;
    Ok(())
}
