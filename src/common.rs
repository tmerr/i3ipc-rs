//! Some common code used by both the event and reply modules.
use reply;
use serde_json as json;
use std::collections::HashMap;

/// Recursively build the tree of containers from the given json value.
pub fn build_tree(val: &json::Value) -> reply::Node {
    reply::Node {
        focus: match val.get("focus") {
            Some(xs) => xs
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap())
                .collect(),
            None => vec![],
        },
        nodes: match val.get("nodes") {
            Some(nds) => nds
                .as_array()
                .unwrap()
                .iter()
                .map(|n| build_tree(n))
                .collect(),
            None => vec![],
        },
        floating_nodes: match val.get("floating_nodes") {
            Some(nds) => nds
                .as_array()
                .unwrap()
                .iter()
                .map(|n| build_tree(n))
                .collect(),
            None => vec![],
        },
        id: val.get("id").unwrap().as_i64().unwrap(),
        name: match val.get("name") {
            Some(n) => match n.as_str() {
                Some(s) => Some(s.to_owned()),
                None => None,
            },
            None => None,
        },
        nodetype: match val.get("type").unwrap().as_str().unwrap() {
            "root" => reply::NodeType::Root,
            "output" => reply::NodeType::Output,
            "con" => reply::NodeType::Con,
            "floating_con" => reply::NodeType::FloatingCon,
            "workspace" => reply::NodeType::Workspace,
            "dockarea" => reply::NodeType::DockArea,
            other => {
                warn!(target: "i3ipc", "Unknown NodeType {}", other);
                reply::NodeType::Unknown
            }
        },
        border: match val.get("border").unwrap().as_str().unwrap() {
            "normal" => reply::NodeBorder::Normal,
            "none" => reply::NodeBorder::None,
            "pixel" => reply::NodeBorder::Pixel,
            other => {
                warn!(target: "i3ipc", "Unknown NodeBorder {}", other);
                reply::NodeBorder::Unknown
            }
        },
        current_border_width: val.get("current_border_width").unwrap().as_i64().unwrap() as i32,
        layout: match val.get("layout").unwrap().as_str().unwrap() {
            "splith" => reply::NodeLayout::SplitH,
            "splitv" => reply::NodeLayout::SplitV,
            "stacked" => reply::NodeLayout::Stacked,
            "tabbed" => reply::NodeLayout::Tabbed,
            "dockarea" => reply::NodeLayout::DockArea,
            "output" => reply::NodeLayout::Output,
            other => {
                warn!(target: "i3ipc", "Unknown NodeLayout {}", other);
                reply::NodeLayout::Unknown
            }
        },
        percent: match *val.get("percent").unwrap() {
            json::Value::Number(ref f) => Some(f.as_f64().unwrap()),
            json::Value::Null => None,
            _ => unreachable!(),
        },
        rect: build_rect(val.get("rect").unwrap()),
        window_rect: build_rect(val.get("window_rect").unwrap()),
        deco_rect: build_rect(val.get("deco_rect").unwrap()),
        geometry: build_rect(val.get("geometry").unwrap()),
        window: match val.get("window").unwrap().clone() {
            json::Value::Number(i) => Some(i.as_i64().unwrap() as i32),
            json::Value::Null => None,
            _ => unreachable!(),
        },
        window_properties: build_window_properties(val.get("window_properties")),
        urgent: val.get("urgent").unwrap().as_bool().unwrap(),
        focused: val.get("focused").unwrap().as_bool().unwrap(),
    }
}

pub fn build_window_properties(
    j: Option<&json::Value>,
) -> Option<HashMap<reply::WindowProperty, String>> {
    match j {
        None => None,
        Some(props) => {
            let properties = props.as_object().unwrap();
            let mut map = HashMap::new();
            for (key, val) in properties {
                let window_property = match key.as_ref() {
                    "class" => Some(reply::WindowProperty::Class),
                    "instance" => Some(reply::WindowProperty::Instance),
                    "window_role" => Some(reply::WindowProperty::WindowRole),
                    "title" => Some(reply::WindowProperty::Title),
                    "transient_for" => Some(reply::WindowProperty::TransientFor),
                    "machine" => Some(reply::WindowProperty::Machine),
                    other => {
                        warn!(target: "i3ipc", "Unknown WindowProperty {}", other);
                        None
                    }
                };
                if let Some(window_property) = window_property {
                    map.insert(
                        window_property,
                        val.as_str().unwrap_or_default().to_string(),
                    );
                }
            }
            Some(map)
        }
    }
}

pub fn build_rect(jrect: &json::Value) -> (i32, i32, i32, i32) {
    let x = jrect.get("x").unwrap().as_i64().unwrap() as i32;
    let y = jrect.get("y").unwrap().as_i64().unwrap() as i32;
    let width = jrect.get("width").unwrap().as_i64().unwrap() as i32;
    let height = jrect.get("height").unwrap().as_i64().unwrap() as i32;
    (x, y, width, height)
}

pub fn build_bar_config(j: &json::Value) -> reply::BarConfig {
    reply::BarConfig {
        id: j.get("id").unwrap().as_str().unwrap().to_owned(),
        mode: j.get("mode").unwrap().as_str().unwrap().to_owned(),
        position: j.get("position").unwrap().as_str().unwrap().to_owned(),
        status_command: j
            .get("status_command")
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned(),
        font: j.get("font").unwrap().as_str().unwrap().to_owned(),
        workspace_buttons: j.get("workspace_buttons").unwrap().as_bool().unwrap(),
        binding_mode_indicator: j.get("binding_mode_indicator").unwrap().as_bool().unwrap(),
        verbose: j.get("verbose").unwrap().as_bool().unwrap(),
        colors: {
            let colors = j.get("colors").unwrap().as_object().unwrap();
            let mut map = HashMap::new();
            for c in colors.keys() {
                let enum_key = match c.as_ref() {
                    "background" => reply::ColorableBarPart::Background,
                    "statusline" => reply::ColorableBarPart::Statusline,
                    "separator" => reply::ColorableBarPart::Separator,

                    #[cfg(feature = "i3-4-12")]
                    "focused_background" => reply::ColorableBarPart::FocusedBackground,

                    #[cfg(feature = "i3-4-12")]
                    "focused_statusline" => reply::ColorableBarPart::FocusedStatusline,

                    #[cfg(feature = "i3-4-12")]
                    "focused_separator" => reply::ColorableBarPart::FocusedSeparator,

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
                    other => {
                        warn!(target: "i3ipc", "Unknown ColorableBarPart {}", other);
                        reply::ColorableBarPart::Unknown
                    }
                };
                let hex = colors.get(c).unwrap().as_str().unwrap().to_owned();
                map.insert(enum_key, hex);
            }
            map
        },
    }
}

#[cfg(feature = "sway-1-1")]
pub fn build_modes(j: &json::Value) -> Vec<reply::Mode> {
    let mut res: Vec<reply::Mode>= Vec::new();
    for mode in j.as_array().unwrap() {
        res.push(build_mode(mode))
    }
    res
}

#[cfg(feature = "sway-1-1")]
pub fn build_mode(jmode: &json::Value) -> reply::Mode {
    let width = jmode.get("width").unwrap().as_i64().unwrap() as i32;
    let height = jmode.get("height").unwrap().as_i64().unwrap() as i32;
    let refresh = jmode.get("refresh").unwrap().as_i64().unwrap() as i32;
    reply::Mode {
        width: width,
        height: height,
        refresh: refresh
    }
}
