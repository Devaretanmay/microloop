
use std::collections::{HashMap, HashSet};


/// A single tool call in a trajectory log.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TrajectoryEntry {
    pub tool: String,
    pub args: serde_json::Value,
    pub result: Option<String>,
    #[serde(default)]
    pub error: bool,
    pub ts: u64,
}

/// Analysis results for a single tool.
#[derive(Debug, Clone)]
pub struct ToolAnalysis {
    pub name: String,
    pub call_count: usize,
    pub repeat_count: usize,
    pub error_count: usize,
    pub error_rate: f64,
    /// Fields that change on every call (volatile/noisy like request IDs)
    pub volatile_fields: Vec<String>,
    pub stable_fields: Vec<String>,
    pub suggested_max_repeats: usize,
    pub max_productive_sequence: usize,
    pub oscillation_detected: bool,
}

/// Complete analysis report for a trajectory.
#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub tools: Vec<ToolAnalysis>,
    pub total_calls: usize,
    pub total_loops: usize,
    pub global_error_rate: f64,
    /// Minimum suggested max_repeats across all tools.
    pub suggested_global_max_repeats: usize,
}

/// A learned policy that can be serialized to YAML.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LearnedPolicy {
    #[serde(default)]
    pub sensitivity: String,
    #[serde(default)]
    pub tools: Vec<LearnedToolConfig>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LearnedToolConfig {
    pub name: String,
    #[serde(rename = "trajectory_gate")]
    pub trajectory_gate: LearnedTrajectoryGate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<ToolStats>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LearnedTrajectoryGate {
    pub max_repeats: usize,
    pub volatile_fields: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolStats {
    pub calls: usize,
    pub repeats: usize,
    pub errors: usize,
    pub error_rate: f64,
}

/// Analyze a trajectory and produce a full analysis report.
pub fn analyze_trajectory(entries: &[TrajectoryEntry]) -> AnalysisReport {
    if entries.is_empty() {
        return AnalysisReport {
            tools: vec![],
            total_calls: 0,
            total_loops: 0,
            global_error_rate: 0.0,
            suggested_global_max_repeats: 3,
        };
    }

    let mut tool_entries: HashMap<String, Vec<&TrajectoryEntry>> = HashMap::new();
    for entry in entries {
        tool_entries.entry(entry.tool.clone()).or_default().push(entry);
    }

    let oscillating_tools = detect_oscillation_across_tools(entries);

    let mut tools = Vec::new();
    let mut total_errors = 0;

    for (name, call_entries) in &tool_entries {
        let mut analysis = analyze_tool(name, call_entries);
        if oscillating_tools.contains(name) {
            analysis.oscillation_detected = true;
        }
        total_errors += analysis.error_count;
        tools.push(analysis);
    }

    tools.sort_by(|a, b| b.call_count.cmp(&a.call_count));

    let total_calls = entries.len();
    let total_loops: usize = tools.iter().map(|t| t.repeat_count.saturating_sub(1)).sum();
    let global_error_rate = if total_calls > 0 {
        total_errors as f64 / total_calls as f64
    } else {
        0.0
    };

    let suggested_global_max_repeats = tools
        .iter()
        .map(|t| t.suggested_max_repeats)
        .min()
        .unwrap_or(3)
        .clamp(2, 10);

    AnalysisReport {
        tools,
        total_calls,
        total_loops,
        global_error_rate,
        suggested_global_max_repeats,
    }
}

fn analyze_tool(name: &str, entries: &[&TrajectoryEntry]) -> ToolAnalysis {
    let call_count = entries.len();
    let error_count = entries.iter().filter(|e| e.error).count();
    let error_rate = if call_count > 0 {
        error_count as f64 / call_count as f64
    } else {
        0.0
    };

    let max_productive_sequence = find_longest_productive_sequence(entries);
    let repeat_count = count_repeats(entries);
    let (volatile_fields, stable_fields) = detect_volatile_fields(entries);
    let suggested_max_repeats = compute_optimal_max_repeats(
        max_productive_sequence,
        repeat_count,
        error_rate,
    );

    // oscillation_detected is set at the trajectory level in analyze_trajectory
    let oscillation_detected = false;

    ToolAnalysis {
        name: name.to_string(),
        call_count,
        repeat_count,
        error_count,
        error_rate,
        volatile_fields,
        stable_fields,
        suggested_max_repeats,
        max_productive_sequence,
        oscillation_detected,
    }
}

/// Find the longest sequence of unique (non-repeating) calls for a tool.
/// This represents the agent's longest productive streak.
fn find_longest_productive_sequence(entries: &[&TrajectoryEntry]) -> usize {
    if entries.is_empty() {
        return 0;
    }

    let mut max_seq = 1;
    let mut current_seq = 1;

    for i in 1..entries.len() {
        let prev = &entries[i - 1];
        let curr = &entries[i];

        // If args differ, it's a new productive step
        if prev.args != curr.args {
            current_seq += 1;
            if current_seq > max_seq {
                max_seq = current_seq;
            }
        } else {
            current_seq = 1;
        }
    }

    max_seq
}

/// Count how many calls are repeats (same tool + same args as previous).
fn count_repeats(entries: &[&TrajectoryEntry]) -> usize {
    if entries.is_empty() {
        return 0;
    }

    let mut repeats = 0;
    for i in 1..entries.len() {
        if entries[i].args == entries[i - 1].args {
            repeats += 1;
        }
    }
    repeats
}

/// A field is "volatile" if it changes on nearly every call (request IDs, timestamps).
fn detect_volatile_fields(entries: &[&TrajectoryEntry]) -> (Vec<String>, Vec<String>) {
    if entries.is_empty() {
        return (vec![], vec![]);
    }

    let mut all_fields: HashSet<String> = HashSet::new();
    for entry in entries {
        if let Some(obj) = entry.args.as_object() {
            for key in obj.keys() {
                all_fields.insert(key.clone());
            }
        }
    }

    let mut volatile = Vec::new();
    let mut stable = Vec::new();

    for field in all_fields {
        let values: Vec<Option<&serde_json::Value>> = entries
            .iter()
            .map(|e| e.args.get(&field))
            .collect();

        let non_null: Vec<_> = values.iter().filter_map(|v| *v).collect();

        if non_null.len() < 2 {
            continue;
        }

        let all_same = non_null.windows(2).all(|w| w[0] == w[1]);

        if all_same {
            stable.push(field);
        } else {
            let unique_count: usize = {
                let mut seen = HashSet::new();
                for v in &non_null {
                    seen.insert(format!("{:?}", v));
                }
                seen.len()
            };

            let ratio = unique_count as f64 / non_null.len() as f64;
            if ratio > 0.8 && non_null.len() >= 2 {
                volatile.push(field);
            } else {
                stable.push(field);
            }
        }
    }

    volatile.sort();
    stable.sort();

    (volatile, stable)
}

/// Tightens or loosens max_repeats based on error rate and productive sequence length.
fn compute_optimal_max_repeats(
    max_productive_sequence: usize,
    repeat_count: usize,
    error_rate: f64,
) -> usize {
    if max_productive_sequence == 0 {
        return 3;
    }

    let mut suggested = max_productive_sequence + 2;

    if error_rate > 0.3 {
        suggested = (suggested as f64 * 0.7) as usize;
    }

    if repeat_count > max_productive_sequence * 2 {
        suggested = (suggested as f64 * 0.8) as usize;
    }

    suggested.clamp(2, 10)
}

/// Detect oscillation patterns across ALL tools in the trajectory.
///
/// Looks for repeating patterns like A→B→A→B (period 2) or
/// A→A→B→B→A→A→B→B (period 4) across the full tool sequence.
fn detect_oscillation_across_tools(entries: &[TrajectoryEntry]) -> Vec<String> {
    if entries.len() < 4 {
        return vec![];
    }

    // Extract tool names in sequence order
    let tools: Vec<&str> = entries.iter().map(|e| e.tool.as_str()).collect();
    let window = tools.len().min(16);
    let recent = &tools[tools.len() - window..];

    // Check for pattern: A, B, A, B (period = 2)
    if window >= 4 {
        let mut pattern_ok = true;
        for i in 2..recent.len() {
            if recent[i] != recent[i - 2] {
                pattern_ok = false;
                break;
            }
        }
        if pattern_ok && recent[0] != recent[1] {
            return vec![recent[0].to_string(), recent[1].to_string()];
        }
    }

    // Check for pattern: A, A, B, B, A, A, B, B (period = 4)
    if window >= 8 {
        let mut pattern_ok = true;
        for i in 4..recent.len() {
            if recent[i] != recent[i - 4] {
                pattern_ok = false;
                break;
            }
        }
        if pattern_ok {
            return vec![recent[0].to_string(), recent[4].to_string()];
        }
    }

    vec![]
}



#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entry(tool: &str, args: serde_json::Value, error: bool, ts: u64) -> TrajectoryEntry {
        TrajectoryEntry {
            tool: tool.into(),
            args,
            result: None,
            error,
            ts,
        }
    }

    #[test]
    fn empty_trajectory() {
        let report = analyze_trajectory(&[]);
        assert_eq!(report.total_calls, 0);
        assert_eq!(report.suggested_global_max_repeats, 3);
    }

    #[test]
    fn single_tool_no_repeats() {
        let entries = vec![
            entry("read_file", json!({"path": "a.rs"}), false, 1),
            entry("read_file", json!({"path": "b.rs"}), false, 2),
            entry("read_file", json!({"path": "c.rs"}), false, 3),
        ];
        let report = analyze_trajectory(&entries);
        assert_eq!(report.total_calls, 3);
        assert_eq!(report.tools.len(), 1);
        assert_eq!(report.tools[0].repeat_count, 0);
        assert_eq!(report.tools[0].suggested_max_repeats, 5);
    }

    #[test]
    fn detects_repeats() {
        let entries = vec![
            entry("delete_line", json!({"line": 5}), true, 1),
            entry("delete_line", json!({"line": 5}), true, 2),
            entry("delete_line", json!({"line": 5}), true, 3),
        ];
        let report = analyze_trajectory(&entries);
        let tool = &report.tools[0];
        assert_eq!(tool.repeat_count, 2);
        assert_eq!(tool.error_count, 3);
        assert!(tool.error_rate > 0.9);
    }

    #[test]
    fn detects_volatile_fields() {
        let entries = vec![
            entry("search", json!({"q": "rust", "sid": "aaa"}), false, 1),
            entry("search", json!({"q": "rust", "sid": "bbb"}), false, 2),
            entry("search", json!({"q": "rust", "sid": "ccc"}), false, 3),
        ];
        let report = analyze_trajectory(&entries);
        let tool = &report.tools[0];
        assert!(tool.volatile_fields.contains(&"sid".to_string()));
        assert!(!tool.volatile_fields.contains(&"q".to_string()));
    }

    #[test]
    fn detects_oscillation() {
        let entries = vec![
            entry("search", json!({"q": "a"}), false, 1),
            entry("write_file", json!({"path": "x"}), false, 2),
            entry("search", json!({"q": "b"}), false, 3),
            entry("write_file", json!({"path": "y"}), false, 4),
            entry("search", json!({"q": "c"}), false, 5),
            entry("write_file", json!({"path": "z"}), false, 6),
        ];
        let report = analyze_trajectory(&entries);
        // Both tools should have oscillation detected
        let search = report.tools.iter().find(|t| t.name == "search").unwrap();
        assert!(search.oscillation_detected);
    }

    #[test]
    fn high_error_rate_tightens_threshold() {
        let entries = vec![
            entry("failing_tool", json!({"x": 1}), true, 1),
            entry("failing_tool", json!({"x": 1}), true, 2),
            entry("failing_tool", json!({"x": 1}), true, 3),
            entry("failing_tool", json!({"x": 1}), true, 4),
            entry("failing_tool", json!({"x": 1}), true, 5),
        ];
        let report = analyze_trajectory(&entries);
        let tool = &report.tools[0];
        // High error rate + lots of repeats → tight threshold
        assert!(tool.suggested_max_repeats <= 3);
    }

    #[test]
    fn productive_sequence_loosens_threshold() {
        let entries: Vec<TrajectoryEntry> = (0..10)
            .map(|i| entry("editor", json!({"line": i}), false, i as u64))
            .collect();
        let report = analyze_trajectory(&entries);
        let tool = &report.tools[0];
        // Long productive sequence (10 unique calls) → relaxed threshold
        assert!(tool.suggested_max_repeats >= 5);
        assert_eq!(tool.max_productive_sequence, 10);
    }

    #[test]
    fn policy_roundtrip() {
        let entries = vec![
            entry("read_file", json!({"path": "main.rs"}), false, 1),
            entry("read_file", json!({"path": "main.rs"}), false, 2),
        ];
        let report = analyze_trajectory(&entries);
        let policy = LearnedPolicy {
            sensitivity: "Default".into(),
            tools: report
                .tools
                .iter()
                .map(|t| LearnedToolConfig {
                    name: t.name.clone(),
                    trajectory_gate: LearnedTrajectoryGate {
                        max_repeats: t.suggested_max_repeats,
                        volatile_fields: t.volatile_fields.clone(),
                    },
                    stats: Some(ToolStats {
                        calls: t.call_count,
                        repeats: t.repeat_count,
                        errors: t.error_count,
                        error_rate: t.error_rate,
                    }),
                })
                .collect(),
        };

        let yaml = serde_yml::to_string(&policy).unwrap();
        assert!(yaml.contains("read_file"));

        // Roundtrip
        let parsed: LearnedPolicy = serde_yml::from_str(&yaml).unwrap();
        assert_eq!(parsed.tools.len(), 1);
        assert_eq!(parsed.tools[0].name, "read_file");
    }
}
