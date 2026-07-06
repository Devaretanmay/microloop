use serde_json::Value;

pub fn strip_volatile_fields(value: &mut Value, paths: &[String]) {
    for path in paths {
        let clean_path = path.strip_prefix("$.").unwrap_or(path);
        if clean_path.is_empty() {
            continue;
        }
        if let Some((parent, last)) = clean_path.rsplit_once('.') {
            let parent_pointer = format!("/{}", parent.replace('.', "/"));
            if let Some(Value::Object(map)) = value.pointer_mut(&parent_pointer) {
                map.remove(last);
            }
        } else if let Some(map) = value.as_object_mut() {
            map.remove(clean_path);
        }
    }
}
