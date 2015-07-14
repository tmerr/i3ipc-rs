//! A library for controlling i3-wm through its ipc interface.

extern crate unix_socket;
extern crate byteorder;
extern crate serde;

use std::process;
use unix_socket::UnixStream;
use std::io;
use std::io::prelude::*;
use std::collections::HashMap;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use serde::json;
use std::str::FromStr;

mod readhelp;
pub mod reply;
pub mod event;

/// An error while instantiating an I3Connection. Creating an I3Connection involves first getting
/// the i3 socket path, then connecting to the socket. Either part could go wrong, which is why
/// there are two possibilities here.
#[derive(Debug)]
pub enum I3ConnectError {
    /// An error while getting the socket path
    GetSocketPathError(io::Error),
    /// An error while accessing the socket
    SocketError(io::Error)
}

fn get_socket_path() -> io::Result<String> {
    let output = try!(process::Command::new("i3")
                                       .arg("--get-socketpath")
                                       .output());
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
                  .trim_right_matches('\n')
                  .to_owned())
    } else {
        let prefix = "i3 --getsocketpath didn't return 0";
        let error_text = if output.stderr.len() > 0 {
            format!("{}. stderr: {:?}", prefix, output.stderr)
        } else {
            prefix.to_owned()
        };
        let error = io::Error::new(io::ErrorKind::Other, error_text);
        Err(error)
    }
}

trait I3Funcs {
    fn send_i3_message(&mut self, u32, &str) -> io::Result<()>;
    fn receive_i3_message(&mut self) -> io::Result<(u32, String)>;
}

impl I3Funcs for UnixStream {
    fn send_i3_message(&mut self, message_type: u32, payload: &str) -> io::Result<()> {
        let mut bytes = Vec::with_capacity(14 + payload.len());
        bytes.extend("i3-ipc".bytes());                              // 6 bytes
        try!(bytes.write_u32::<LittleEndian>(payload.len() as u32)); // 4 bytes
        try!(bytes.write_u32::<LittleEndian>(message_type));         // 4 bytes
        bytes.extend(payload.bytes());                               // payload.len() bytes
        self.write_all(&bytes[..])
    }

    /// returns a tuple of (message type, payload)
    fn receive_i3_message(&mut self) -> io::Result<(u32, String)> {
        let magic_data = try!(readhelp::read_exact(self, 6));
        let magic_string = String::from_utf8_lossy(&magic_data);
        if magic_string != "i3-ipc" {
            let error_text = format!("unexpected magic string: expected 'i3-ipc' but got {}",
                                      magic_string);
            return Err(io::Error::new(io::ErrorKind::Other, error_text));
        }
        let payload_len = try!(self.read_u32::<LittleEndian>());
        let message_type = try!(self.read_u32::<LittleEndian>());
        let payload_data = try!(readhelp::read_exact(self, payload_len as usize));
        let payload_string = String::from_utf8_lossy(&payload_data).into_owned();
        Ok((message_type, payload_string))
    }
}

/// the msgtype passed in should have its highest order bit stripped
fn build_event(msgtype: u32, payload: &str) -> event::Event {
    match msgtype {
        0 => event::Event::EWorkspace(event::Workspace::from_str(payload).unwrap()),
        1 => event::Event::EOutput(event::Output::from_str(payload).unwrap()),
        2 => event::Event::EMode(event::Mode::from_str(payload).unwrap()),
        3 => event::Event::EWindow(event::Window::from_str(payload).unwrap()),
        4 => event::Event::EBarConfig(event::BarConfig::from_str(payload).unwrap()),
        5 => event::Event::EBindingEvent(event::BindingEvent::from_str(payload).unwrap()),
        _ => unreachable!()
    }
}

pub struct EventIterator<'a> {
    stream: &'a mut UnixStream,
}

impl<'a> Iterator for EventIterator<'a> {
    type Item = io::Result<event::Event>;

    fn next(&mut self) -> Option<Self::Item>{
        let result = self.stream.receive_i3_message();
        if result.is_err() {
            return Some(Err(result.err().unwrap()));
        }
        let (msgint, payload) = result.unwrap();
        // throw out the highest order bit of the message type. it just says it's an event.
        let msgtype = (msgint << 1) >> 1;
        Some(Ok(build_event(msgtype, &payload)))
    }
}

/// Abstraction over an ipc socket to i3. Handles events.
pub struct I3EventListener {
    stream: UnixStream
}

impl I3EventListener {
    /// Establishes the IPC connection.
    pub fn connect() -> Result<I3EventListener, I3ConnectError> {
        return match get_socket_path() {
            Ok(path) => {
                match UnixStream::connect(path) {
                    Ok(stream) => Ok(I3EventListener { stream: stream }),
                    Err(error) => Err(I3ConnectError::SocketError(error))
                }
            }
            Err(error) => Err(I3ConnectError::GetSocketPathError(error))
        }
    }

    /// Subscribes your connection to certain events.
    pub fn subscribe(&self, events: &[event::EventType]) -> io::Result<reply::Subscribe> {
        panic!("not implemented");
    }

    /// Iterates over 
    pub fn events() {
    }
}

/// Abstraction over an ipc socket to i3. Handles messages/replies.
pub struct I3Connection {
    stream: UnixStream
}

impl I3Connection {

    /// Establishes the IPC connection.
    pub fn connect() -> Result<I3Connection, I3ConnectError> {
        return match get_socket_path() {
            Ok(path) => {
                match UnixStream::connect(path) {
                    Ok(stream) => Ok(I3Connection { stream: stream }),
                    Err(error) => Err(I3ConnectError::SocketError(error))
                }
            }
            Err(error) => Err(I3ConnectError::GetSocketPathError(error))
        }
    }

    /// The payload of the message is a command for i3 (like the commands you can bind to keys
    /// in the configuration file) and will be executed directly after receiving it.
    pub fn command(&mut self, string: &str) -> io::Result<reply::Command> {
        try!(self.stream.send_i3_message(0, string));
        let (_, payload) = try!(self.stream.receive_i3_message());

        // assumes valid json
        let j: json::Value = json::from_str(&payload).unwrap();
        let commands = j.as_array().unwrap();
        let vec: Vec<_>
            = commands.iter()
                      .map(|c| 
                           reply::CommandOutcome {
                               success: c.find("success").unwrap().as_boolean().unwrap(),
                               error: match c.find("error") {
                                   Some(val) => Some(val.as_string().unwrap().to_owned()),
                                   None => None
                               }
                           })
                      .collect();

        Ok(reply::Command { outcomes: vec })
    }

    fn unpack_rect(jrect: &json::Value) -> (i32, i32, i32, i32) {
        let x = jrect.find("x").unwrap().as_i64().unwrap() as i32;
        let y = jrect.find("y").unwrap().as_i64().unwrap() as i32;
        let width = jrect.find("width").unwrap().as_i64().unwrap() as i32;
        let height = jrect.find("height").unwrap().as_i64().unwrap() as i32;
        (x, y, width, height)
    }

    /// Gets the current workspaces.
    pub fn get_workspaces(&mut self) -> io::Result<reply::Workspaces> {
        try!(self.stream.send_i3_message(1, ""));
        let (_, payload) = try!(self.stream.receive_i3_message());

        let j: json::Value = json::from_str(&payload).unwrap();
        let jworkspaces = j.as_array().unwrap();
        let workspaces: Vec<_>
            = jworkspaces.iter()
                         .map(|w|
                              reply::Workspace {
                                  num: w.find("num").unwrap().as_i64().unwrap() as i32,
                                  name: w.find("name").unwrap().as_string().unwrap().to_owned(),
                                  visible: w.find("visible").unwrap().as_boolean().unwrap(),
                                  focused: w.find("focused").unwrap().as_boolean().unwrap(),
                                  urgent: w.find("urgent").unwrap().as_boolean().unwrap(),
                                  rect: I3Connection::unpack_rect(w.find("rect").unwrap()),
                                  output: w.find("output").unwrap().as_string().unwrap().to_owned()
                              })
                         .collect();
        Ok(reply::Workspaces { workspaces: workspaces })
    }

    /// Gets the current outputs.
    pub fn get_outputs(&mut self) -> io::Result<reply::Outputs> {
        try!(self.stream.send_i3_message(3, ""));
        let (_, payload) = try!(self.stream.receive_i3_message());

        let j: json::Value = json::from_str(&payload).unwrap();
        let joutputs = j.as_array().unwrap();
        let outputs: Vec<_>
            = joutputs.iter()
                      .map(|o|
                           reply::Output {
                               name: o.find("name").unwrap().as_string().unwrap().to_owned(),
                               active: o.find("active").unwrap().as_boolean().unwrap(),
                               current_workspace: match o.find("current_workspace").unwrap().clone() {
                                   json::Value::String(c_w) => Some(c_w),
                                   json::Value::Null => None,
                                   _ => unreachable!()
                               },
                               rect: I3Connection::unpack_rect(o.find("rect").unwrap())
                           })
                      .collect();
        Ok(reply::Outputs { outputs: outputs })
    }

    /// Gets the layout tree. i3 uses a tree as data structure which includes every container.
    pub fn get_tree(&mut self) -> io::Result<reply::Tree> {
        panic!("not implemented");
    }

    /// Gets a list of marks (identifiers for containers to easily jump to them later).
    pub fn get_marks(&mut self) -> io::Result<reply::Marks> {
        try!(self.stream.send_i3_message(5, ""));
        let (_, payload) = try!(self.stream.receive_i3_message());
        let marks: Vec<String> = json::from_str(&payload).unwrap();
        Ok(reply::Marks { marks: marks })
    }

    /// Gets an array with all configured bar IDs.
    pub fn get_bar_ids(&mut self) -> io::Result<reply::BarIds> {
        try!(self.stream.send_i3_message(6, ""));
        let (_, payload) = try!(self.stream.receive_i3_message());
        let ids: Vec<String> = json::from_str(&payload).unwrap();
        Ok(reply::BarIds { ids: ids })
    }

    /// Gets the configuration of the workspace bar with the given ID.
    pub fn get_bar_config(&mut self, id: &str) -> io::Result<reply::BarConfig> {
        try!(self.stream.send_i3_message(6, id));
        let (_, payload) = try!(self.stream.receive_i3_message());
        let j: json::Value = json::from_str(&payload).unwrap();
        Ok(reply::BarConfig {
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
                        "active_workspace_bg" => reply::ColorableBarPart::ActiveWorkspaceBg,
                        "inactive_workspace_text" => reply::ColorableBarPart::InactiveWorkspaceText,
                        "inactive_workspace_bg" => reply::ColorableBarPart::InactiveWorkspaceBg,
                        "urgent_workspace_text" => reply::ColorableBarPart::UrgentWorkspaceText,
                        "urgent_workspace_bar" => reply::ColorableBarPart::UrgentWorkspaceBar,
                        other => reply::ColorableBarPart::Undocumented(other.to_owned())
                    };
                    let hex = colors.get(c).unwrap().as_string().unwrap().to_owned();
                    map.insert(enum_key, hex);
                }
                map
            }
        })
    }

    /// Gets the version of i3. The reply will include the major, minor, patch and human-readable
    /// version.
    pub fn get_version(&mut self) -> io::Result<reply::Version> {
        try!(self.stream.send_i3_message(7, ""));
        let (_, payload) = try!(self.stream.receive_i3_message());
        let j: json::Value = json::from_str(&payload).unwrap();
        Ok(reply::Version {
            major: j.find("major").unwrap().as_i64().unwrap() as i32,
            minor: j.find("minor").unwrap().as_i64().unwrap() as i32,
            patch: j.find("patch").unwrap().as_i64().unwrap() as i32,
            human_readable: j.find("human_readable").unwrap().as_string().unwrap().to_owned()
        })
    }
}


#[cfg(test)]
mod test {
    use I3Connection;
    use I3EventListener;
    use event;
    use event::EventType;
    use std::str::FromStr;

    // for the following tests send a request and get the reponse.
    // response types are specific so often getting them at all indicates success.
    // can't do much better without mocking an i3 installation.
    
    #[test]
    fn connect() {
        I3Connection::connect().unwrap();
    }

    #[test]
    fn command_nothing() {
        let mut connection = I3Connection::connect().unwrap();
        let result = connection.command("").unwrap();
        assert_eq!(result.outcomes.len(), 0);
    }

    #[test]
    fn command_single_sucess() {
        let mut connection = I3Connection::connect().unwrap();
        let a = connection.command("exec /bin/true").unwrap();
        assert_eq!(a.outcomes.len(), 1);
        assert!(a.outcomes[0].success);
    }

    #[test]
    fn command_multiple_success() {
        let mut connection = I3Connection::connect().unwrap();
        let result = connection.command("exec /bin/true; exec /bin/true").unwrap();
        assert_eq!(result.outcomes.len(), 2);
        assert!(result.outcomes[0].success);
        assert!(result.outcomes[1].success);
    }

    #[test]
    fn command_fail() {
        let mut connection = I3Connection::connect().unwrap();
        let result = connection.command("ThisIsClearlyNotACommand").unwrap();
        assert_eq!(result.outcomes.len(), 1);
        assert!(!result.outcomes[0].success);
    }

    #[test]
    fn get_workspaces() {
        I3Connection::connect().unwrap().get_workspaces().unwrap();
    }

    #[test]
    fn get_outputs() {
        I3Connection::connect().unwrap().get_outputs().unwrap();
    }

    #[test]
    fn get_tree() {
        I3Connection::connect().unwrap().get_tree().unwrap();
    }

    #[test]
    fn get_marks() {
        I3Connection::connect().unwrap().get_marks().unwrap();
    }

    #[test]
    fn get_bar_ids() {
        I3Connection::connect().unwrap().get_bar_ids().unwrap();
    }

    #[test]
    fn get_bar_ids_and_one_config() {
        let mut connection = I3Connection::connect().unwrap();
        let ids = connection.get_bar_ids().unwrap().ids;
        let oneconfig = connection.get_bar_config(&ids[0]).unwrap();
    }

    #[test]
    fn get_version() {
        I3Connection::connect().unwrap().get_version().unwrap();
    }

    #[test]
    fn event_subscribe() {
        I3EventListener::connect().unwrap().subscribe(&[EventType::Workspace]).unwrap();
    }

    #[test]
    fn from_str_workspace() {
        let json_str = r##"
        {
            "change": "focus",
            "current": {
                "id": 28489712,
                "name": "something"
                "type": "workspace",
                "border": "normal",
                "current_border_width": 2,
                "layout": "splith"
                "orientation": "none",
                "percent": 30.0,
                "rect": { "x": 1600, "y": 0, "width": 1600, "height": 1200 },
                "window_rect": { "x": 2, "y": 0, "width": 632, "height": 366 },
                "deco_rect": { "x": 1, "y": 1, "width": 631, "height": 365 },
                "geometry": { "x": 6, "y": 6, "width": 10, "height": 10 },
                "window": 1,
                "urgent" false,
                "focused": true,
            }
            "old": null
        }"##;
        event::Workspace::from_str(json_str).unwrap();
    }

    #[test]
    fn from_str_output() {
        let json_str = r##"{ "change": "unspecified" }"##;
        event::Output::from_str(json_str).unwrap();
    }

    #[test]
    fn from_str_mode() {
        let json_str = r##"{ "change": "default" }"##;
        event::Mode::from_str(json_str).unwrap();
    }

    #[test]
    fn from_str_window() {
        let json_str = r##"
        {
            "change": "new",
            "container: {
                "id": 28489712,
                "name": "something"
                "type": "workspace",
                "border": "normal",
                "current_border_width": 2,
                "layout": "splith"
                "orientation": "none",
                "percent": 30.0,
                "rect": { "x": 1600, "y": 0, "width": 1600, "height": 1200 },
                "window_rect": { "x": 2, "y": 0, "width": 632, "height": 366 },
                "deco_rect": { "x": 1, "y": 1, "width": 631, "height": 365 },
                "geometry": { "x": 6, "y": 6, "width": 10, "height": 10 },
                "window": 1,
                "urgent" false,
                "focused": true,
            }
        }"##;
        event::Window::from_str(json_str).unwrap();
    }

    #[test]
    fn from_str_barconfig() {
        let json_str = r##"
        {
            "id": "bar-bxuqzf",
            "mode": "dock",
            "position": "bottom",
            "status_command": "i3status",
            "font": "-misc-fixed-medium-r-normal--13-120-75-75-C-70-iso10646-1",
            "workspace_buttons": true,
            "binding_mode_indicator": true,
            "verbose": false,
            "colors": {
                    "background": "#c0c0c0",
                    "statusline": "#00ff00",
                    "focused_workspace_text": "#ffffff",
                    "focused_workspace_bg": "#000000"
            }
        }"##;
        event::BarConfig::from_str(json_str).unwrap();
    }

    #[test]
    fn from_str_binding_event() {
        let json_str = r##"
        {
            "change": "run",
            "binding": {
                "command": "nop",
                "mods": [
                    "shift",
                    "ctrl"
                ],
                "input_code": 0,
                "symbol": "t",
                "input_type": "keyboard"
            }
        }"##;
        event::BindingEvent::from_str(json_str).unwrap();
    }
}
