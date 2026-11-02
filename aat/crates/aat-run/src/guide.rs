//! What the agent is told about reporting progress, appended to every body prompt (as
//! pi-automations' `notesGuide`/`progressGuide`, with aat's verbs and variables).

use aat_spec::{flow_order, parse_chart};

// Bodies ask for receipts in plain words; this ties those words to the commands.
const RECEIPTS_LINE: &str = "When the instructions ask you to record what you published or to leave a message, use these commands.";

/// One sentence: leave the reader one message with links before finishing.
fn message_line() -> &'static str {
    "Before you finish, leave the reader ONE message: `aat message \"what happened / what needs you\" --link \"label=https://…\"`. Links matter most; a later message replaces the earlier one."
}

/// For a run with no chart: notes and a message only.
pub fn notes_guide() -> String {
    [
        "<notes>",
        RECEIPTS_LINE,
        "If you publish anything or leave something for the reader, record it with `aat note \"what it is\" --link \"label=https://…\"` (AAT_RUN is already set).",
        message_line(),
        "If `aat` fails, carry on.",
        "</notes>",
    ]
    .join("\n")
}

/// For a run tracked on its flowchart: every verb, and the nodes in the order the run goes.
pub fn progress_guide(chart: &str, strict: bool) -> String {
    let ids: Vec<String> = flow_order(&parse_chart(chart))
        .into_iter()
        .map(|n| n.id)
        .collect();
    let nodes = format!("Nodes: {} (or their exact labels).", ids.join(", "));
    let mut lines: Vec<&str> = vec![
        "<progress>",
        "This run is tracked on its flowchart. Report where you are with `aat` (AAT_RUN is already set):",
        "- `aat step <node> [-m \"text\"] [--status active|waiting|done|failed|skipped]` — set one node (default active)",
        "- `aat next <to> --from <node> [-m \"text\"]` — finish <node> and start <to>",
        "- `aat say \"text\" [--node <node>]` — add an update to a node",
        "- `aat note \"text\" --link \"label=https://…\"` — a note on the run's record, with the links you published",
        "- `aat status` — where the run is; any command takes `--json`",
        &nodes,
    ];
    if strict {
        lines.push(
            "Each `##` section is one step: start it with `aat next <id> --from <the step before>`, following the chart's arrows.",
        );
    }
    lines.extend([
        "Subagents inherit AAT_RUN; set AAT_NODE=<its node> for each so its `aat say`/`aat next` land there.",
        RECEIPTS_LINE,
        message_line(),
        "If `aat` fails, carry on.",
        "</progress>",
    ]);
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHART: &str = "flowchart TD\n  pre0[\"Pick\"] --> body[\"Verify\"]\n  body --> read[\"Read\"]\n  read --> reproduce\n  read -->|no browser| draft\n  reproduce --> draft\n  draft --> post0\n";

    #[test]
    fn lists_the_nodes_in_flow_order() {
        let g = progress_guide(CHART, false);
        assert!(g.starts_with("<progress>\n"));
        assert!(g.ends_with("\n</progress>"));
        assert!(
            g.contains("Nodes: pre0, body, read, reproduce, draft, post0 (or their exact labels).")
        );
        for verb in ["step", "next", "say", "note", "status"] {
            assert!(g.contains(&format!("`aat {verb}")), "{verb}");
        }
        assert!(g.contains("`aat message "));
        assert!(!g.contains("pa "));
    }

    #[test]
    fn adds_one_line_and_only_in_strict_mode() {
        let strict: Vec<String> = progress_guide(CHART, true)
            .lines()
            .map(String::from)
            .collect();
        let loose: Vec<String> = progress_guide(CHART, false)
            .lines()
            .map(String::from)
            .collect();
        assert_eq!(strict.len(), loose.len() + 1);
        let extra: Vec<&String> = strict.iter().filter(|l| !loose.contains(l)).collect();
        assert_eq!(
            extra,
            [
                "Each `##` section is one step: start it with `aat next <id> --from <the step before>`, following the chart's arrows."
            ]
        );
    }

    #[test]
    fn the_notes_guide_names_note_and_message() {
        let g = notes_guide();
        assert!(g.starts_with("<notes>\n") && g.ends_with("\n</notes>"));
        assert!(g.contains("`aat note "));
        assert!(g.contains("`aat message "));
        assert!(!g.contains("<progress>"));
    }
}
