//! Abstractions for the replies passed back from i3.

use std::collections::HashMap;

/// The outcome of a single command.
#[derive(Debug)]
pub struct CommandOutcome {
    /// Whether the command was successful.
    pub success: bool,
    /// A human-readable error message.
    pub error: Option<String>,
}

/// The reply to the `command` request.
#[derive(Debug)]
pub struct Command {
    /// A list of `CommandOutcome` structs; one for each command that was parsed.
    pub outcomes: Vec<CommandOutcome>,
}

/// A single workspace.
#[derive(Debug)]
pub struct Workspace {
    /// The logical number of the workspace. Corresponds to the command to switch to this
    /// workspace. For named workspaces, this will be -1.
    pub num: i32,
    /// The name of this workspace (by default num+1), as changed by the user.
    pub name: String,
    /// Whether this workspace is currently visible on an output (multiple workspaces can be
    /// visible at the same time).
    pub visible: bool,
    /// Whether this workspace currently has the focus (only one workspace can have the focus
    /// at the same time).
    pub focused: bool,
    /// Whether a window on this workspace has the "urgent" flag set.
    pub urgent: bool,
    /// The rectangle of this workspace (equals the rect of the output it is on), consists of
    /// x, y, width, height.
    pub rect: (i32, i32, i32, i32),
    /// The video output this workspace is on (LVDS1, VGA1, …).
    pub output: String,
}

/// The reply to the `get_workspaces` request.
#[derive(Debug)]
pub struct Workspaces {
    /// A list of workspaces.
    pub workspaces: Vec<Workspace>,
}

/// The reply to the `subscribe` request.
#[derive(Debug)]
pub struct Subscribe {
    /// Indicates whether the subscription was successful (the default) or whether a JSON
    /// parse error occurred.
    pub success: bool,
}

#[cfg(feature = "sway-1-1")]
#[derive(Debug)]
/// A mode for sway
pub struct Mode {
    pub width: i32,
    pub height: i32,
    pub refresh: i32,
}

/// A single output (display)
#[derive(Debug)]
pub struct Output {
    /// The name of this output (as seen in xrandr).
    pub name: String,
    #[cfg(feature = "sway-1-1")]
    /// Make of the Output
    pub make: String,
    #[cfg(feature = "sway-1-1")]
    /// Make of the Output
    pub model: String,
    #[cfg(feature = "sway-1-1")]
    /// Make of the Output
    pub serial: String,
    /// Whether the output is currently active (has a valid mode).
    pub active: bool,
    #[cfg(feature = "sway-1-1")]
    /// DPMS status of the output
    pub dpms: bool,
    /// Whether the output is currently the primary output.
    pub primary: bool,
    #[cfg(feature = "sway-1-1")]
    /// Scale of the output
    pub scale: Option<f64>,
    #[cfg(feature = "sway-1-1")]
    /// Subpixel hinting for the output
    pub subpixel_hinting: Option<String>,
    #[cfg(feature = "sway-1-1")]
    /// Transform for the output
    pub transform: Option<String>,
    /// The name of the current workspace that is visible on this output. None if the output is
    /// not active.
    pub current_workspace: Option<String>,
    #[cfg(feature = "sway-1-1")]
    /// Modes for the output
    pub modes: Vec<Mode>,
    #[cfg(feature = "sway-1-1")]
    /// current mode for the output
    pub current_mode: Option<Mode>,
    /// The rectangle of this output (equals the rect of the output it is on), consists of
    /// x, y, width, height.
    pub rect: (i32, i32, i32, i32),
}

/// The reply to the `get_outputs` request.
#[derive(Debug)]
pub struct Outputs {
    /// A list of outputs (displays)
    pub outputs: Vec<Output>,
}

#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub enum WindowProperty {
    Title,
    Instance,
    Class,
    WindowRole,
    TransientFor,
    Machine,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum NodeType {
    Root,
    Output,
    Con,
    FloatingCon,
    Workspace,
    DockArea,
    /// A NodeType we don't support yet.
    Unknown,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum NodeBorder {
    Normal,
    None,
    Pixel,
    /// A NodeBorder we don't support yet.
    Unknown,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum NodeLayout {
    SplitH,
    SplitV,
    Stacked,
    Tabbed,
    DockArea,
    Output,
    /// A NodeLayout we don't support yet.
    Unknown,
}

/// The reply to the `get_tree` request.
#[derive(Debug, Clone)]
pub struct Node {
    /// List of child node IDs (see `nodes`, `floating_nodes` and `id`) in focus order. Traversing
    /// the tree by following the first entry in this array will result in eventually reaching the
    /// one node with `focused` set to true.
    pub focus: Vec<i64>,

    /// The child nodes of this container.
    pub nodes: Vec<Node>,

    /// The child floating nodes of this container.
    pub floating_nodes: Vec<Node>,

    /// The internal ID (actually a C pointer value) of this container. Do not make any
    /// assumptions about it. You can use it to (re-)identify and address containers when
    /// talking to i3.
    pub id: i64,

    /// The internal name of this container. For all containers which are part of the tree
    /// structure down to the workspace contents, this is set to a nice human-readable name of
    /// the container. For containers that have an X11 window, the content is the title
    /// (_NET_WM_NAME property) of that window. For all other containers, the content is not
    /// defined (yet).
    pub name: Option<String>,

    /// Type of this container. Can be one of "root", "output", "con", "floating_con",
    /// "workspace" or "dockarea".
    pub nodetype: NodeType,

    /// Can be either "normal", "none" or "1pixel", dependending on the container’s border
    /// style.
    pub border: NodeBorder,

    /// Number of pixels of the border width.
    pub current_border_width: i32,

    /// Can be either "splith", "splitv", "stacked", "tabbed", "dockarea" or "output". Other values
    /// might be possible in the future, should we add new layouts.
    pub layout: NodeLayout,

    /// The percentage which this container takes in its parent. A value of null means that the
    /// percent property does not make sense for this container, for example for the root
    /// container.
    pub percent: Option<f64>,

    /// The (x, y, width, height) absolute display coordinates for this container. Display
    /// coordinates means that when you have two 1600x1200 monitors on a single X11 Display
    /// (the standard way), the coordinates of the first window on the second monitor are
    /// (1600, 0, 1600, 1200).
    pub rect: (i32, i32, i32, i32),

    /// The (x, y, width, height) coordinates of the actual client window inside its container.
    /// These coordinates are  relative to the container and do not include the window
    /// decoration (which is actually rendered on the parent container). So for example, when
    /// using the default layout, you will have a 2 pixel border on each side, making the
    /// window_rect (2, 0, 632, 366).
    pub window_rect: (i32, i32, i32, i32),

    /// The (x, y, width, height) coordinates of the window decoration inside its container.
    /// These coordinates are relative to the container and do not include the actual client
    /// window.
    pub deco_rect: (i32, i32, i32, i32),

    /// The original geometry the window specified when i3 mapped it. Used when switching a
    /// window to floating mode, for example.
    pub geometry: (i32, i32, i32, i32),

    /// The X11 window ID of the actual client window inside this container. This field is set
    /// to null for split containers or otherwise empty containers. This ID corresponds to what
    /// xwininfo(1) and other X11-related tools display (usually in hex).
    pub window: Option<i32>,

    /// X11 window properties title, instance, class, window_role and transient_for.
    pub window_properties: Option<HashMap<WindowProperty, String>>,

    /// Whether this container (window, split container, floating container or workspace) has the
    /// urgency hint set, directly or indirectly. All parent containers up until the workspace
    /// container will be marked urgent if they have at least one urgent child.
    pub urgent: bool,

    /// Whether this container is currently focused.
    pub focused: bool,
}

/// The reply to the `get_marks` request.
///
/// Consists of a single vector of strings for each container that has a mark. A mark can only
/// be set on one container, so the vector is unique. The order of that vector is undefined. If
/// no window has a mark the response will be an empty vector.
#[derive(Debug)]
pub struct Marks {
    pub marks: Vec<String>,
}

/// The reply to the `get_bar_ids` request.
///
/// This can be used by third-party workspace bars (especially i3bar, but others are free to
/// implement compatible alternatives) to get the bar block configuration from i3.
#[derive(Debug)]
pub struct BarIds {
    /// A vector of configured bar IDs.
    pub ids: Vec<String>,
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum ColorableBarPart {
    /// Background color of the bar.
    Background,

    /// Text color to be used for the statusline.
    Statusline,

    /// Text color to be used for the separator.
    Separator,

    /// Background color of the bar on the currently focused monitor output.
    #[cfg(feature = "i3-4-12")]
    #[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-12")))]
    FocusedBackground,

    /// Text color to be used for the statusline on the currently focused
    /// monitor output.
    #[cfg(feature = "i3-4-12")]
    #[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-12")))]
    FocusedStatusline,

    /// Text color to be used for the separator on the currently focused
    /// monitor output.
    #[cfg(feature = "i3-4-12")]
    #[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-12")))]
    FocusedSeparator,

    /// Text color for a workspace button when the workspace has focus.
    FocusedWorkspaceText,

    /// Background color for a workspace button when the workspace has focus.
    FocusedWorkspaceBg,

    /// Border color for a workspace button when the workspace has focus.
    FocusedWorkspaceBorder,

    /// Text color for a workspace button when the workspace is active (visible) on some
    /// output, but the focus is on another one. You can only tell this apart from the
    /// focused workspace when you are using multiple monitors.
    ActiveWorkspaceText,

    /// Background color for a workspace button when the workspace is active (visible) on some
    /// output, but the focus is on another one. You can only tell this apart from the
    /// focused workspace when you are using multiple monitors.
    ActiveWorkspaceBg,

    /// Border color for a workspace button when the workspace is active (visible) on some
    /// output, but the focus is on another one. You can only tell this apart from the
    /// focused workspace when you are using multiple monitors.
    ActiveWorkspaceBorder,

    /// Text color for a workspace button when the workspace does not have focus and is not
    /// active (visible) on any output. This will be the case for most workspaces.
    InactiveWorkspaceText,

    /// Background color for a workspace button when the workspace does not have focus and is
    /// not active (visible) on any output. This will be the case for most workspaces.
    InactiveWorkspaceBg,

    /// Border color for a workspace button when the workspace does not have focus and is
    /// not active (visible) on any output. This will be the case for most workspaces.
    InactiveWorkspaceBorder,

    /// Text color for workspaces which contain at least one window with the urgency hint set.
    UrgentWorkspaceText,

    /// Background color for workspaces which contain at least one window with the urgency hint
    /// set.
    UrgentWorkspaceBg,

    /// Border color for workspaces which contain at least one window with the urgency hint set.
    UrgentWorkspaceBorder,

    /// Text color for the binding mode indicator.
    BindingModeText,

    /// Background color for the binding mode indicator.
    BindingModeBg,

    /// Border color for the binding mode indicator.
    BindingModeBorder,

    /// A ColorableBarPart we don't support yet.
    Unknown,
}

/// The reply to the `get_bar_config` request.
///
/// This can be used by third-party workspace bars (especially i3bar, but others are free to
/// implement compatible alternatives) to get the bar block configuration from i3.
#[derive(Debug)]
pub struct BarConfig {
    /// The ID for this bar. Included in case you request multiple configurations and want to
    /// differentiate the different replies.
    pub id: String,

    /// Either dock (the bar sets the dock window type) or hide (the bar does not show unless a
    /// specific key is pressed).
    pub mode: String,

    /// Either bottom or top at the moment.
    pub position: String,

    /// Command which will be run to generate a statusline. Each line on stdout of this command
    /// will be displayed in the bar. At the moment, no formatting is supported.
    pub status_command: String,

    /// The font to use for text on the bar.
    pub font: String,

    /// Display workspace buttons or not? Defaults to true.
    pub workspace_buttons: bool,

    /// Display the mode indicator or not? Defaults to true.
    pub binding_mode_indicator: bool,

    /// Should the bar enable verbose output for debugging? Defaults to false.
    pub verbose: bool,

    /// Contains key/value pairs of colors. Each value is a color code in hex, formatted
    /// \#rrggbb (like in HTML).
    pub colors: HashMap<ColorableBarPart, String>,
}

/// The reply to the `get_version` request.
#[derive(Debug)]
pub struct Version {
    /// The major version of i3, such as 4.
    pub major: i32,

    /// The minor version of i3, such as 2. Changes in the IPC interface (new features) will
    /// only occur with new minor (or major) releases. However, bugfixes might be introduced in
    /// patch releases, too.
    pub minor: i32,

    /// The patch version of i3, such as 1 (when the complete version is 4.2.1). For versions
    /// such as 4.2, patch will be set to 0.
    pub patch: i32,

    /// A human-readable version of i3 containing the precise git version, build date and
    /// branch name. When you need to display the i3 version to your users, use the
    /// human-readable version whenever possible (since this is what i3 --version displays,
    /// too).
    pub human_readable: String,

    /// The current config path.
    pub loaded_config_file_name: String,
}

/// The reply to the `get_binding_modes` request.
#[cfg(feature = "i3-4-13")]
#[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-13")))]
#[derive(Debug)]
pub struct BindingModes {
    /// A vector of all currently configured binding modes.
    pub modes: Vec<String>,
}

/// The reply to the `get_config` request.
#[cfg(feature = "i3-4-14")]
#[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-14")))]
#[derive(Debug)]
pub struct Config {
    /// A string containing the config file as loaded by i3 most recently.
    pub config: String,
}
