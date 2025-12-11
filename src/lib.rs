pub mod wasm_api;

use leptos::prelude::*;
use leptos::html::Textarea;
use wasm_bindgen::prelude::*;
use web_sys::HtmlTextAreaElement;

use wasm_bindgen::JsCast;

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
    let (witness, set_witness) = signal(String::new());
    let (cmr, set_cmr) = signal::<Option<String>>(None);
    let (code_base64, set_code_base64) = signal::<Option<String>>(None);
    let (witness_info, set_witness_info) = signal::<Option<String>>(None);
    let (error, set_error) = signal::<Option<String>>(None);
    let textarea_ref = NodeRef::<Textarea>::new();
    let (drag_over_code, set_drag_over_code) = signal(false);
    let (drag_over_witness, set_drag_over_witness) = signal(false);

    let handle_compile = move |_| {
        let code_value = code.get();
        let witness_value = witness.get();
        
        if code_value.trim().is_empty() {
            set_error.set(Some("Code is empty".to_string()));
            set_cmr.set(None);
            set_code_base64.set(None);
            set_witness_info.set(None);
            return;
        }

        log(&format!("Compiling: {}", code_value));
        set_error.set(None);

        // Check if witness data is provided and use appropriate compilation method
        let compile_result = if !witness_value.trim().is_empty() {
            log("Using compile_with_witness");
            wasm_api::compile_with_witness(&code_value, &witness_value)
        } else {
            log("Using compile_simplicity");
            wasm_api::compile_simplicity(&code_value)
        };

        log(&format!("Compile result: {}", compile_result));
        
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
            
            if let Some(cmr_val) = parsed.get("cmr").and_then(|v| v.as_str()) {
                if cmr_val != "null" && !cmr_val.is_empty() {
                    set_cmr.set(Some(cmr_val.to_string()));
                }
            }
            
            let b64 = encode_base64(&code_value);
            set_code_base64.set(Some(b64));
            
            if let Some(w) = parsed.get("witness") {
                let witness_str = w.to_string();
                set_witness_info.set(Some(witness_str));
            } else if !witness_value.trim().is_empty() {
                set_witness_info.set(Some("Witness processed successfully".to_string()));
            } else {
                set_witness_info.set(Some("No witness data provided".to_string()));
            }
            
            set_error.set(None);
            return;
        }
        
        set_error.set(Some("Invalid response from compiler".to_string()));
        set_cmr.set(None);
        set_code_base64.set(None);
        set_witness_info.set(None);
    };

    let insert_template = move |_| {
        if let Some(textarea) = textarea_ref.get() {
            let textarea_el: HtmlTextAreaElement = textarea.into();
            let start = match textarea_el.selection_start() {
                Ok(Some(pos)) => pos as usize,
                _ => 0,
            };
            
            let current_code = code.get();
            let template = "mod param {}\nfn main() {}";
            let mut new_code = current_code.clone();
            new_code.insert_str(start, template);
            
            set_code.set(new_code.clone());
            
            let new_pos = start + template.len();
            set_timeout(
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

    let clear_code = move |_| {
        set_code.set(String::new());
        set_cmr.set(None);
        set_code_base64.set(None);
        set_witness_info.set(None);
        set_error.set(None);
    };

    let clear_witness = move |_| {
        set_witness.set(String::new());
    };

    // Drag & Drop for .simf files
    let handle_simf_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        set_drag_over_code.set(false);
        
        if let Some(data_transfer) = ev.data_transfer() {
            if let Some(files) = data_transfer.files() {
                if files.length() > 0 {
                    if let Some(file) = files.get(0) {
                        let file_name = file.name();
                        if file_name.ends_with(".simf") {
                            let reader = web_sys::FileReader::new().ok();
                            if let Some(reader) = reader {
                                let reader_clone = reader.clone();
                                let onload = Closure::wrap(
                                    Box::new(move |_: web_sys::ProgressEvent| {
                                        if let Ok(content) = reader_clone.result() {
                                            if let Some(text) = content.as_string() {
                                                set_code.set(text);
                                                set_error.set(None);
                                            }
                                        }
                                    }) as Box<dyn FnMut(web_sys::ProgressEvent)>
                                );
                                
                                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                                onload.forget();
                                
                                let _ = reader.read_as_text(&file);
                            }
                        } else {
                            set_error.set(Some("Only .simf files are supported for code".to_string()));
                        }
                    }
                }
            }
        }
    };

    // Drag & Drop for witness files
    let handle_witness_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        set_drag_over_witness.set(false);
        
        if let Some(data_transfer) = ev.data_transfer() {
            if let Some(files) = data_transfer.files() {
                if files.length() > 0 {
                    if let Some(file) = files.get(0) {
                        let reader = web_sys::FileReader::new().ok();
                        if let Some(reader) = reader {
                            let reader_clone = reader.clone();
                            let onload = Closure::wrap(
                                Box::new(move |_: web_sys::ProgressEvent| {
                                    if let Ok(content) = reader_clone.result() {
                                        if let Some(text) = content.as_string() {
                                            set_witness.set(text);
                                        }
                                    }
                                }) as Box<dyn FnMut(web_sys::ProgressEvent)>
                            );
                            
                            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                            onload.forget();
                            
                            let _ = reader.read_as_text(&file);
                        }
                    }
                }
            }
        }
    };

    let handle_simf_dragover = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        set_drag_over_code.set(true);
    };

    let handle_simf_dragleave = move |_: web_sys::DragEvent| {
        set_drag_over_code.set(false);
    };

    let handle_witness_dragover = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        set_drag_over_witness.set(true);
    };

    let handle_witness_dragleave = move |_: web_sys::DragEvent| {
        set_drag_over_witness.set(false);
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
                
                .section label {
                    font-weight: 600;
                    color: #333;
                    font-size: 14px;
                    display: block;
                    margin-bottom: 12px;
                }
                
                .button-group {
                    display: flex;
                    gap: 10px;
                    margin-top: 15px;
                    flex-wrap: wrap;
                }
                
                .drop-zone {
                    border: 2px dashed #ccc;
                    border-radius: 8px;
                    padding: 40px 20px;
                    text-align: center;
                    background: #f9f9f9;
                    cursor: pointer;
                    transition: all 0.2s ease;
                    margin-bottom: 15px;
                    min-height: 100px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    flex-direction: column;
                }
                
                .drop-zone:hover {
                    border-color: #007bff;
                    background: #f0f8ff;
                }
                
                .drop-zone.drag-over {
                    border-color: #007bff;
                    background: #e7f3ff;
                    box-shadow: 0 0 8px rgba(0, 123, 255, 0.3);
                }
                
                .drop-zone-icon {
                    font-size: 32px;
                    margin-bottom: 10px;
                }
                
                .drop-zone-text {
                    color: #666;
                    font-size: 14px;
                    font-weight: 500;
                }
                
                .drop-zone-hint {
                    color: #999;
                    font-size: 12px;
                    margin-top: 8px;
                }
                
                textarea {
                    width: 100%;
                    height: 200px;
                    padding: 12px;
                    border: 1px solid #ddd;
                    border-radius: 4px;
                    font-family: 'Monaco', 'Courier New', monospace;
                    font-size: 13px;
                    resize: none;
                    background: #fafafa;
                    margin-bottom: 15px;
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
                
                button.danger {
                    background: #dc3545;
                }
                
                button.danger:hover {
                    background: #c82333;
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
                        <label>"SimplicityHL Code"</label>
                        
                        <div 
                            class=move || {
                                if drag_over_code.get() {
                                    "drop-zone drag-over"
                                } else {
                                    "drop-zone"
                                }
                            }
                            on:dragover=handle_simf_dragover
                            on:dragleave=handle_simf_dragleave
                            on:drop=handle_simf_drop
                        >
                            <div class="drop-zone-icon">"📄"</div>
                            <div class="drop-zone-text">"Drag here to import .simf file"</div>
                            <div class="drop-zone-hint">"or edit directly below"</div>
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
                            <button class="danger" on:click=clear_code>
                                "🗑️ Clear"
                            </button>
                        </div>
                    </div>

                    {/* Right: Witness Input */}
                    <div class="section">
                        <label>"Witness Data"</label>
                        
                        <div 
                            class=move || {
                                if drag_over_witness.get() {
                                    "drop-zone drag-over"
                                } else {
                                    "drop-zone"
                                }
                            }
                            on:dragover=handle_witness_dragover
                            on:dragleave=handle_witness_dragleave
                            on:drop=handle_witness_drop
                        >
                            <div class="drop-zone-icon">"📋"</div>
                            <div class="drop-zone-text">"Drag here to import witness file"</div>
                            <div class="drop-zone-hint">"or paste witness data below"</div>
                        </div>
                        
                        <textarea
                            prop:value=move || witness.get()
                            on:input=move |ev| {
                                set_witness.set(event_target_value(&ev));
                            }
                            placeholder="Witness data will appear here..."
                        />
                        
                        <div class="button-group">
                            <button class="danger" on:click=clear_witness>
                                "🗑️ Clear Witness"
                            </button>
                        </div>
                        
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
                        <Show
                            when=move || cmr.get().is_some()
                            fallback=move || {
                                view! {
                                    <div class="empty-state">
                                        "Compilation results will appear here"
                                    </div>
                                }
                            }
                        >
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
                        </Show>
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

fn set_timeout<F>(f: F, duration: std::time::Duration)
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

fn encode_base64(data: &str) -> String {
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name = btoa)]
        fn btoa(s: &str) -> String;
    }
    
    let bytes = data.as_bytes();
    let mut latin1_string = String::new();
    for &byte in bytes {
        latin1_string.push(byte as char);
    }
    
    btoa(&latin1_string)
}
