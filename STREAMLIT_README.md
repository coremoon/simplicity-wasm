# Simplicity WASM Compiler - Streamlit Edition

A browser-based Simplicity smart contract compiler built with Leptos/WASM and Streamlit. Compiles Simplicity code directly in the browser without server-side computation.

## Features

- **Pure Browser Execution**: All compilation runs in-browser via WebAssembly
- **No Backend Required**: No Flask, no subprocess, no external API calls
- **Interactive Dashboard**: Real-time compilation with Streamlit UI
- **CMR Output**: Generates Commitment Merkle Root hashes
- **Witness Support**: Optional witness data integration
- **Docker Ready**: Single-command deployment

## Project Structure

```
simplicity-wasm/
├── dist/                                 # Pre-built WASM artifacts
│   ├── simplicity-wasm-*.js       # JavaScript glue code
│   ├── simplicity-wasm-*.wasm  # WebAssembly binary
│   └── index.html                        # Static HTML
├── streamlit/                            # Streamlit application
│   ├── Dockerfile                        # Container definition
│   ├── app.py                            # Main application
│   ├── requirements.txt                  # Python dependencies
│   └── README.md                         # Streamlit documentation
├── contract/                             # Example Simplicity contracts
│   ├── simple.simf
│   ├── simple.wit
│   ├── lastwill.simf
│   └── lastwill.wit
├── src/                                  # Rust source code
│   ├── lib.rs                            # Main implementation
│   ├── simplicity_lib.rs                 # Simplicity-specific logic
│   └── wasm_api.rs                       # WASM API exports
├── Cargo.toml                            # Rust configuration
├── Trunk.toml                            # Leptos/WASM build config
└── [other files]
```

## Quick Start

### With Docker (Recommended)

```bash
# Build the image
docker build -t pywasm -f streamlit/Dockerfile --build-context dist=dist .

# Run the container
docker run -p 8081:8501 pywasm
```

Open http://localhost:8081 in your browser.

### Local Development

```bash
# Install dependencies
cd streamlit
pip install -r requirements.txt

# Run Streamlit
streamlit run app.py
```

The app will open at http://localhost:8501.

## Docker Build Context Explained

The build command uses `--build-context`:

```bash
docker build \
  -t pywasm \
  -f streamlit/Dockerfile \
  --build-context dist=dist \
  .
```

This means:
- `-f streamlit/Dockerfile`: Use the Dockerfile from the `streamlit/` directory
- `--build-context dist=dist`: Mount the `dist/` directory (at project root) as a build context
- The Dockerfile can then use `COPY dist ./dist` even though it's in a subdirectory

**Why this approach?**
- No need for the full Rust toolchain in Docker
- Only Python + pre-built WASM artifacts
- Faster builds (smaller Python container)

## Dockerfile Details

Located at `streamlit/Dockerfile`:

```dockerfile
FROM python:3.11-slim

WORKDIR /app

# Install minimal system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl && \
    rm -rf /var/lib/apt/lists/*

# Install Python dependencies
COPY streamlit/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy pre-built WASM artifacts (from --build-context)
COPY dist ./dist

# Copy Streamlit application
COPY streamlit/app.py ./app.py

# Create non-root user
RUN useradd -m -u 1000 app && chown -R app:app /app
USER app

# Expose Streamlit port
EXPOSE 8501

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl --fail http://localhost:8501/_stcore/health || exit 1

# Run Streamlit
CMD ["streamlit", "run", "app.py", "--server.port=8501", "--server.address=0.0.0.0"]
```

## How It Works

### 1. Loading WASM Assets

The Streamlit app loads WASM and JavaScript from `dist/`:

```python
def load_wasm_files():
    """Load WASM and JS files from dist/"""
    dist_path = Path(__file__).parent.parent / "dist"
    
    # Read WASM as base64
    with open(wasm_files[0], 'rb') as f:
        wasm_base64 = base64.b64encode(f.read()).decode('utf-8')
    
    # Read JavaScript glue code
    with open(js_files[0], 'r') as f:
        js_code = f.read()
```

### 2. Browser-Side Execution

Assets are encoded as data URLs and injected into an HTML component:

```javascript
// Data URL encoding
const wasmUrl = `data:application/wasm;base64,${wasmBase64}`;
const jsUrl = `data:application/javascript;base64,${jsBase64}`;

// Dynamic ES module import
const jsModule = await import(jsUrl);
const init = jsModule.default || jsModule.init;
const wasm = await init({ module_or_path: wasmUrl });

// Call compiler
const result = jsModule.compile_simplicity(code);
```

### 3. Result Display

Results are displayed in two places:

**HTML Component** (immediate):
- Compilation status (✅ Success / ❌ Error)
- CMR hash
- Full JSON output

**Streamlit UI** (below component):
- **CMR Tab**: Commitment Merkle Root
- **Base64 Tab**: Encoded source code
- **Witness Tab**: JSON witness data
- **Metadata Tab**: Code statistics, timestamp

## Usage

### Compiling Code

1. Paste your Simplicity code in the **SimplicityHL Code** text area
2. (Optional) Add witness data as JSON in the **Witness Data** field
3. Click **Compile with WASM**
4. View results in the component and tabs below

### Example Input

```
(begin
  (define (sum a b) (+ a b))
  (sum 5 3)
)
```

### Example Output

```json
{
  "success": true,
  "cmr": "2c4e1d...",
  "base64": "KGJlZ2luIChkZWZpbmUgKHN1bSBhIGIpICgrIGEgYikpIChzdW0gNSAzKSkK",
  "witness": null,
  "metadata": {
    "code_size": 45,
    "timestamp": "2025-12-11T14:46:31Z",
    "mode": "production"
  }
}
```

## Port Mapping

```bash
# Default: Host port 8081 → Container port 8501
docker run -p 8081:8501 pywasm

# Custom: Host port 9000 → Container port 8501
docker run -p 9000:8501 pywasm
```

## Configuration

### Streamlit Settings

Configured in `streamlit/Dockerfile`:

```dockerfile
CMD ["streamlit", "run", "app.py", \
  "--server.port=8501", \
  "--server.address=0.0.0.0"]
```

To use custom Streamlit config, mount it:

```bash
docker run -p 8081:8501 \
  -v ~/.streamlit/config.toml:/app/.streamlit/config.toml \
  pywasm
```

## Building WASM Artifacts

Before running Docker, ensure `dist/` contains compiled WASM:

```bash
# Install Trunk (Leptos WASM bundler)
cargo install trunk

# Build WASM (release-optimized)
trunk build --release

# Output: dist/ with WASM artifacts
ls -la dist/
# simplicity-wasm-2624d5f96c908f11.js
# simplicity-wasm-2624d5f96c908f11_bg.wasm
# index.html
```

## Development Workflow

### 1. Modify Rust Code

```bash
nano src/lib.rs
```

### 2. Build WASM

```bash
trunk build --release
```

### 3. Test Locally

```bash
cd streamlit
streamlit run app.py
```

### 4. Build Docker Image

```bash
docker build -t pywasm -f streamlit/Dockerfile --build-context dist=dist .
```

### 5. Run Container

```bash
docker run -p 8081:8501 pywasm
```

## Health Checks

The container includes an automatic health check:

```bash
docker run -p 8081:8501 pywasm

# View health status
docker ps --format "table {{.ID}}\t{{.Status}}"
# Look for (healthy) in Status column

# Manual health check
curl http://localhost:8501/_stcore/health
```

## Troubleshooting

### "dist/ directory not found"

**Cause**: Pre-built WASM artifacts are missing

**Solution**: Build WASM first:
```bash
trunk build --release
docker build -t pywasm -f streamlit/Dockerfile --build-context dist=dist .
```

### "Port already in use"

**Cause**: Port 8081 (or specified port) is already in use

**Solution**: Use a different port:
```bash
docker run -p 8082:8501 pywasm
```

### "Cannot import module"

**Cause**: JavaScript glue code encoding issue

**Solution**: Verify the dist/ directory:
```bash
ls -la dist/*.js
file dist/*.js
```

### "WASM instantiation failed"

**Cause**: WASM imports or initialization failed

**Solution**: Check browser console (F12) for JavaScript errors

### Streamlit shows duplicate titles

**Solution**: This is handled by CSS in `streamlit/app.py`:
```python
st.markdown("""
<style>
    h1, h2, h3 { display: none !important; }
</style>
""", unsafe_allow_html=True)
```

## Performance Characteristics

- **Compilation Time**: < 100ms (browser-dependent)
- **Memory Usage**: ~30-50MB Python + WASM runtime
- **WASM Binary Size**: ~5-10MB (depending on Leptos features)
- **Build Time**: ~2-3 minutes (Rust compilation)
- **Container Startup**: ~5-10 seconds

## Security Considerations

- **Client-Side Execution**: All compilation happens in the browser
- **No Data Exposure**: User code never leaves the browser
- **WASM Sandboxing**: WebAssembly runtime is isolated
- **HTTPS Recommended**: Use HTTPS in production for data integrity
- **No External Calls**: Pure client-side processing

## Limitations

- **Code Size**: Limited by browser memory (typically 100MB+)
- **Compilation Speed**: Slower than native (browser VM overhead)
- **Debugging**: Limited to browser console (F12)
- **Offline Support**: Works offline after initial load (due to caching)

## Technical Details

### Leptos WASM Initialization Pattern

```javascript
import init, * as bindings from './glue-code.js';

const wasm = await init({
  module_or_path: './compiled.wasm'
});
```

This pattern is preserved when using data URLs.

### Data URL Encoding

WASM and JavaScript are encoded as base64 data URLs:

```
data:application/wasm;base64,UklGRiw...
data:application/javascript;base64,aW1wb3J0...
```

This eliminates the need for a file server or additional HTTP requests.

### Browser Limitations

- WASM execution is sandboxed to the browser runtime
- No filesystem access
- No network requests (except via explicit JavaScript)
- All computation is strictly client-side

## Contributing

To contribute improvements:

1. Build WASM locally: `trunk build --release`
2. Test in Streamlit: `streamlit run streamlit/app.py`
3. Test in Docker: `docker build -t pywasm -f streamlit/Dockerfile --build-context dist=dist .`
4. Submit changes with working `dist/` artifacts

## References

- [Leptos Documentation](https://leptos.dev)
- [Streamlit Documentation](https://docs.streamlit.io)
- [WebAssembly Specs](https://webassembly.org)
- [Trunk Documentation](https://trunkrs.dev)
- [Simplicity Smart Contracts](https://github.com/ElementsProject/simplicity)
- [Docker Build Contexts](https://docs.docker.com/build/building/context/)

## Support

For issues or questions:

1. Check browser console (F12) for JavaScript errors
2. Verify `dist/` contains compiled WASM artifacts
3. Ensure Docker has sufficient memory (2GB+ recommended)
4. View Streamlit logs: `docker logs <container_id>`
5. Check Streamlit logs on local run: `streamlit run app.py --logger.level=debug`
