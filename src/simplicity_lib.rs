pub mod wasm_api;

use leptos::prelude::*;
use leptos::html::Textarea;
use wasm_bindgen::prelude::*;
use web_sys::HtmlTextAreaElement;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    log("Simplicity WASM App Started");
    
    leptos::mount::mount_to_body(|| {
        view! {
            <App />
        }
    });
}

#[component]
fn App() -> impl IntoView {
    let (code, set_code) = signal("mod param {}\nfn main() {}".to_string());
    let (cmr, set_cmr) = signal::<Option<String>>(None);
    let (code_base64, set_code_base64) = signal::<Option<String>>(None);
    let (witness_info, set_witness_info) = signal::<Option<String>>(None);
    let (error, set_error) = signal::<Option<String>>(None);
    let textarea_ref = NodeRef::<Textarea>::new();

    let handle_compile = move |_| {
        let code_value = code.get();
        
        if code_value.trim().is_empty() {
            set_error.set(Some("Code is empty".to_string()));
            set_cmr.set(None);
            set_code_base64.set(None);
            set_witness_info.set(None);
            return;
        }

        log(&format!("Compiling: {}", code_value));
        set_error.set(None);

        // Call the WASM function
        match wasm_api::compile_simplicity(&code_value) {
            compile_result => {
                log(&format!("Compile result: {}", compile_result));
                
                // Parse the JSON result
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&compile_result) {
                    if let Some(err) = parsed.get("error").and_then(|v| v.as_str()) {
                        if err != "null" && !err.is_empty() {
                            set_error.set(Some(err.to_string()));
                            set_cmr.set(None);
                            set_code_base64.set(None);
                            set_witness_info.set(None);
                            return;
                        }
                    }
                    
                    // Extract CMR
                    if let Some(cmr_val) = parsed.get("cmr").and_then(|v| v.as_str()) {
                        if cmr_val != "null" && !cmr_val.is_empty() {
                            set_cmr.set(Some(cmr_val.to_string()));
                        }
                    }
                    
                    // Encode code to base64
                    let b64 = encode_base64(&code_value);
                    set_code_base64.set(Some(b64));
                    
                    // Extract witness info if available
                    if let Some(witness) = parsed.get("witness") {
                        let witness_str = witness.to_string();
                        set_witness_info.set(Some(witness_str));
                    } else {
                        // Fallback witness info
                        set_witness_info.set(Some("No witness data provided".to_string()));
                    }
                    
                    set_error.set(None);
                    return;
                }
                
                set_error.set(Some("Invalid response from compiler".to_string()));
                set_cmr.set(None);
                set_code_base64.set(None);
                set_witness_info.set(None);
            }
        }
    };

    let insert_template = move |_| {
        if let Some(textarea) = textarea_ref.get() {
            let textarea_el: HtmlTextAreaElement = textarea.into();
            // Correct type handling for selection_start
            let start = match textarea_el.selection_start() {
                Ok(Some(pos)) => pos as usize,
                _ => 0,
            };
            
            let current_code = code.get();
            
            let template = "mod param {}\nfn main() {}";
            let mut new_code = current_code.clone();
            new_code.insert_str(start, template);
            
            set_code.set(new_code.clone());
            
            // Restore cursor position after template
            let new_pos = start + template.len();
            setTimeout(
                move || {
                    if let Some(textarea) = textarea_ref.get() {
                        let textarea_el: HtmlTextAreaElement = textarea.into();
                        let _ = textarea_el.set_selection_range(new_pos as u32, new_pos as u32);
                    }
                },
                std::time::Duration::from_millis(10),
            );
        }
    };

    view! {
        <>
            <style>
                {r#"
                * {
                    margin: 0;
                    padding: 0;
                    box-sizing: border-box;
                }
                
                body {
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    background: #f5f5f5;
                }
                
                .container {
                    max-width: 1400px;
                    margin: 0 auto;
                    padding: 30px 20px;
                }
                
                .header {
                    margin-bottom: 30px;
                }
                
                .header h1 {
                    font-size: 32px;
                    margin-bottom: 10px;
                    color: #333;
                }
                
                .header p {
                    color: #666;
                    font-size: 16px;
                }
                
                .grid {
                    display: grid;
                    grid-template-columns: 1fr 1fr;
                    gap: 30px;
                    margin-bottom: 40px;
                }
                
                .section {
                    background: white;
                    padding: 25px;
                    border-radius: 8px;
                    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
                }
                
                .section-header {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-bottom: 12px;
                }
                
                .section label {
                    font-weight: 600;
                    color: #333;
                    font-size: 14px;
                }
                
                .button-group {
                    display: flex;
                    gap: 10px;
                    margin-top: 15px;
                }
                
                textarea {
                    width: 100%;
                    height: 400px;
                    padding: 12px;
                    border: 1px solid #ddd;
                    border-radius: 4px;
                    font-family: 'Monaco', 'Courier New', monospace;
                    font-size: 13px;
                    resize: none;
                    background: #fafafa;
                }
                
                textarea:focus {
                    outline: none;
                    border-color: #007bff;
                    background: white;
                    box-shadow: 0 0 0 3px rgba(0, 123, 255, 0.1);
                }
                
                button {
                    padding: 10px 16px;
                    background: #007bff;
                    color: white;
                    border: none;
                    border-radius: 4px;
                    cursor: pointer;
                    font-size: 14px;
                    font-weight: 600;
                    transition: background 0.2s;
                    white-space: nowrap;
                }
                
                button:hover {
                    background: #0056b3;
                }
                
                button:active {
                    background: #004085;
                }
                
                button.secondary {
                    background: #6c757d;
                }
                
                button.secondary:hover {
                    background: #5a6268;
                }
                
                .error {
                    padding: 15px;
                    background: #f8d7da;
                    color: #721c24;
                    border: 1px solid #f5c6cb;
                    border-radius: 4px;
                    margin-bottom: 15px;
                }
                
                .error strong {
                    display: block;
                    margin-bottom: 8px;
                }
                
                .error pre {
                    margin: 0;
                    white-space: pre-wrap;
                    word-break: break-word;
                    font-size: 13px;
                    font-family: 'Monaco', 'Courier New', monospace;
                }
                
                .success {
                    padding: 15px;
                    background: #d4edda;
                    color: #155724;
                    border: 1px solid #c3e6cb;
                    border-radius: 4px;
                    margin-bottom: 15px;
                }
                
                .success strong {
                    display: block;
                    margin-bottom: 10px;
                }
                
                .output-group {
                    margin-bottom: 20px;
                }
                
                .output-label {
                    font-weight: 600;
                    color: #155724;
                    font-size: 12px;
                    text-transform: uppercase;
                    margin-bottom: 6px;
                    display: block;
                }
                
                .output-box {
                    background: #f5f5f5;
                    padding: 12px;
                    border-radius: 3px;
                    word-break: break-all;
                    font-family: 'Monaco', 'Courier New', monospace;
                    font-size: 12px;
                    color: #333;
                    overflow-x: auto;
                    max-height: 150px;
                    overflow-y: auto;
                    border: 1px solid #e0e0e0;
                }
                
                .output-box.witness {
                    max-height: 200px;
                    white-space: pre-wrap;
                    word-wrap: break-word;
                }
                
                .empty-state {
                    padding: 40px 30px;
                    background: #f0f0f0;
                    border-radius: 4px;
                    text-align: center;
                    color: #666;
                    min-height: 200px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                }
                
                .footer {
                    margin-top: 40px;
                    padding-top: 20px;
                    border-top: 1px solid #ddd;
                    color: #666;
                    font-size: 14px;
                }
                
                .footer p {
                    margin-bottom: 8px;
                }
                
                @media (max-width: 768px) {
                    .grid {
                        grid-template-columns: 1fr;
                        gap: 20px;
                    }
                    
                    .header h1 {
                        font-size: 24px;
                    }
                    
                    .button-group {
                        flex-direction: column;
                    }
                    
                    button {
                        width: 100%;
                    }
                }
                "#}
            </style>
            
            <div class="container">
                <div class="header">
                    <h1>"Simplicity WASM Compiler"</h1>
                    <p>"Compile Simplicity smart contracts directly in your browser"</p>
                </div>
                
                <div class="grid">
                    {/* Left: Code Input */}
                    <div class="section">
                        <div class="section-header">
                            <label>"Simplicity Program:"</label>
                        </div>
                        <textarea
                            node_ref=textarea_ref
                            prop:value=move || code.get()
                            on:input=move |ev| {
                                set_code.set(event_target_value(&ev));
                            }
                            placeholder="Enter Simplicity code here..."
                        />
                        <div class="button-group">
                            <button on:click=handle_compile>
                                "🔨 Compile"
                            </button>
                            <button class="secondary" on:click=insert_template>
                                "📋 Insert Template"
                            </button>
                        </div>
                    </div>

                    {/* Right: Results */}
                    <div class="section">
                        <label>"Compilation Results:"</label>
                        
                        {/* Error Display */}
                        {move || {
                            error.get().map(|err| {
                                view! {
                                    <div class="error">
                                        <strong>"⚠️ Error:"</strong>
                                        <pre>{err}</pre>
                                    </div>
                                }
                            })
                        }}

                        {/* Success Results */}
                        {move || {
                            if cmr.get().is_some() {
                                return view! {
                                    <div class="success">
                                        <strong>"✅ Compilation Successful!"</strong>
                                        
                                        <div class="output-group">
                                            <span class="output-label">"CMR (Commitment Merkle Root):"</span>
                                            <div class="output-box">
                                                {move || cmr.get().unwrap_or_default()}
                                            </div>
                                        </div>
                                        
                                        <div class="output-group">
                                            <span class="output-label">"Code (Base64):"</span>
                                            <div class="output-box">
                                                {move || code_base64.get().unwrap_or_default()}
                                            </div>
                                        </div>
                                        
                                        <div class="output-group">
                                            <span class="output-label">"Witness Information:"</span>
                                            <div class="output-box witness">
                                                {move || witness_info.get().unwrap_or_default()}
                                            </div>
                                        </div>
                                    </div>
                                }.into_view();
                            }
                            
                            view! {
                                <div class="empty-state">
                                    "Enter your Simplicity code and click Compile to see results"
                                </div>
                            }.into_view()
                        }}
                    </div>
                </div>

                <div class="footer">
                    <p>"This is a WebAssembly-based Simplicity compiler running entirely in your browser."</p>
                    <p>"No data is sent to any server."</p>
                </div>
            </div>
        </>
    }
}

fn setTimeout<F>(f: F, duration: std::time::Duration)
where
    F: FnOnce() + 'static,
{
    use wasm_bindgen::closure::Closure;
    use web_sys::window;

    let closure = Closure::once(f);
    window()
        .expect("no window")
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            duration.as_millis() as i32,
        )
        .expect("failed to set timeout");
    closure.forget();
}

// Base64 encoding function
fn encode_base64(data: &str) -> String {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        type Uint8Array;
        
        #[wasm_bindgen(constructor)]
        fn new(len: usize) -> Uint8Array;
        
        #[wasm_bindgen(method, indexing_getter)]
        fn get(this: &Uint8Array, index: u32) -> u8;
        
        #[wasm_bindgen(method, indexing_setter)]
        fn set(this: &Uint8Array, index: u32, value: u8);
    }
    
    #[wasm_bindgen]
    extern "C" {
        type Global;
        
        #[wasm_bindgen(js_name = btoa)]
        fn btoa(s: &str) -> String;
    }
    
    // For simplicity, use JavaScript's btoa function
    // Convert UTF-8 string to bytes first
    let bytes = data.as_bytes();
    let mut latin1_string = String::new();
    for &byte in bytes {
        latin1_string.push(byte as char);
    }
    
    btoa(&latin1_string)
}
