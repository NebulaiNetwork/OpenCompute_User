# OpenCompute_User

The base code for the OpenCompute user module.

This project uses a Python virtual environment to build and run a Rust-Python hybrid module via [Maturin](https://github.com/PyO3/maturin). Ensure you have all prerequisites installed before proceeding.

## Prerequisites

- Python 3.10.12+
- [Maturin](https://pyo3.rs/maturin/)
- Rust (recommended version: 1.86.0 or higher)
- `make`

## Setup and Build

Follow these steps to build and run the project:

### 1. Activate the Python Virtual Environment

If you haven't created one yet:

```bash
cd OpenCompute_User
python -m venv venv
```
Load virtual env
```bash
# Unix/macOS:
source ./venv/bin/activate
# Windows:
venv\Scripts\activate
```
Then install maturin:  
```bash
pip install maturin
```

### 2. Build the Project
```bash
make
```

### 3. Run the Test Script
```bash
python test.py
```
