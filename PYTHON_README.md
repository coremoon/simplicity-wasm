# Reusing Pre-Built Leptos WASM in Python Projects

This guide explains how to integrate a pre-built Leptos WebAssembly frontend into a Python web framework (Flask, FastAPI, Django, etc.).

## Overview

Instead of rebuilding your WASM frontend every time, you can:
1. Build the Leptos WASM application once with `trunk build --release`
2. Copy the `dist/` directory to your Python project
3. Serve it as static assets from your Python web server

This approach provides:
- **Separation of concerns**: Frontend (Rust/WASM) and backend (Python) are independent
- **Faster iteration**: Change backend without rebuilding WASM
- **Flexible deployment**: Choose any Python web framework
- **Code reuse**: Same WASM frontend with different backends

## Project Structure

```
my-project/
├── dist/                    # Pre-built Leptos WASM (from trunk build)
│   ├── index.html
│   ├── *.js
│   └── *.wasm
├── python/
│   ├── app.py              # Your Python application
│   ├── requirements.txt
│   └── Dockerfile
└── README.md
```

## Step 1: Build the Leptos Frontend

From your Leptos project directory:

```bash
trunk build --release
```

This generates:
- `dist/index.html` - SPA entry point
- `dist/*.js` - JavaScript bundles
- `dist/*.wasm` - WebAssembly modules
- Other assets (CSS, images, etc.)

## Step 2: Copy to Your Python Project

```bash
# Copy the built dist/ to your Python project
cp -r dist/ your-python-project/dist/

# Or organize it differently
cp -r dist/ your-python-project/static/dist/
```

## Step 3: Serve Static Assets

### Flask Example

```python
from flask import Flask, send_from_directory
import os

app = Flask(__name__, 
    static_folder=os.path.join(os.path.dirname(__file__), 'dist'),
    static_url_path='')

@app.route('/')
def index():
    """Serve index.html"""
    return send_from_directory('dist', 'index.html')

@app.route('/<path:filename>')
def serve_static(filename):
    """Serve static assets (JS, WASM, CSS, etc)"""
    try:
        return send_from_directory('dist', filename)
    except FileNotFoundError:
        # SPA routing: serve index.html for unknown routes
        return send_from_directory('dist', 'index.html')

@app.errorhandler(404)
def not_found(error):
    """Fallback for 404 - serve index.html for client-side routing"""
    return send_from_directory('dist', 'index.html'), 200
```

### FastAPI Example

```python
from fastapi import FastAPI
from fastapi.staticfiles import StaticFiles
from fastapi.responses import FileResponse
import os

app = FastAPI()

# Mount static files
app.mount("/", StaticFiles(directory="dist", html=True), name="dist")

@app.get("/")
async def index():
    """Serve index.html"""
    return FileResponse("dist/index.html")
```

### Django Example

In `settings.py`:
```python
STATIC_URL = '/static/'
STATICFILES_DIRS = [
    os.path.join(BASE_DIR, 'dist'),
]
```

In `urls.py`:
```python
from django.views.generic import TemplateView
from django.urls import path

urlpatterns = [
    path('', TemplateView.as_view(template_name='index.html')),
]
```

## Step 4: Configure SPA Routing

The Leptos frontend uses **client-side routing**. All unknown routes should serve `index.html`:

### Flask
```python
@app.errorhandler(404)
def not_found(error):
    return send_from_directory('dist', 'index.html'), 200
```

### FastAPI
```python
from fastapi.staticfiles import StaticFiles

app.mount("/", StaticFiles(directory="dist", html=True))
```

### Django
```python
# In urls.py - catch-all at the end
path('<path:path>', TemplateView.as_view(template_name='index.html')),
```

## Step 5: API Communication

The Leptos frontend communicates with your Python backend via **HTTP APIs**.

### Define Backend Endpoints

**Python (Flask example):**
```python
@app.route('/api/compile', methods=['POST'])
def api_compile():
    data = request.get_json()
    code = data.get('code')
    # Process code
    return jsonify({"cmr": "...", "error": None})

@app.route('/api/health', methods=['GET'])
def health():
    return jsonify({"status": "healthy"})
```

### Call from Frontend

The Leptos frontend can call these endpoints:

```rust
// In Leptos component
let response = fetch(
    "/api/compile",
    FetchOptions::post()
        .body(serde_json::json!({"code": code_value}).to_string())
)
.await
.unwrap()
.json::<CompileResult>()
.await
.unwrap();
```

## Step 6: Docker Deployment

### Dockerfile

```dockerfile
FROM python:3.11-slim

WORKDIR /app

# Copy Python requirements
COPY requirements.txt .
RUN pip install -r requirements.txt

# Copy pre-built WASM frontend
COPY dist ./dist

# Copy Python app
COPY app.py .

EXPOSE 5000
CMD ["python", "app.py"]
```

### Build and Run

```bash
# Build
docker build -t my-app .

# Run
docker run -p 5000:5000 my-app
```

## Step 7: Caching Strategy

Configure proper caching for WASM assets:

### Flask
```python
from flask import send_from_directory

@app.route('/<path:filename>')
def serve_static(filename):
    response = send_from_directory('dist', filename)
    
    # Cache WASM/JS for 1 year (versioned by hash in filename)
    if filename.endswith(('.wasm', '.js')):
        response.cache_control.max_age = 31536000  # 1 year
        response.cache_control.public = True
    
    # Don't cache HTML
    elif filename.endswith('.html'):
        response.cache_control.max_age = 0
        response.cache_control.no_cache = True
    
    return response
```

### Nginx (if using reverse proxy)
```nginx
location ~* \.(wasm|js)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}

location ~* \.html$ {
    expires -1;
    add_header Cache-Control "no-cache, no-store, must-revalidate";
}
```

## Common Issues

### Issue: 404 on WASM Files

**Problem**: Browser can't load `.wasm` files
**Solution**: Ensure `dist/` is correctly mounted as static folder

```python
# ✅ CORRECT
app = Flask(__name__, static_folder='dist', static_url_path='')

# ❌ WRONG
app = Flask(__name__)  # dist/ not served
```

### Issue: JavaScript Module Errors

**Problem**: JS modules can't find WASM
**Solution**: WASM files must be in same directory as JS

```
dist/
├── index.html
├── app.js          ← Looks for .wasm here
└── app_bg.wasm     ← Must be in same dir
```

### Issue: Routing Issues (404 on navigation)

**Problem**: Client-side routing broken, page reloads show 404
**Solution**: Configure fallback to `index.html`

```python
# All unknown routes → index.html
@app.errorhandler(404)
def not_found(e):
    return send_from_directory('dist', 'index.html')
```

## Performance Optimization

### Bundle Size
- Leptos with WASM: ~2.5MB (uncompressed)
- Gzipped: ~700KB
- Brotli: ~600KB

### Load Time
- Initial HTML: <50ms
- WASM download: 100-500ms (varies by network)
- WASM initialization: <100ms

### Optimization Tips
1. **Gzip/Brotli compression** on server
2. **HTTP/2 Push** for critical assets
3. **Service Workers** for offline support
4. **Lazy loading** for non-critical components

## Updating the Frontend

When you update your Leptos code:

1. Rebuild WASM:
   ```bash
   cd leptos-project
   trunk build --release
   ```

2. Copy new `dist/`:
   ```bash
   cp -r dist/ ../python-project/dist/
   ```

3. Redeploy Python app (no code changes needed!)

## Example: Complete Setup

### Full Flask Application

```python
from flask import Flask, send_from_directory, jsonify, request
from flask_cors import CORS
import os

app = Flask(__name__, 
    static_folder=os.path.join(os.path.dirname(__file__), 'dist'),
    static_url_path='')
CORS(app)

# Serve frontend
@app.route('/')
def index():
    return send_from_directory('dist', 'index.html')

@app.route('/<path:filename>')
def static_files(filename):
    try:
        return send_from_directory('dist', filename)
    except FileNotFoundError:
        return send_from_directory('dist', 'index.html')

# Backend API
@app.route('/api/health', methods=['GET'])
def health():
    return jsonify({"status": "healthy"})

@app.route('/api/compile', methods=['POST'])
def compile_code():
    data = request.get_json()
    code = data.get('code')
    # Process...
    return jsonify({"cmr": "...", "error": None})

@app.errorhandler(404)
def not_found(e):
    return send_from_directory('dist', 'index.html')

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000, debug=False)
```

## Deployment Checklist

- [ ] Leptos built with `trunk build --release`
- [ ] `dist/` directory contains all assets
- [ ] Python app has correct `static_folder` path
- [ ] SPA routing configured (404 → index.html)
- [ ] API endpoints match frontend expectations
- [ ] CORS enabled if needed
- [ ] Caching headers configured
- [ ] Dockerfile copies both `dist/` and Python code
- [ ] Health check endpoint works
- [ ] WASM files load without 404 errors

## Summary

| Aspect | Benefit |
|--------|---------|
| **Separation** | Frontend and backend developed independently |
| **Framework Choice** | Use any Python framework (Flask, FastAPI, Django, etc.) |
| **Reusability** | Same WASM with multiple backends |
| **Performance** | Compiled WASM runs at near-native speed |
| **Deployment** | Simple Docker container with static assets |
| **Caching** | WASM cached aggressively (versioned by hash) |

---

**Last Updated**: 2025-12-11
