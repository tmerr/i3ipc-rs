//! Some common code used by both the event and reply modules.
use std::collections::HashMap;
use serde::json;
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
        focused: val.find("focused").unwrap().as_boolean().unwrap(),
        undocumented: HashMap::new() // TODO: implement.
    }
}

pub fn build_rect(jrect: &json::Value) -> (i32, i32, i32, i32) {
    let x = jrect.find("x").unwrap().as_i64().unwrap() as i32;
    let y = jrect.find("y").unwrap().as_i64().unwrap() as i32;
    let width = jrect.find("width").unwrap().as_i64().unwrap() as i32;
    let height = jrect.find("height").unwrap().as_i64().unwrap() as i32;
    (x, y, width, height)
}
