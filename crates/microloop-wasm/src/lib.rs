use microloop::MicroloopState;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct MicroloopWasm {
    state: MicroloopState,
}

#[wasm_bindgen]
impl MicroloopWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(yaml_config: &str) -> Result<MicroloopWasm, JsValue> {
        let state = MicroloopState::new(yaml_config).map_err(|e| JsValue::from_str(&e))?;
        Ok(MicroloopWasm { state })
    }

    #[wasm_bindgen]
    pub fn verify(&mut self, tool_name: &str, tool_args_json: &str) -> u8 {
        microloop::verify(
            &mut self.state,
            tool_name.as_bytes(),
            tool_args_json.as_bytes(),
        )
    }

    #[wasm_bindgen]
    pub fn get_last_error(&self) -> Option<String> {
        if self.state.error_buffer.is_empty() {
            None
        } else {
            let s = std::str::from_utf8(&self.state.error_buffer)
                .unwrap_or("")
                .trim_end_matches('\0')
                .to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        }
    }
}
