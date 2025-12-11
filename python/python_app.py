"""
Simplicity WASM Compiler - Flask Backend
Serves the pre-built Leptos WASM frontend from dist/
Provides HTTP API endpoints for compilation
"""

from flask import Flask, render_template, request, jsonify, send_from_directory
from flask_cors import CORS
import json
import base64
import logging
import os
from datetime import datetime

app = Flask(__name__, 
    static_folder=os.path.join(os.path.dirname(__file__), 'dist'),
    static_url_path='')
CORS(app)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class CompileResult:
    """Result object for compilation"""
    def __init__(self, cmr=None, error=None, witness_data=None):
        self.cmr = cmr
        self.error = error
        self.witness_data = witness_data
    
    def to_dict(self):
        result = {
            "cmr": self.cmr,
            "error": self.error
        }
        if self.witness_data:
            result["witness_data"] = self.witness_data
        return result


def validate_json(data: str) -> tuple:
    """Validate JSON format"""
    try:
        json.loads(data)
        return True, ""
    except json.JSONDecodeError as e:
        return False, f"Invalid JSON: {str(e)}"


def compile_simplicity(code: str) -> CompileResult:
    """
    Compile Simplicity code without witness
    
    Currently returns placeholder CMR.
    In production, this would call the actual Simplicity compiler library.
    """
    if not code or not code.strip():
        return CompileResult(error="Code is empty")
    
    logger.info(f"Compiling code: {code[:50]}...")
    
    # Placeholder CMR (would come from real compiler)
    # In production: from simplicityhl import compile_program
    cmr = "c40a10263f7436b4160acbef1c36fba4be4d95df181a968afeab5eac247adff7"
    
    return CompileResult(cmr=cmr)


def compile_with_witness(code: str, witness_data: str) -> CompileResult:
    """
    Compile Simplicity code with JSON witness data
    
    Args:
        code: Simplicity source code
        witness_data: JSON-formatted witness variables
    
    Returns:
        CompileResult with CMR and witness data
    """
    if not code or not code.strip():
        return CompileResult(error="Code is empty")
    
    if not witness_data or not witness_data.strip():
        return CompileResult(error="Witness data is empty")
    
    # Validate JSON
    is_valid, error_msg = validate_json(witness_data)
    if not is_valid:
        return CompileResult(error=error_msg)
    
    logger.info(f"Compiling with witness: {code[:50]}...")
    logger.info(f"Witness variables count: {len(json.loads(witness_data))}")
    
    # Parse witness JSON
    try:
        witness_json = json.loads(witness_data)
    except json.JSONDecodeError as e:
        return CompileResult(error=f"Witness parse error: {str(e)}")
    
    # TODO: Call actual Simplicity compiler with witness
    # In production: from simplicityhl import compile_with_witness
    cmr = "c40a10263f7436b4160acbef1c36fba4be4d95df181a968afeab5eac247adff7"
    
    return CompileResult(
        cmr=cmr,
        witness_data=witness_json
    )


# ===== STATIC FILE SERVING =====

@app.route('/')
def index():
    """Serve index.html (Leptos WASM app entry point)"""
    dist_path = os.path.join(os.path.dirname(__file__), 'dist/index.html')
    if os.path.exists(dist_path):
        return send_from_directory(os.path.join(os.path.dirname(__file__), 'dist'), 'index.html')
    return "Error: dist/index.html not found", 404


@app.route('/<path:filename>')
def serve_static(filename):
    """Serve all static assets from dist/ (JS, CSS, WASM, etc)"""
    dist_dir = os.path.join(os.path.dirname(__file__), '../dist')
    try:
        return send_from_directory(dist_dir, filename)
    except FileNotFoundError:
        # For client-side routing, serve index.html for unknown routes
        return send_from_directory(dist_dir, 'index.html')


# ===== API ENDPOINTS =====

@app.route('/api/compile', methods=['POST'])
def api_compile():
    """
    API endpoint: POST /api/compile
    
    Compiles Simplicity code with optional witness data.
    
    JSON Request:
    {
        "code": "mod param { } fn main() { }",
        "witness_data": "{\\"VAR\\": ...}"  (optional)
    }
    
    JSON Response:
    {
        "cmr": "hash...",
        "error": null,
        "witness_data": {...}  (if provided)
    }
    """
    try:
        data = request.get_json()
        
        if not data:
            return jsonify({"cmr": None, "error": "No JSON data provided"}), 400
        
        code = data.get('code', '').strip()
        witness_data = data.get('witness_data', '').strip() if data.get('witness_data') else None
        
        logger.info(f"API /compile: code_length={len(code)}, with_witness={bool(witness_data)}")
        
        # Compile based on whether witness is provided
        if witness_data:
            result = compile_with_witness(code, witness_data)
        else:
            result = compile_simplicity(code)
        
        response = result.to_dict()
        logger.info(f"Compilation result: cmr={result.cmr is not None}, error={result.error}")
        
        return jsonify(response), 200
    
    except Exception as e:
        logger.error(f"Compilation error: {str(e)}", exc_info=True)
        return jsonify({"cmr": None, "error": f"Server error: {str(e)}"}), 500


@app.route('/api/encode-base64', methods=['POST'])
def api_encode_base64():
    """
    API endpoint: POST /api/encode-base64
    
    Encodes text to Base64.
    
    JSON Request:
    {
        "data": "text to encode"
    }
    
    JSON Response:
    {
        "encoded": "base64string"
    }
    """
    try:
        data = request.get_json()
        if not data or 'data' not in data:
            return jsonify({"error": "No data provided"}), 400
        
        text = data['data']
        encoded = base64.b64encode(text.encode('utf-8')).decode('utf-8')
        
        return jsonify({"encoded": encoded}), 200
    
    except Exception as e:
        logger.error(f"Base64 encoding error: {str(e)}")
        return jsonify({"error": str(e)}), 500


@app.route('/api/health', methods=['GET'])
def api_health():
    """
    Health check endpoint
    
    Returns:
    {
        "status": "healthy",
        "timestamp": "2025-12-11T12:00:00Z"
    }
    """
    return jsonify({
        "status": "healthy",
        "timestamp": datetime.utcnow().isoformat() + "Z"
    }), 200


# ===== ERROR HANDLERS =====

@app.errorhandler(404)
def not_found(error):
    """Serve index.html for 404 (SPA routing)"""
    dist_dir = os.path.join(os.path.dirname(__file__), 'dist')
    return send_from_directory(dist_dir, 'index.html'), 200


@app.errorhandler(500)
def server_error(error):
    """Handle 500 errors"""
    logger.error(f"Server error: {str(error)}")
    return jsonify({"error": "Internal server error"}), 500


if __name__ == '__main__':
    logger.info("Starting Simplicity Compiler Flask Backend")
    logger.info(f"Serving static files from: {os.path.join(os.path.dirname(__file__), 'dist')}")
    
    # Development: debug mode with reloader
    app.run(
        host='0.0.0.0',
        port=5000,
        debug=True,
        use_reloader=True
    )
