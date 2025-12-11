# Simplicity Compiler - Streamlit Dashboard

A **Streamlit dashboard** that integrates the pre-built Leptos WASM compiler to provide an interactive UI for compiling Simplicity smart contracts with witness data support.

## Overview

Instead of serving the full Leptos SPA, this approach:
- Uses **pre-built WASM** assets (`dist/`)
- Builds a dashboard UI with **Streamlit** (pure Python)
- Provides the same compilation functionality with a different frontend
- Demonstrates reusing WASM across different presentation layers

### Key Differences from Web Frontend

| Aspect | Leptos Web | Streamlit |
|--------|-----------|-----------|
| **Framework** | Rust/Leptos | Python/Streamlit |
| **UI Library** | Leptos components | Streamlit widgets |
| **Styling** | CSS | Streamlit theming + custom CSS |
| **Interactivity** | Real-time reactive | Button/widget callbacks |
| **Deployment** | Nginx/static files | Streamlit server |
| **Development** | Rust compilation | Python only |

## Quick Start

### Prerequisites

- Python 3.11+
- Pre-built `dist/` directory from Leptos project
- Streamlit

### Installation

```bash
# Install dependencies
pip install -r streamlit_requirements.txt

# Place pre-built WASM in dist/
# (copy from: leptos-project/dist/)

# Run dashboard
streamlit run streamlit_app.py
```

Access at: **http://localhost:8501**

## Project Structure

```
streamlit/
├── streamlit_app.py           # Dashboard application
├── streamlit_requirements.txt  # Dependencies
├── Dockerfile_streamlit        # Docker configuration
└── dist/                       # Pre-built WASM (symlink or copy)
    ├── *.wasm
    ├── *.js
    └── ...
```

## Features

### 1. Code Compilation

- **Text Input**: Paste Simplicity code directly
- **File Upload**: Upload `.simf` files
- **Real-time Compilation**: Click "Compile" button

### 2. Witness Data

- **Text Input**: Paste JSON witness data
- **File Upload**: Upload `.wit` files
- **Validation**: Validates JSON structure

### 3. Results Display

Three tabs showing compilation results:

- **CMR**: Commitment Merkle Root hash
- **Base64**: Encoded code
- **Metadata**: Compilation metadata and statistics

### 4. Examples

Pre-loaded examples:
- Minimal contract
- Contract with witness variables
- Lastwill inheritance contract

## Usage Guide

### Compile Without Witness

1. Enter code in left panel
2. Click "🔨 Compile"
3. View CMR and Base64 results

### Compile With Witness

1. Enter code in left panel
2. Enter witness JSON in right panel
3. Click "🔨 Compile"
4. Results include witness metadata

### Upload Files

**Code files** (.simf):
```bash
# Select file in uploader
# Code auto-loads into text area
```

**Witness files** (.wit):
```bash
# Select file in uploader
# Witness data auto-loads
```

## Integration with WASM

### Current Implementation

The dashboard currently shows:
- WASM file information in sidebar
- Available assets in `dist/`
- Placeholder compilation (TODO: actual WASM call)

### Calling WASM from Streamlit

To actually use the WASM compiler, you have three options:

#### Option 1: Subprocess (Node.js)

```python
import subprocess
import json

def compile_simplicity(code: str) -> dict:
    """Call WASM via Node.js"""
    result = subprocess.run([
        'node', '-e',
        f"""
        const wasm = require('./dist/index.js');
        const result = wasm.compile_simplicity({json.dumps(code)});
        console.log(JSON.stringify(result));
        """
    ], capture_output=True, text=True)
    
    return json.loads(result.stdout)
```

#### Option 2: Python WASM Binding

```python
# Using wasmtime or wasmer
from wasmtime import Module, Instance

def compile_simplicity(code: str) -> dict:
    module = Module(open('dist/compiler.wasm', 'rb').read())
    instance = Instance(module)
    # Call WASM function
    result = instance.exports.compile_simplicity(code)
    return json.loads(result)
```

#### Option 3: Backend HTTP API (Recommended)

Keep Flask/FastAPI backend running and call via HTTP:

```python
import requests

def compile_simplicity(code: str) -> dict:
    """Call WASM via backend API"""
    response = requests.post(
        'http://localhost:5000/api/compile',
        json={"code": code}
    )
    return response.json()
```

## Docker Deployment

### Build

```bash
# From project root
docker build -f Dockerfile_streamlit -t simplicity-streamlit .
```

### Run

```bash
docker run -p 8501:8501 simplicity-streamlit
```

Access at: http://localhost:8501

### Docker Compose

```yaml
services:
  streamlit:
    build:
      context: .
      dockerfile: Dockerfile_streamlit
    ports:
      - "8501:8501"
    volumes:
      - ./dist:/app/dist
```

## Configuration

### Streamlit Settings

Edit `.streamlit/config.toml`:

```toml
[theme]
primaryColor = "#1f77b4"
backgroundColor = "#ffffff"
secondaryBackgroundColor = "#f0f2f6"
textColor = "#262730"

[server]
port = 8501
headless = true
```

### Custom CSS

Modify the `<style>` section in `streamlit_app.py`:

```python
st.markdown("""
<style>
    .my-custom-class {
        color: #1f77b4;
        font-weight: bold;
    }
</style>
""", unsafe_allow_html=True)
```

## Performance

- **Initial Load**: <2s
- **Compilation**: Depends on WASM implementation
- **Memory**: ~100MB for Streamlit + WASM
- **Concurrent Users**: Single-threaded (use with proxy for scaling)

## Advantages vs Web Frontend

✅ **No Frontend Build**: Use pre-built WASM  
✅ **Faster Development**: Pure Python, no Rust compilation  
✅ **Easy Customization**: Modify UI with simple Python code  
✅ **Integration**: Call any Python library  
✅ **Different UX**: Dashboard style vs SPA  

## Limitations

❌ **Single-threaded**: Can't handle concurrent requests well  
❌ **Not suitable for high-traffic**: Use web frontend + Nginx instead  
❌ **File Size**: WASM assets still required in `dist/`  

## Updating WASM

When you update the Leptos frontend:

1. Build with `trunk build --release`
2. Copy `dist/` to streamlit directory
3. Restart Streamlit: `streamlit run streamlit_app.py`

```bash
# From Leptos project
trunk build --release

# Copy to Streamlit
cp -r dist/ ../streamlit-project/dist/

# Restart
cd ../streamlit-project
streamlit run streamlit_app.py
```

## Development

### Local Development

```bash
# Install dev dependencies
pip install -r streamlit_requirements.txt

# Run with auto-reload
streamlit run streamlit_app.py

# Visit http://localhost:8501
```

### Hot Reload

Streamlit automatically reloads on Python file changes. For WASM updates:

```bash
# Update WASM in dist/
cp -r new-dist/* dist/

# Refresh browser or restart Streamlit
```

## Troubleshooting

### dist/ directory not found

```
Error: dist/ directory not found
```

**Solution**: Build Leptos frontend
```bash
cd leptos-project
trunk build --release
cp -r dist/ ../streamlit-project/
```

### Port already in use

```bash
# Use different port
streamlit run streamlit_app.py --server.port=8502
```

### WASM compilation fails

Implement one of the three options above to actually call the WASM compiler.

## Examples

### Example 1: Simple Contract

```simplicity
mod param {
}

fn main() {
}
```

### Example 2: With Witness

```simplicity
fn main() {
    let amount: u64 = witness::AMOUNT;
    amount
}
```

**Witness**:
```json
{
    "AMOUNT": {
        "value": "1000",
        "type": "u64"
    }
}
```

### Example 3: Lastwill Contract

See "Examples" tab in dashboard.

## Next Steps

1. **Implement WASM Calling**: Use one of the three methods above
2. **Add Database**: Store compilation history
3. **User Authentication**: Track who compiled what
4. **Advanced UI**: Add visualization/graphs
5. **Real-time Updates**: Use Streamlit session state for caching

## Resources

- [Streamlit Documentation](https://docs.streamlit.io)
- [Streamlit API Reference](https://docs.streamlit.io/library/api-reference)
- [WASM in Python](https://wasmer.io/posts/python-wasm)

---

**Status**: ✅ Functional Dashboard (WASM integration TODO)

**Last Updated**: 2025-12-11
