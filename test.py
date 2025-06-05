from oc_user import OcUser
import oc_user
import numpy as np

import time

oc = OcUser("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJkYXRhIjoiZHVudHkiLCJleHAiOjE3NTE2OTY1ODZ9.YN0riVjbOdQfogHx34jWiaYAwE509TQwkbQYNNUGIjU", "./test.rs", 1)

def test_add_batch():

    time.sleep(5)

    # muti call run_code，collect task_id
    task_ids = []
    for _ in range(5):
        task_id = oc.run_code("add", (1, 2), "i32")
        task_ids.append(task_id)

    print("Task ID：", task_ids)

    # Wait for all tasks to finish
    results = oc.wait_task(task_ids)

    print("Results:", results)

#test_add_batch()


def naive_matmul(a, b):
    size = len(a)
    b_T = list(zip(*b))
    result = []
    for i in range(size):
        row = []
        for j in range(size):
            val = sum(a[i][k] * b_T[j][k] for k in range(size))
            row.append(val)
        result.append(row)
    return result


def test_matmul_parallel():
    time.sleep(5)

    size = 1000
    a = np.random.rand(size, size).tolist()
    b = np.random.rand(size, size).tolist()
    

    start_parallel = time.time()

    # Parallel dispatch for each row
    task_ids = []
    for i in range(size):
        task_id = oc.run_code("gpu_matmul_row", (a[i], b), "Vec<f32>")
        task_ids.append(task_id)

    # Wait for all sub-tasks
    results = oc.wait_task(task_ids)

    end_parallel = time.time()

    # Each result is a row, combine into matrix
    result_matrix = results


    parallel_time_ms = (end_parallel - start_parallel) * 1000
    print(f"\nParallel computation time: {parallel_time_ms:.2f} ms")
    
    start_naive = time.time()
    expected_matrix = naive_matmul(a, b)
    end_naive = time.time()
    naive_time_ms = (end_naive - start_naive) * 1000
    print(f"Pure Python computation time: {naive_time_ms:.2f} ms")

    # Compare total sums, Make a rough judgment on whether it is consistent
    result_sum = sum(sum(row) for row in result_matrix)
    expected_sum = sum(sum(row) for row in expected_matrix)

    print(f"\nMatrix total sum comparison:")
    print(f"Parallel computation sum: {result_sum:.6f}")
    print(f"Manual computation sum: {expected_sum:.6f}")
    print(f"Difference (absolute): {abs(result_sum - expected_sum):.6f}")

#test_matmul_parallel()


def test_matrix():
    time.sleep(5)

    size = 800
    a = np.random.rand(size, size).tolist()
    b = np.random.rand(size, size).tolist()

    start_parallel = time.time()

    # Parallel dispatch
    task_ids = []
    task_id = oc.run_code("gpu_matrix_mul", (a, b), "Vec<Vec<f32>>")
    task_ids.append(task_id)

    # Wait for all sub-tasks
    results = oc.wait_task(task_ids)

    end_parallel = time.time()

    # Each result is a row, combine into matrix
    result_matrix = results

    parallel_time_ms = (end_parallel - start_parallel) * 1000
    print(f"\nGPU computation time: {parallel_time_ms:.2f} ms")

    
    start_naive = time.time()
    expected_matrix = naive_matmul(a, b)
    end_naive = time.time()
    naive_time_ms = (end_naive - start_naive) * 1000
    print(f"Pure Python computation time: {naive_time_ms:.2f} ms")
    
    '''
    # np calc are fast, but here is a simple test of the feasibility of remote execution.
    a_np = np.array(a)
    b_np = np.array(b)

    start_np = time.time()
    np_result = np.dot(a_np, b_np)
    end_np = time.time()
    np_time_ms = (end_np - start_np) * 1000
    print(f"NumPy computation time: {np_time_ms:.2f} ms")
    '''

    #print("result_matrix =", result_matrix)

    if len(result_matrix) == 1 and isinstance(result_matrix[0], list):
        result_matrix = result_matrix[0]

    # Compare total sums
    result_sum = sum(sum(row) for row in result_matrix)
    expected_sum = sum(sum(row) for row in expected_matrix)


    print(f"\nMatrix total sum comparison:")
    print(f"Parallel computation sum: {result_sum:.6f}")
    print(f"Manual computation sum: {expected_sum:.6f}")
    print(f"Difference (absolute): {abs(result_sum - expected_sum):.6f}")

test_matrix()