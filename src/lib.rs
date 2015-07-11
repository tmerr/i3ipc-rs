//! A library for controlling i3-wm through its ipc interface.

pub mod reply;

pub enum Event {
    // none of these are complete
    Workspace,
    Output,
    Mode,
    Window,
    BarConfigUpdate,
    Binding
}

struct I3Connection;

impl I3Connection {
    /// Establishes an IPC connection to i3.
    fn new() -> I3Connection {
        panic!("not implemented");
    }

    /// The payload of the message is a command for i3 (like the commands you can bind to keys
    /// in the configuration file) and will be executed directly after receiving it.
    fn command(string: &str) -> Result<reply::Command, String> {
        panic!("not implemented");
    }

    /// Gets the current workspaces.
    fn get_workspaces() -> Result<reply::Workspaces, String> {
        panic!("not implemented");
    }

    /// Subscribes your connection to certain events.
    fn subscribe() -> Result<reply::Subscribe, String> {
        panic!("not implemented");
    }

    /// Gets the current outputs.
    fn get_outputs() -> Result<reply::Outputs, String> {
        panic!("not implemented");
    }

    /// Gets the layout tree. i3 uses a tree as data structure which includes every container.
    fn get_tree() -> Result<reply::Tree, String> {
        panic!("not implemented");
    }

    /// Gets a list of marks (identifiers for containers to easily jump to them later).
    fn get_marks() -> Result<reply::Marks, String> {
        panic!("not implemented");
    }

    /// Gets an array with all configured bar IDs.
    fn get_bar_ids() -> Result<reply::BarIds, String> {
        panic!("not implemented");
    }

    /// Gets the configuration of the workspace bar with the given ID.
    fn get_bar_config(id: String) -> Result<reply::BarConfig, String> {
        panic!("not implemented");
    }

    /// Gets the version of i3. The reply will include the major, minor, patch and human-readable
    /// version.
    fn get_version() -> Result<reply::Version, String> {
        panic!("not implemented");
    }
}


#[cfg(test)]
mod test {
    #[test]
    fn command() {
        panic!("not implemented");
    }

    #[test]
    fn get_worksplaces() {
        panic!("not implemented");
    }

    #[test]
    fn subscribe() {
        panic!("not implemented");
    }

    #[test]
    fn get_outputs() {
        panic!("not implemented");
    }

    #[test]
    fn get_tree() {
        panic!("not implemented");
    }

    #[test]
    fn get_marks() {
        panic!("not implemented");
    }

    #[test]
    fn get_bar_config() {
        panic!("not implemented");
    }

    #[test]
    fn get_version() {
        panic!("not implemented");
    }
}
