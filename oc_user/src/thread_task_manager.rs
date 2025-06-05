use crate::{config, IntoPyObject, PyObject, Python, PyAny};
use public::{DBG_ERR, DBG_LOG};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thread_manager::{recv_msg, send_msg};

#[derive(Deserialize, Serialize)]
pub struct TaskInfo {
    operator_id: u64,
    error: String,
    result: String,
}

pub fn finish_task(task_info: TaskInfo) {
    send_msg::<TaskInfo>(config::THREAD_TASK_MANAGER, task_info);
}

pub fn query_task_list_result(
    py: Python<'_>,
    task_ids: &[u64],
    expect_result: HashMap<u64, String>,
) -> Vec<PyObject> {
    let mut waiting: HashSet<u64> = task_ids.iter().copied().collect();
    let mut results: HashMap<u64, String> = HashMap::new();

    while !waiting.is_empty() {
        if let Some(task_info) = recv_msg::<TaskInfo>(config::THREAD_TASK_MANAGER) {
            let task_id = task_info.operator_id;

            // DBG_LOG!("recv task[", task_id, "] result[", task_info.result, "]");

            if task_info.error.len() != 0{
                DBG_ERR!("task id[", task_id, "] run error[", task_info.error, "]")
            }else{
                if waiting.contains(&task_id) {
                    results.insert(task_id, task_info.result);
                    waiting.remove(&task_id);
                } else {
                    DBG_LOG!("task {} not in waiting list, skip", task_id);
                }
            }
        } else {
            DBG_ERR!("task parser error.");
        }
    }

    task_ids
    .iter()
    .map(|id| {
        let result_str = results.remove(id).unwrap_or_else(|| "".into());
        let type_str = expect_result.get(id).map(|s| s.as_str()).unwrap_or("");

        let obj: pyo3::Bound<'_, PyAny> = match type_str {
            "i32" => {
                let val = result_str.parse::<i32>().unwrap_or_default();
                val.into_pyobject(py).unwrap().into_any()
            }
            "f32" => {
                let val = result_str.parse::<f32>().unwrap_or_default();
                val.into_pyobject(py).unwrap().into_any()
            }
            "String" => {
                result_str.into_pyobject(py).unwrap().into_any()
            }
            "Vec<i32>" => {
                let val: Vec<i32> = serde_json::from_str(&result_str).unwrap_or_default();
                val.into_pyobject(py).unwrap().into_any()
            }
            "Vec<f32>" => {
                let val: Vec<f32> = serde_json::from_str(&result_str).unwrap_or_default();
                val.into_pyobject(py).unwrap().into_any()
            }
            "Vec<Vec<i32>>" => {
                let val: Vec<Vec<i32>> = serde_json::from_str(&result_str).unwrap_or_default();
                val.into_pyobject(py).unwrap().into_any()
            }
            "Vec<Vec<f32>>" => {
                let val: Vec<Vec<f32>> = serde_json::from_str(&result_str).unwrap_or_default();
                val.into_pyobject(py).unwrap().into_any()
            }
            _ => py.None().into_pyobject(py).unwrap().into_any(),
        };

        obj.into()
    })
    .collect()
}
