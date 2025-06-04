pub fn add(a, b){
    a + b
}

pub fn matmul_row(row, b) {
    let b_cols = b[0].len();
    let b_rows = b.len();

    let result_row = [];

    for j in 0..b_cols {
        let sum = 0.0;
        for k in 0..b_rows {
            sum += row[k] * b[k][j];
        }
        result_row.push(sum);
    }

    result_row
}

pub async fn gpu_matmul_row(row, b) {
    let result = gpu_vec_matrix_multiply(row, b).await;
    result
}

pub async fn gpu_matrix_mul(a, b) {
    let result = gpu_matrix_multiply(a, b).await;
    result
}
