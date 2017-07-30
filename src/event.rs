//! Abstractions for the events passed back from i3.

use reply;
use serde_json as json;
use std::str::FromStr;
use common;

use event::inner::*;

/// An event passed back from i3.
#[derive(Debug)]
pub enum Event {
    WorkspaceEvent(WorkspaceEventInfo),
    OutputEvent(OutputEventInfo),
    ModeEvent(ModeEventInfo),
    WindowEvent(WindowEventInfo),
    BarConfigEvent(BarConfigEventInfo),
    BindingEvent(BindingEventInfo),
}

/// Data for `WorkspaceEvent`.
#[derive(Debug)]
pub struct WorkspaceEventInfo {
    /// The type of change.
    pub change: WorkspaceChange,
    /// Will be `Some` if the type of event affects the workspace.
    pub current: Option<reply::Node>,
    /// Will be `Some` only when `change == Focus` *and* there was a previous workspace.
    /// Note that if the previous workspace was empty it will get destroyed when switching, but
    /// will still appear here.
    pub old: Option<reply::Node>
}

impl FromStr for WorkspaceEventInfo {
    type Err = json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: json::Value = try!(json::from_str(s));
        Ok(WorkspaceEventInfo {
            change: match val.find("change").unwrap().as_string().unwrap().as_ref() {
                "focus" => WorkspaceChange::Focus,
                "init" => WorkspaceChange::Init,
                "empty" => WorkspaceChange::Empty,
                "urgent" => WorkspaceChange::Urgent,
                _ => unreachable!()
            },
            current: match val.find("current").unwrap().clone() {
                json::Value::Null => None,
                val => Some(common::build_tree(&val))
            },
            old: match val.find("old") {
                Some(o) => match o.clone() {
                    json::Value::Null => None,
                    val => Some(common::build_tree(&val))
                },
                None => None
            }
        })
    }
}

/// Data for `OutputEvent`.
#[derive(Debug)]
pub struct OutputEventInfo {
    /// The type of change.
    pub change: OutputChange
}

impl FromStr for OutputEventInfo {
    type Err = json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: json::Value = try!(json::from_str(s));
        Ok(OutputEventInfo {
            change: match val.find("change").unwrap().as_string().unwrap().as_ref() {
                "unspecified" => OutputChange::Unspecified,
                _ => unreachable!()
            }
        })
    }
}

/// Data for `ModeEvent`.
#[derive(Debug)]
pub struct ModeEventInfo {
    /// The name of current mode in use. It is the same as specified in config when creating a
    /// mode. The default mode is simply named default.
    pub change: String
}

impl FromStr for ModeEventInfo {
    type Err = json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: json::Value = try!(json::from_str(s));
        Ok(ModeEventInfo {
            change: val.find("change").unwrap().as_string().unwrap().to_owned()
        })
    }
}

/// Data for `WindowEvent`.
#[derive(Debug)]
pub struct WindowEventInfo {
    /// Indicates the type of change
    pub change: WindowChange,
    /// The window's parent container. Be aware that for the "new" event, the container will hold
    /// the initial name of the newly reparented window (e.g. if you run urxvt with a shell that
    /// changes the title, you will still at this point get the window title as "urxvt").
    pub container: reply::Node
}

impl FromStr for WindowEventInfo {
    type Err = json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: json::Value = try!(json::from_str(s));
        Ok(WindowEventInfo {
            change: match val.find("change").unwrap().as_string().unwrap().as_ref() {
                "new" => WindowChange::New,
                "close" => WindowChange::Close,
                "focus" => WindowChange::Focus,
                "title" => WindowChange::Title,
                "fullscreen_mode" => WindowChange::FullscreenMode,
                "move" => WindowChange::Move,
                "floating" => WindowChange::Floating,
                "urgent" => WindowChange::Urgent,
                "mark" => WindowChange::Mark,
                _ => unreachable!()
            },
            container: common::build_tree(val.find("container").unwrap())
        })
    }
}

/// Data for `BarConfigEvent`.
#[derive(Debug)]
pub struct BarConfigEventInfo {
    /// The new i3 bar configuration.
    pub bar_config: reply::BarConfig
}

impl FromStr for BarConfigEventInfo {
    type Err = json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: json::Value = try!(json::from_str(s));
        Ok(BarConfigEventInfo {
            bar_config: common::build_bar_config(&val)
        })
    }
}

/// Data for `BindingEvent`.
///
/// Reports on the details of a binding that ran a command because of user input.
#[derive(Debug)]
pub struct BindingEventInfo {
    /// Indicates what sort of binding event was triggered (right now it will always be "run" but
    /// that may be expanded in the future).
    pub change: BindingChange,
    pub binding: Binding
}

impl FromStr for BindingEventInfo {
    type Err = json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val: json::Value = try!(json::from_str(s));
        let bind = val.find("binding").unwrap();
        Ok(BindingEventInfo {
            change: match val.find("change").unwrap().as_string().unwrap().as_ref() {
                "run" => BindingChange::Run,
                _ => unreachable!()
            },
            binding: Binding {
                command: bind.find("command").unwrap().as_string().unwrap().to_owned(),
                event_state_mask: bind.find("event_state_mask").unwrap()
                         .as_array().unwrap().iter()
                         .map(|m| m.as_string().unwrap().to_owned())
                         .collect(),
                input_code: bind.find("input_code").unwrap().as_i64().unwrap() as i32,
                symbol: match bind.find("symbol").unwrap().clone() {
                    json::Value::String(s) => Some(s),
                    json::Value::Null => None,
                    _ => unreachable!()
                },
                input_type: match bind.find("input_type").unwrap().as_string().unwrap().as_ref() {
                    "keyboard" => InputType::Keyboard,
                    "mouse" => InputType::Mouse,
                    _ => unreachable!()
                }
            }
        })
    }
}

/// Less important types
pub mod inner {
    /// The kind of workspace change.
    #[derive(Debug)]
    pub enum WorkspaceChange {
        Focus,
        Init,
        Empty,
        Urgent
    }

    /// The kind of output change.
    #[derive(Debug)]
    pub enum OutputChange {
        Unspecified
    }

    /// The kind of window change.
    #[derive(Debug)]
    pub enum WindowChange {
        /// The window has become managed by i3.
        New,
        /// The window has closed>.
        Close,
        /// The window has received input focus.
        Focus,
        /// The window's title has changed.
        Title,
        /// The window has entered or exited fullscreen mode.
        FullscreenMode,
        /// The window has changed its position in the tree.
        Move,
        /// The window has transitioned to or from floating.
        Floating,
        /// The window has become urgent or lost its urgent status.
        Urgent,
        /// The window's marks have been modified
        Mark
    }

    /// Either keyboard or mouse.
    #[derive(Debug)]
    pub enum InputType {
        Keyboard,
        Mouse
    }

    /// Contains details about the binding that was run.
    #[derive(Debug)]
    pub struct Binding {
        /// The i3 command that is configured to run for this binding.
        pub command: String,

        /// The group and modifier keys that were configured with this binding.
        pub event_state_mask: Vec<String>,

        /// If the binding was configured with blindcode, this will be the key code that was given for
        /// the binding. If the binding is a mouse binding, it will be the number of times the mouse
        /// button was pressed. Otherwise it will be 0.
        pub input_code: i32,

        /// If this is a keyboard binding that was configured with bindsym, this field will contain the
        /// given symbol. Otherwise it will be None.
        pub symbol: Option<String>,

        /// Will be Keyboard or Mouse depending on whether this was a keyboard or mouse binding.
        pub input_type: InputType
    }

    /// The kind of binding change.
    #[derive(Debug)]
    pub enum BindingChange {
        Run
    }
}
