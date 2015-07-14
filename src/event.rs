//! Abstractions for the events passed back from i3.

use reply;
use serde;
use std::str::FromStr;

use event::inner::*;

/// An event passed back from i3.
pub enum Event {
    WorkspaceEvent(WorkspaceEventInfo),
    OutputEvent(OutputEventInfo),
    ModeEvent(ModeEventInfo),
    WindowEvent(WindowEventInfo),
    BarConfigEvent(BarConfigEventInfo),
    BindingEvent(BindingEventInfo),
}

/// Data for `WorkspaceEvent`.
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
    type Err = serde::json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        panic!("not implemented");
    }
}


/// Data for `OutputEvent`.
pub struct OutputEventInfo {
    /// The type of change.
    pub change: OutputChange
}

impl FromStr for OutputEventInfo {
    type Err = serde::json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        panic!("not implemented");
    }
}

/// Data for `ModeEvent`.
pub struct ModeEventInfo {
    /// The name of current mode in use. It is the same as specified in config when creating a
    /// mode. The default mode is simply named default.
    pub change: String
}

impl FromStr for ModeEventInfo {
    type Err = serde::json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        panic!("not implemented");
    }
}

/// Data for `WindowEvent`.
pub struct WindowEventInfo {
    /// Indicates the type of change
    pub change: WindowChange,
    pub container: reply::Node
}

impl FromStr for WindowEventInfo {
    type Err = serde::json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        panic!("not implemented");
    }
}

/// Data for `BarConfigEvent`.
pub struct BarConfigEventInfo {
    /// The new i3 bar configuration.
    pub bar_config: reply::BarConfig
}

impl FromStr for BarConfigEventInfo {
    type Err = serde::json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        panic!("not implemented");
    }
}

/// Data for `BindingEvent`.
///
/// Reports on the details of a binding that ran a command because of user input.
pub struct BindingEventInfo {
    /// Indicates what sort of binding event was triggered (right now it will always be "run" but
    /// that may be expanded in the future).
    pub change: BindingChange,
    pub binding: Binding
}

impl FromStr for BindingEventInfo {
    type Err = serde::json::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        panic!("not implemented");
    }
}

/// Less important types
pub mod inner {
    /// The kind of workspace change.
    pub enum WorkspaceChange {
        Focus,
        Init,
        Empty,
        Urgent
    }

    /// The kind of output change.
    pub enum OutputChange {
        Unspecified
    }

    /// The kind of window change.
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
        Urgent
    }

    /// Either keyboard or mouse.
    pub enum InputType {
        Keyboard,
        Mouse
    }

    /// Contains details about the binding that was run.
    pub struct Binding {
        /// The i3 command that is configured to run for this binding.
        pub command: String,

        /// The modifier keys that were configured with this binding.
        pub mods: Vec<String>,

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
    pub enum BindingChange {
        Run
    }
}
