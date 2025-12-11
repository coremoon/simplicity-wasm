use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use simplicityhl::parse::ParseFromStr;
use simplicityhl::CompiledProgram;

#[derive(Serialize, Deserialize, Debug)]
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
        }).unwrap_or_else(|_| r#"{"cmr":null,"error":"Serialization error"}"#.to_string());
    }
    
    // Parse arguments
    match simplicityhl::Arguments::parse_from_str(code) {
        Err(e) => {
            let result = CompileResult {
                cmr: None,
                error: Some(format!("Parse error: {}", e)),
            };
            serde_json::to_string(&result).unwrap_or_else(|_| r#"{"cmr":null,"error":"Serialization error"}"#.to_string())
        }
        Ok(args) => {
            // Compile
            match CompiledProgram::new(code, args, false) {
                Err(e) => {
                    let result = CompileResult {
                        cmr: None,
                        error: Some(format!("Compilation error: {}", e)),
                    };
                    serde_json::to_string(&result).unwrap_or_else(|_| r#"{"cmr":null,"error":"Serialization error"}"#.to_string())
                }
                Ok(compiled) => {
                    let cmr = compiled.commit().cmr();
                    let result = CompileResult {
                        cmr: Some(format!("{}", cmr)),
                        error: None,
                    };
                    serde_json::to_string(&result).unwrap_or_else(|_| r#"{"cmr":null,"error":"Serialization error"}"#.to_string())
                }
            }
        }
    }
}

/// Compile with witness data support
/// witness_data: JSON format with witness variables
#[wasm_bindgen]
pub fn compile_with_witness(code: &str, witness_data: &str) -> String {
    if code.trim().is_empty() {
        return serde_json::to_string(&CompileResult {
            cmr: None,
            error: Some("Code is empty".to_string()),
        }).unwrap_or_else(|_| r#"{"cmr":null,"error":"Serialization error"}"#.to_string());
    }
    
    if witness_data.trim().is_empty() {
        return serde_json::to_string(&CompileResult {
            cmr: None,
            error: Some("Witness data is empty".to_string()),
        }).unwrap_or_else(|_| r#"{"cmr":null,"error":"Serialization error"}"#.to_string());
    }
    
    // First, validate that witness_data is valid JSON
    match serde_json::from_str::<serde_json::Value>(witness_data) {
        Err(e) => {
            let result = CompileResult {
                cmr: None,
                error: Some(format!("Invalid JSON witness data: {}", e)),
            };
            return serde_json::to_string(&result).unwrap_or_else(|_| r#"{"cmr":null,"error":"Serialization error"}"#.to_string());
        }
        Ok(_) => {} // Valid JSON, continue
    }
    
    // Parse arguments from code
    match simplicityhl::Arguments::parse_from_str(code) {
        Err(e) => {
            let result = CompileResult {
                cmr: None,
                error: Some(format!("Parse error: {}", e)),
            };
            serde_json::to_string(&result).unwrap_or_else(|_| r#"{"cmr":null,"error":"Serialization error"}"#.to_string())
        }
        Ok(args) => {
            // Compile code
            match CompiledProgram::new(code, args, false) {
                Err(e) => {
                    let result = CompileResult {
                        cmr: None,
                        error: Some(format!("Compilation error: {}", e)),
                    };
                    serde_json::to_string(&result).unwrap_or_else(|_| r#"{"cmr":null,"error":"Serialization error"}"#.to_string())
                }
                Ok(compiled) => {
                    // Get CMR
                    let cmr = compiled.commit().cmr();
                    
                    // Return success with witness data stored
                    let result = CompileResult {
                        cmr: Some(format!("{}", cmr)),
                        error: None,
                    };
                    
                    // Create extended response with witness data
                    let mut response = serde_json::to_value(&result).unwrap();
                    response["witness_data"] = serde_json::from_str(witness_data).unwrap_or(serde_json::json!({}));
                    
                    serde_json::to_string(&response).unwrap_or_else(|_| r#"{"cmr":null,"error":"Serialization error"}"#.to_string())
                }
            }
        }
    }
}

#[wasm_bindgen]
pub fn parse_program(code: &str) -> String {
    if code.trim().is_empty() {
        return serde_json::to_string(&CompileResult {
            cmr: None,
            error: Some("Code is empty".to_string()),
        }).unwrap_or_else(|_| r#"{"cmr":null,"error":"Serialization error"}"#.to_string());
    }
    
    match simplicityhl::Arguments::parse_from_str(code) {
        Err(e) => {
            let result = CompileResult {
                cmr: None,
                error: Some(format!("Parse error: {}", e)),
            };
            serde_json::to_string(&result).unwrap_or_else(|_| r#"{"cmr":null,"error":"Serialization error"}"#.to_string())
        }
        Ok(args) => {
            let result = CompileResult {
                cmr: Some(format!("Parsed successfully: {:?}", args)),
                error: None,
            };
            serde_json::to_string(&result).unwrap_or_else(|_| r#"{"cmr":null,"error":"Serialization error"}"#.to_string())
        }
    }
}
