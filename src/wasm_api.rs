use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use simplicityhl::parse::ParseFromStr;
use simplicityhl::CompiledProgram;

#[derive(Serialize, Deserialize)]
pub struct CompileResult {
    pub cmr: Option<String>,
    pub error: Option<String>,
}

#[wasm_bindgen]
pub fn compile_simplicity(code: &str) -> String {
    if code.trim().is_empty() {
        return serde_json::to_string(&CompileResult {
            cmr: None,
            error: Some("Code is empty".to_string()),
        }).unwrap();
    }
    
    // Parse arguments
    match simplicityhl::Arguments::parse_from_str(code) {
        Err(e) => {
            return serde_json::to_string(&CompileResult {
                cmr: None,
                error: Some(format!("Parse error: {}", e)),
            }).unwrap();
        }
        Ok(args) => {
            // Compile
            match CompiledProgram::new(code, args, false) {
                Err(e) => {
                    return serde_json::to_string(&CompileResult {
                        cmr: None,
                        error: Some(format!("Compilation error: {}", e)),
                    }).unwrap();
                }
                Ok(compiled) => {
                    let cmr = compiled.commit().cmr();
                    return serde_json::to_string(&CompileResult {
                        cmr: Some(format!("{}", cmr)),
                        error: None,
                    }).unwrap();
                }
            }
        }
    }
}

#[wasm_bindgen]
pub fn compile_with_witness(_code: &str) -> String {
    serde_json::to_string(&CompileResult {
        cmr: None,
        error: Some("Not implemented".to_string()),
    }).unwrap()
}

#[wasm_bindgen]
pub fn parse_program(_code: &str) -> String {
    serde_json::to_string(&CompileResult {
        cmr: None,
        error: Some("Not implemented".to_string()),
    }).unwrap()
}
