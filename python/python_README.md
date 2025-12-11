# Simplicity Compiler - Python/Flask Backend

Alternative backend implementation serving the same pre-built Leptos WASM frontend with a Python/Flask REST API.

## Quick Start

### Development

```bash
# Install dependencies
pip install -r requirements.txt

# Run Flask development server
python app.py

# Access at http://localhost:5000
```

### Production with Docker

```bash
# Build image
docker build -f Dockerfile -t simplicity-python .

# Run container
docker run -d -p 5000:80 --name simplicity-python simplicity-python

# Access at http://localhost:5000
```

## Architecture

### Static Assets

The Flask backend serves pre-built WASM assets from the `dist/` directory:
- `index.html` - Entry point for Leptos SPA
- `*.js` - JavaScript bundles
- `*.wasm` - WebAssembly modules
- `*.css` - Stylesheets

### API Endpoints

#### GET `/api/health`
Health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-12-11T12:00:00Z"
}
```

#### POST `/api/compile`
Compile Simplicity code with optional witness data.

**Request:**
```json
{
  "code": "mod param {} fn main() {}",
  "witness_data": "{\"VAR\": {...}}"
}
```

**Response:**
```json
{
  "cmr": "c40a10263f7436b4160acbef1c36fba4be4d95df181a968afeab5eac247adff7",
  "error": null,
  "witness_data": {...}
}
```

#### POST `/api/encode-base64`
Encode text to Base64.

**Request:**
```json
{
  "data": "text to encode"
}
```

**Response:**
```json
{
  "encoded": "dGV4dCB0byBlbmNvZGU="
}
```

## Project Structure

```
python/
├── app.py              # Flask application and API endpoints
├── requirements.txt    # Python dependencies
├── Dockerfile          # Docker configuration
└── README.md          # This file
```

## Features

- **SPA Routing**: All unknown routes serve `index.html` for client-side routing
- **CORS Support**: Enables cross-origin requests
- **API Endpoints**: RESTful compilation API
- **Static Asset Serving**: Serves pre-built WASM frontend
- **Logging**: Comprehensive request/response logging
- **Health Checks**: Built-in health check endpoint for container orchestration
- **Error Handling**: Graceful error responses with proper HTTP status codes

## Development

### Adding New API Endpoints

```python
@app.route('/api/new-endpoint', methods=['POST'])
def api_new_endpoint():
    """Document your endpoint here"""
    try:
        data = request.get_json()
        # Process request
        return jsonify(result), 200
    except Exception as e:
        logger.error(f"Error: {str(e)}")
        return jsonify({"error": str(e)}), 500
```

### Logging

The application logs all compilation requests and results:

```python
logger.info(f"API /compile: code_length={len(code)}, with_witness={bool(witness_data)}")
```

View logs with:
```bash
docker logs -f simplicity-python
```

## Dependencies

- **Flask 3.0.0** - Web framework
- **Flask-CORS 4.0.0** - Cross-origin resource sharing
- **Werkzeug 3.0.1** - WSGI utility library

## Integration with WASM Frontend

The pre-built Leptos WASM assets should be in the `dist/` directory:

```
simplicity-wasm/
├── dist/              # Built Leptos frontend
│   ├── index.html
│   ├── *.js
│   └── *.wasm
├── python/
│   ├── app.py
│   ├── requirements.txt
│   └── Dockerfile
└── ...
```

Build the frontend with:
```bash
cd ..
trunk build --release
```

Then run the Python backend which will serve the pre-built assets.


## Status

✅ **Fully Functional** - Serves pre-built WASM frontend with REST API

Last updated: 2025-12-11
