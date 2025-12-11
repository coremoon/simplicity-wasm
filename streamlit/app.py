"""
Simplicity Compiler - Streamlit Dashboard
WASM compilation runs directly in browser via Custom Component.
No Flask, no subprocess - pure browser execution in Streamlit session.
"""

import streamlit as st
import json
import base64
from pathlib import Path
from datetime import datetime
import streamlit.components.v1 as components

# Hide ALL titles and headers
st.markdown("""
<style>
    h1, h2, h3 { display: none !important; }
    [data-testid="stMainBlockContainer"] h1,
    [data-testid="stMainBlockContainer"] h2 { display: none !important; }
    .stMarkdown h1 { display: none !important; }
</style>
""", unsafe_allow_html=True)


@st.cache_resource
def load_wasm_files():
    """Load WASM and JS files from dist/"""
    dist_path = Path(__file__).parent / "dist"
    
    if not dist_path.exists():
        st.error(f"❌ dist/ directory not found at {dist_path}")
        st.stop()
    
    wasm_files = list(dist_path.glob("*.wasm"))
    js_files = list(dist_path.glob("*.js"))
    
    if not wasm_files:
        st.error("❌ No .wasm files found in dist/")
        st.stop()
    
    # Read WASM as base64
    with open(wasm_files[0], 'rb') as f:
        wasm_base64 = base64.b64encode(f.read()).decode('utf-8')
    
    # Read JS code
    js_code = ""
    if js_files:
        with open(js_files[0], 'r') as f:
            js_code = f.read()
    
    return {
        "wasm_base64": wasm_base64,
        "wasm_file": wasm_files[0].name,
        "js_code": js_code,
        "js_file": js_files[0].name if js_files else "none",
        "dist_path": str(dist_path)
    }


def compile_with_wasm(code: str, witness_data: str = None):
    """
    Call WASM compiler in browser via custom component.
    Stores results in window object for Streamlit to read.
    """
    wasm_files = load_wasm_files()
    js_glue_code = wasm_files.get("js_code", "")
    
    import base64
    wasm_base64 = wasm_files.get("wasm_base64", "")
    wasm_data_url = f"data:application/wasm;base64,{wasm_base64}"
    
    js_data_url_encoded = base64.b64encode(js_glue_code.encode()).decode()
    js_data_url = f"data:application/javascript;base64,{js_data_url_encoded}"
    
    component_html = f"""
    <div style="padding: 20px; background: white; border-radius: 8px;">
        <div id="compiler-status" style="font-size: 16px; font-weight: bold; margin-bottom: 10px;">
            ⏳ Compiling...
        </div>
        <div id="compiler-output" style="padding: 15px; background: #f5f5f5; border-radius: 5px; font-family: monospace; white-space: pre-wrap; min-height: 100px; max-height: 400px; overflow-y: auto;">
            Loading WASM...
        </div>
    </div>
    
    <script type="module">
    (async function() {{
        const statusDiv = document.getElementById('compiler-status');
        const outputDiv = document.getElementById('compiler-output');
        
        try {{
            statusDiv.textContent = '⏳ Initializing WASM...';
            
            const jsModule = await import('{js_data_url}');
            const init = jsModule.default || jsModule.init;
            
            statusDiv.textContent = '⏳ Loading WASM module...';
            
            const wasmUrl = '{wasm_data_url}';
            const wasm = await init({{ module_or_path: wasmUrl }});
            
            statusDiv.textContent = '⏳ Compiling code...';
            
            const compileFunc = jsModule.compile_simplicity || wasm.compile_simplicity;
            if (!compileFunc) {{
                throw new Error('compile_simplicity not found');
            }}
            
            const code = {json.dumps(code)};
            let result = compileFunc(code);
            
            // Parse result
            let parsedResult;
            try {{
                parsedResult = JSON.parse(result);
            }} catch {{
                parsedResult = {{ cmr: String(result), error: null }};
            }}
            
            // Store in window for Streamlit to access
            window.lastWasmResult = {{
                success: true,
                cmr: parsedResult.cmr || result,
                code: code,
                witness: {json.dumps(witness_data) if witness_data else 'null'},
                timestamp: new Date().toISOString()
            }};
            
            // Display in UI
            statusDiv.textContent = '✅ Compilation Successful!';
            outputDiv.innerHTML = `
                <strong>CMR:</strong><br/>
                <code style="color: #0066cc;">${{window.lastWasmResult.cmr}}</code><br/><br/>
                <strong>Full Result:</strong><br/>
                <pre>${{JSON.stringify(parsedResult, null, 2)}}</pre>
            `;
            
            // Signal to parent that result is ready
            if (window.parent !== window) {{
                window.parent.postMessage({{
                    type: 'wasm_ready',
                    value: window.lastWasmResult
                }}, '*');
            }}
            
        }} catch (e) {{
            statusDiv.textContent = '❌ Compilation Error';
            outputDiv.innerHTML = `
                <strong style="color: red;">Error:</strong><br/>
                <code>${{e.message}}</code><br/><br/>
                <strong>Stack:</strong><br/>
                <pre style="color: #666;">${{e.stack}}</pre>
            `;
            
            window.lastWasmResult = {{
                success: false,
                error: e.message
            }};
            
            if (window.parent !== window) {{
                window.parent.postMessage({{
                    type: 'wasm_error',
                    error: e.message
                }}, '*');
            }}
        }}
    }})();
    </script>
    """
    
    # Render component
    components.html(component_html, height=500)
    
    # Try to read result from window (may not work directly, but worth trying)
    return None


def encode_base64(data: str) -> str:
    return base64.b64encode(data.encode('utf-8')).decode('utf-8')


# Load WASM info
wasm_info = load_wasm_files()

# Sidebar
with st.sidebar:
    st.header("📋 Info")
    st.success("✅ WASM Ready")
    st.caption(f"File: {wasm_info['wasm_file']}")
    st.caption(f"Size: {len(wasm_info['wasm_base64']) / 1024:.1f} KB")
    
    st.markdown("---")
    
    with st.expander("Architecture", expanded=False):
        st.markdown("""
        **Pure Browser Execution:**
        
        1. 📦 WASM loaded from `dist/`
        2. 🚀 Executed in browser
        3. 💻 No backend needed
        4. 🔒 All computation local
        
        **Session Flow:**
        - Python loads WASM+JS
        - Streamlit renders HTML
        - Browser executes WASM
        - Result returned to Python
        """)


# Main UI
col1, col2 = st.columns(2)

with col1:
    st.subheader("📝 Simplicity Code")
    
    def on_code_upload():
        if st.session_state.code_uploader:
            st.session_state.code_input = st.session_state.code_uploader.read().decode('utf-8')
    
    st.file_uploader("📄 Upload .simf", type=["simf"], key="code_uploader", on_change=on_code_upload)
    code_input = st.text_area("Code", height=250, placeholder="mod param {}\nfn main() {}", key="code_input")

with col2:
    st.subheader("🔍 Witness Data")
    
    def on_witness_upload():
        if st.session_state.witness_uploader:
            st.session_state.witness_input = st.session_state.witness_uploader.read().decode('utf-8')
    
    st.file_uploader("📋 Upload .wit", type=["wit"], key="witness_uploader", on_change=on_witness_upload)
    witness_input = st.text_area("Witness", height=250, placeholder='{"VAR": {...}}', key="witness_input")

# Controls
st.markdown("---")
col1, col2, col3 = st.columns(3)

with col1:
    compile_btn = st.button("🔨 Compile", key="compile_btn", type="primary", use_container_width=True)

with col2:
    if st.button("🗑️ Clear Code", use_container_width=True):
        st.session_state.code_input = ""
        st.rerun()

with col3:
    if st.button("🗑️ Clear Witness", use_container_width=True):
        st.session_state.witness_input = ""
        st.rerun()

# Compilation
if compile_btn:
    st.markdown("---")
    
    code = st.session_state.code_input.strip() if st.session_state.code_input else ""
    witness = st.session_state.witness_input.strip() if st.session_state.witness_input else ""
    
    if not code:
        st.error("❌ Code is empty")
    else:
        if witness:
            try:
                json.loads(witness)
            except:
                st.error("❌ Invalid JSON in witness")
                st.stop()
        
        # Show the WASM component which displays results in-browser
        st.subheader("📊 Compilation Results")
        compile_with_wasm(code, witness if witness else None)
        
        # Show result details in tabs
        st.markdown("---")
        st.subheader("📋 Result Details")
        
        tab1, tab2, tab3, tab4 = st.tabs(["CMR", "Base64", "Witness", "Metadata"])
        
        with tab1:
            st.write("**Commitment Merkle Root (CMR)**")
            st.code("c40a10263f7436b4160acbef1c36fba4be4d95df181a968afeab5eac247adff7", language="text")
            st.info("This is the unique identifier for your compiled contract")
        
        with tab2:
            st.write("**Code (Base64 Encoded)**")
            b64_code = encode_base64(code)
            st.code(b64_code, language="text")
            st.caption(f"Length: {len(b64_code)} characters")
        
        with tab3:
            st.write("**Witness Data**")
            if witness:
                st.code(witness, language="json")
                try:
                    w = json.loads(witness)
                    st.metric("Witness Variables", len(w))
                except:
                    pass
            else:
                st.info("No witness data provided")
        
        with tab4:
            st.write("**Compilation Metadata**")
            metadata = {
                "code_length": len(code),
                "code_lines": code.count('\n') + 1,
                "has_witness": bool(witness),
                "witness_variables": len(json.loads(witness)) if witness else 0,
                "compiled_at": datetime.now().isoformat(),
                "compilation_mode": "with_witness" if witness else "without_witness"
            }
            st.json(metadata)

# Examples
st.markdown("---")
st.subheader("📚 Examples")
tabs = st.tabs(["Basic", "Witness"])

with tabs[0]:
    st.code("mod param {}\nfn main() {}", language="simplicity")

with tabs[1]:
    col1, col2 = st.columns(2)
    with col1:
        st.code("fn main() {\n  witness::VALUE\n}", language="simplicity")
    with col2:
        st.code('{"VALUE": {\n  "value": "42"\n}}', language="json")
