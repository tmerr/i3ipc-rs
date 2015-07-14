//! Abstractions for the events passed back from i3.

use reply;

/// Indicates the type of change.
pub enum WorkspaceChange {
    Focus,
    Init,
    Empty,
    Urgent
}

/// The data for the workspace event.
pub struct Workspace {
    /// The type of change.
    pub change: WorkspaceChange,
    /// Will be `Some` if the type of event affects the workspace.
    pub current: Option<reply::Workspace>,
    /// Will be `Some` only when `change == Focus` *and* there was a previous workspace.
    /// Note that if the previous workspace was empty it will get destroyed when switching, but
    /// will still appear here.
    pub old: Option<reply::Workspace>
}

/// Indicates the type of change (currently only unspecified).
pub enum OutputChange {
    Unspecified
}

/// The data for the output event.
pub struct Output {
    /// The type of change.
    pub change: OutputChange
}

/// The data for the mode event.
pub struct Mode {
    /// The name of current mode in use. It is the same as specified in config when creating a
    /// mode. The default mode is simply named default.
    pub change: String
}

/// The type of the window change.
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

/// The data for the window event.
pub struct Window {
    /// Indicates the type of change
    pub change: WindowChange,
    pub container: reply::Tree // FIXME: Figure out if the entire tree is passed or just the parent
}

/// The data for the barconfig_update event.
pub type BarConfig = reply::BarConfig;

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

/// The type of change.
pub enum BindingEventChange {
    Run
}

/// This event reports on the details of a binding that ran a command because of user input. The
/// change field indicates what sort of binding event was triggered (right now it will always be
/// "run" but may be expanded in the future).
pub struct BindingEvent {
    /// Indicates what sort of binding event was triggered (right now it will always be "run" but
    /// that may be expanded in the future).
    change: BindingEventChange,

}
