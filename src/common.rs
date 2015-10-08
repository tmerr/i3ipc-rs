//! Some common code used by both the event and reply modules.
use std::collections::HashMap;
use serde_json as json;
use reply;

/// Recursively build the tree of containers from the given json value.
pub fn build_tree(val: &json::Value) -> reply::Node {
    reply::Node {
        nodes: match val.find("nodes") {
            Some(nds) => nds.as_array()
                            .unwrap()
                            .iter()
                            .map(|n| build_tree(n))
                            .collect::<Vec<_>>(),
            None => vec![]
        },
        id: val.find("id").unwrap().as_i64().unwrap() as i32,
        name: match val.find("name") {
            Some(n) => match n.as_string() {
                Some(s) => Some(s.to_owned()),
                None => None
            },
            None => None
        },
        nodetype: match val.find("type").unwrap().as_string().unwrap().as_ref() {
            "root" => reply::NodeType::Root,
            "output" => reply::NodeType::Output,
            "con" => reply::NodeType::Con,
            "floating_con" => reply::NodeType::FloatingCon,
            "workspace" => reply::NodeType::Workspace,
            "dockarea" => reply::NodeType::DockArea,
            _ => unreachable!()
        },
        border: match val.find("border").unwrap().as_string().unwrap().as_ref() {
            "normal" => reply::NodeBorder::Normal,
            "none" => reply::NodeBorder::None,
            "1pixel" => reply::NodeBorder::OnePixel,
            _ => unreachable!()
        },
        current_border_width: val.find("current_border_width").unwrap().as_i64().unwrap() as i32,
        layout: match val.find("layout").unwrap().as_string().unwrap().as_ref() {
            "splith" => reply::NodeLayout::SplitH,
            "splitv" => reply::NodeLayout::SplitV,
            "stacked" => reply::NodeLayout::Stacked,
            "tabbed" => reply::NodeLayout::Tabbed,
            "dockarea" => reply::NodeLayout::DockArea,
            "output" => reply::NodeLayout::Output,
            _ => unreachable!()
        },
        percent: match *val.find("percent").unwrap() {
            json::Value::F64(f) => Some(f),
            json::Value::Null => None,
            _ => unreachable!()
        },
        rect: build_rect(val.find("rect").unwrap()),
        window_rect: build_rect(val.find("window_rect").unwrap()),
        deco_rect: build_rect(val.find("deco_rect").unwrap()),
        geometry: build_rect(val.find("geometry").unwrap()),
        window: match val.find("window").unwrap().clone() {
            json::Value::I64(i) => Some(i as i32),
            json::Value::U64(u) => Some(u as i32),
            json::Value::Null => None,
            _ => unreachable!()
        },
        urgent: val.find("urgent").unwrap().as_boolean().unwrap(),
        focused: val.find("focused").unwrap().as_boolean().unwrap()
    }
}

pub fn build_rect(jrect: &json::Value) -> (i32, i32, i32, i32) {
    let x = jrect.find("x").unwrap().as_i64().unwrap() as i32;
    let y = jrect.find("y").unwrap().as_i64().unwrap() as i32;
    let width = jrect.find("width").unwrap().as_i64().unwrap() as i32;
    let height = jrect.find("height").unwrap().as_i64().unwrap() as i32;
    (x, y, width, height)
}

pub fn build_bar_config(j: &json::Value) -> reply::BarConfig {
    reply::BarConfig {
        id: j.find("id").unwrap().as_string().unwrap().to_owned(),
        mode: j.find("mode").unwrap().as_string().unwrap().to_owned(),
        position: j.find("position").unwrap().as_string().unwrap().to_owned(),
        status_command: j.find("status_command").unwrap().as_string().unwrap().to_owned(),
        font: j.find("font").unwrap().as_string().unwrap().to_owned(),
        workspace_buttons: j.find("workspace_buttons").unwrap().as_boolean().unwrap(),
        binding_mode_indicator: j.find("binding_mode_indicator").unwrap().as_boolean().unwrap(),
        verbose: j.find("verbose").unwrap().as_boolean().unwrap(),
        colors: {
            let colors = j.find("colors").unwrap().as_object().unwrap();
            let mut map = HashMap::new();
            for c in colors.keys() {
                let enum_key = match c.as_ref() {
                    "background" => reply::ColorableBarPart::Background,
                    "statusline" => reply::ColorableBarPart::Statusline,
                    "separator" => reply::ColorableBarPart::Separator,
                    "focused_workspace_text" => reply::ColorableBarPart::FocusedWorkspaceText,
                    "focused_workspace_bg" => reply::ColorableBarPart::FocusedWorkspaceBg,
                    "focused_workspace_border" => reply::ColorableBarPart::FocusedWorkspaceBorder,
                    "active_workspace_text" => reply::ColorableBarPart::ActiveWorkspaceText,
                    "active_workspace_bg" => reply::ColorableBarPart::ActiveWorkspaceBg,
                    "active_workspace_border" => reply::ColorableBarPart::ActiveWorkspaceBorder,
                    "inactive_workspace_text" => reply::ColorableBarPart::InactiveWorkspaceText,
                    "inactive_workspace_bg" => reply::ColorableBarPart::InactiveWorkspaceBg,
                    "inactive_workspace_border" => reply::ColorableBarPart::InactiveWorkspaceBorder,
                    "urgent_workspace_text" => reply::ColorableBarPart::UrgentWorkspaceText,
                    "urgent_workspace_bg" => reply::ColorableBarPart::UrgentWorkspaceBg,
                    "urgent_workspace_border" => reply::ColorableBarPart::UrgentWorkspaceBorder,
                    "binding_mode_text" => reply::ColorableBarPart::BindingModeText,
                    "binding_mode_bg" => reply::ColorableBarPart::BindingModeBg,
                    "binding_mode_border" => reply::ColorableBarPart::BindingModeBorder,
                    other => reply::ColorableBarPart::Undocumented(other.to_owned())
                };
                let hex = colors.get(c).unwrap().as_string().unwrap().to_owned();
                map.insert(enum_key, hex);
            }
            map
        }
    }
}
