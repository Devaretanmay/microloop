use serde_json::Value;

pub fn strip_volatile_fields(value: &mut Value, paths: &[String]) {
    for path_str in paths {
        let segments = parse_json_path(path_str);
        if segments.is_empty() {
            continue;
        }
        let (last, parent_segs) = segments.split_last().unwrap();
        let parent_path = format!("/{}", parent_segs.join("/"));

        let target = if parent_segs.is_empty() {
            Some(&mut *value)
        } else {
            value.pointer_mut(&parent_path)
        };

        if let Some(Value::Object(map)) = target {
            map.remove(last);
        }
    }
}

fn parse_json_path(path: &str) -> Vec<String> {
    let s = if let Some(stripped) = path.strip_prefix("$.") {
        stripped
    } else {
        path
    };
    s.split('.').map(|p| p.to_string()).collect()
}
