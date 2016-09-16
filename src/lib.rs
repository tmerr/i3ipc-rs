//! A library for controlling i3-wm through its ipc interface.
//!
//! Using `I3Connection` you
//! could send a command or get the hierarchy of containers. With
//! `I3EventListener` you could listen for when the focused window changes. One of the goals is
//! is to make this process as fool-proof as possible: usage should follow from the type
//! signatures. 
//!
//! The types in the `event` and `reply` modules are near direct translations from the JSON
//! used to talk to i3. The relevant
//! documentation (meaning of each json object and field) is shamelessly stolen from the
//! [site](https://i3wm.org/docs/ipc.html)
//! and put into those modules.
//!
//! This library should cover all of i3's documented ipc features. If it's missing something
//! please open an issue on github.

extern crate unix_socket;
extern crate byteorder;
extern crate serde;
extern crate serde_json;

use std::{io, fmt, process};
use std::io::prelude::*;
use std::error::Error;
use std::str::FromStr;

use unix_socket::UnixStream;
use serde_json as json;
use event::Event;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

mod readhelp;
mod common;
pub mod reply;
pub mod event;

/// An error initializing a connection.
///
/// It first involves first getting the i3 socket path, then connecting to the socket. Either part
/// could go wrong which is why there are two possibilities here.
#[derive(Debug)]
pub enum EstablishError {
    /// An error while getting the socket path
    GetSocketPathError(io::Error),
    /// An error while accessing the socket
    SocketError(io::Error)
}

impl Error for EstablishError {
    fn description(&self) -> &str {
        match *self {
            EstablishError::GetSocketPathError(_) => "Couldn't determine i3's socket path",
            EstablishError::SocketError(_) => "Found i3's socket path but failed to connect"
        }
    }
    fn cause(&self) -> Option<&Error> {
        match *self {
            EstablishError::GetSocketPathError(ref e) => Some(e),
            EstablishError::SocketError(ref e) => Some(e)
        }
    }
}

impl fmt::Display for EstablishError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// An error sending or receiving a message.
#[derive(Debug)]
pub enum MessageError {
    /// Network error sending the message.
    Send(io::Error),
    /// Network error receiving the response.
    Receive(io::Error),
    /// Got the response but couldn't parse the JSON.
    JsonCouldntParse(json::Error),
}

impl Error for MessageError {
    fn description(&self) -> &str {
        match *self {
            MessageError::Send(_) => "Network error while sending message to i3",
            MessageError::Receive(_) => "Network error while receiving message from i3",
            MessageError::JsonCouldntParse(_) => "Got a response from i3 but couldn't parse the JSON",
        }
    }
    fn cause(&self) -> Option<&Error> {
        match *self {
            MessageError::Send(ref e) => Some(e),
            MessageError::Receive(ref e) => Some(e),
            MessageError::JsonCouldntParse(ref e) => Some(e),
        }
    }
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
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

/// Iterates over events from i3.
///
/// Each element may be `Err` or `Ok` (Err for an issue with the socket connection or data sent
/// from i3).
#[derive(Debug)]
pub struct EventIterator<'a> {
    stream: &'a mut UnixStream,
}

impl<'a> Iterator for EventIterator<'a> {
    type Item = Result<event::Event, MessageError>;

    fn next(&mut self) -> Option<Self::Item>{
        /// the msgtype passed in should have its highest order bit stripped
        /// makes the i3 event
        fn build_event(msgtype: u32, payload: &str) -> Result<event::Event, json::Error> {
            Ok(match msgtype {
                0 => event::Event::WorkspaceEvent(try!(event::WorkspaceEventInfo::from_str(payload))),
                1 => event::Event::OutputEvent(try!(event::OutputEventInfo::from_str(payload))),
                2 => event::Event::ModeEvent(try!(event::ModeEventInfo::from_str(payload))),
                3 => event::Event::WindowEvent(try!(event::WindowEventInfo::from_str(payload))),
                4 => event::Event::BarConfigEvent(try!(event::BarConfigEventInfo::from_str(payload))),
                5 => event::Event::BindingEvent(try!(event::BindingEventInfo::from_str(payload))),
                _ => unreachable!()
            })
        }

        match self.stream.receive_i3_message() {
            Ok((msgint, payload)) => {
                // strip the highest order bit indicating it's an event.
                let msgtype = (msgint << 1) >> 1;
                
                Some(match build_event(msgtype, &payload) {
                    Ok(event) => Ok(event),
                    Err(e) => Err(MessageError::JsonCouldntParse(e)),
                })
            }
            Err(e) => Some(Err(MessageError::Receive(e))),
        }
    }
}

/// A subscription for `I3EventListener`
#[derive(Debug)]
pub enum Subscription {
    Workspace,
    Output,
    Mode,
    Window,
    BarConfig,
    Binding
}

/// Abstraction over an ipc socket to i3. Handles events.
#[derive(Debug)]
pub struct I3EventListener {
    stream: UnixStream
}

impl I3EventListener {
    /// Establishes the IPC connection.
    pub fn connect() -> Result<I3EventListener, EstablishError> {
        return match get_socket_path() {
            Ok(path) => {
                match UnixStream::connect(path) {
                    Ok(stream) => Ok(I3EventListener { stream: stream }),
                    Err(error) => Err(EstablishError::SocketError(error))
                }
            }
            Err(error) => Err(EstablishError::GetSocketPathError(error))
        }
    }

    /// Subscribes your connection to certain events.
    pub fn subscribe(&mut self, events: &[Subscription]) -> Result<reply::Subscribe, MessageError> {
        let json =
            "[ ".to_owned()
            + &events.iter()
                    .map(|s| match *s {
                        Subscription::Workspace => "\"workspace\"",
                        Subscription::Output => "\"output\"",
                        Subscription::Mode => "\"mode\"",
                        Subscription::Window => "\"window\"",
                        Subscription::BarConfig => "\"barconfig_update\"",
                        Subscription::Binding => "\"binding\""})
                    .collect::<Vec<_>>()
                    .join(", ")[..]
            + " ]";
        if let Err(e) = self.stream.send_i3_message(2, &json) {
            return Err(MessageError::Send(e));
        }
        let payload = match self.stream.receive_i3_message() {
            Ok((_, payload)) => payload,
            Err(e) => { return Err(MessageError::Receive(e)); }
        };
        let j: json::Value = match json::from_str(&payload) {
            Ok(j) => j,
            Err(e) => { return Err(MessageError::JsonCouldntParse(e)) }
        };
        let is_success = j.find("success").unwrap().as_boolean().unwrap();
        Ok(reply::Subscribe { success: is_success })
    }

    /// Iterate over subscribed events forever.
    pub fn listen(&mut self) -> EventIterator {
        EventIterator { stream: &mut self.stream }
    }
}

/// Abstraction over an ipc socket to i3. Handles messages/replies.
#[derive(Debug)]
pub struct I3Connection {
    stream: UnixStream
}

impl I3Connection {
    /// Establishes the IPC connection.
    pub fn connect() -> Result<I3Connection, EstablishError> {
        return match get_socket_path() {
            Ok(path) => {
                match UnixStream::connect(path) {
                    Ok(stream) => Ok(I3Connection { stream: stream }),
                    Err(error) => Err(EstablishError::SocketError(error))
                }
            }
            Err(error) => Err(EstablishError::GetSocketPathError(error))
        }
    }

    /// The payload of the message is a command for i3 (like the commands you can bind to keys
    /// in the configuration file) and will be executed directly after receiving it.
    pub fn command(&mut self, string: &str) -> Result<reply::Command, MessageError> {
        if let Err(e) = self.stream.send_i3_message(0, string) {
            return Err(MessageError::Send(e));
        }
        let payload = match self.stream.receive_i3_message() {
            Ok((_, payload)) => payload,
            Err(e) => { return Err(MessageError::Receive(e)); }
        };

        let j = match json::from_str::<json::Value>(&payload) {
            Ok(v) => v,
            Err(e) => { return Err(MessageError::JsonCouldntParse(e)); }
        };
        // assumes valid json contents
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

    /// Gets the current workspaces.
    pub fn get_workspaces(&mut self) -> Result<reply::Workspaces, MessageError> {
        if let Err(e) = self.stream.send_i3_message(1, "") {
            return Err(MessageError::Send(e));
        }
        let payload = match self.stream.receive_i3_message() {
            Ok((_, payload)) => payload,
            Err(e) => { return Err(MessageError::Receive(e)); }
        };

        let j = match json::from_str::<json::Value>(&payload) {
            Ok(v) => v,
            Err(e) => { return Err(MessageError::JsonCouldntParse(e)); }
        };
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
                                  rect: common::build_rect(w.find("rect").unwrap()),
                                  output: w.find("output").unwrap().as_string().unwrap().to_owned()
                              })
                         .collect();
        Ok(reply::Workspaces { workspaces: workspaces })
    }

    /// Gets the current outputs.
    pub fn get_outputs(&mut self) -> Result<reply::Outputs, MessageError> {
        if let Err(e) = self.stream.send_i3_message(3, "") {
            return Err(MessageError::Send(e));
        }
        let payload = match self.stream.receive_i3_message() {
            Ok((_, payload)) => payload,
            Err(e) => { return Err(MessageError::Receive(e)); }
        };

        let j = match json::from_str::<json::Value>(&payload) {
            Ok(v) => v,
            Err(e) => { return Err(MessageError::JsonCouldntParse(e)); }
        };
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
                               rect: common::build_rect(o.find("rect").unwrap())
                           })
                      .collect();
        Ok(reply::Outputs { outputs: outputs })
    }

    /// Gets the layout tree. i3 uses a tree as data structure which includes every container.
    pub fn get_tree(&mut self) -> Result<reply::Node, MessageError> {
        if let Err(e) = self.stream.send_i3_message(4, "") {
            return Err(MessageError::Send(e));
        }
        let payload = match self.stream.receive_i3_message() {
            Ok((_, payload)) => payload,
            Err(e) => { return Err(MessageError::Receive(e)); }
        };
        let val = match json::from_str::<json::Value>(&payload) {
            Ok(v) => v,
            Err(e) => { return Err(MessageError::JsonCouldntParse(e)); }
        };
        Ok(common::build_tree(&val))
    }

    /// Gets a list of marks (identifiers for containers to easily jump to them later).
    pub fn get_marks(&mut self) -> Result<reply::Marks, MessageError> {
        if let Err(e) = self.stream.send_i3_message(5, "") {
            return Err(MessageError::Send(e));
        }
        let payload = match self.stream.receive_i3_message() {
            Ok((_, payload)) => payload,
            Err(e) => { return Err(MessageError::Receive(e)); }
        };
        let marks: Vec<String> = match json::from_str(&payload) {
            Ok(v) => v,
            Err(e) => { return Err(MessageError::JsonCouldntParse(e)); }
        };
        Ok(reply::Marks { marks: marks })
    }

    /// Gets an array with all configured bar IDs.
    pub fn get_bar_ids(&mut self) -> Result<reply::BarIds, MessageError> {
        if let Err(e) = self.stream.send_i3_message(6, "") {
            return Err(MessageError::Send(e));
        }
        let payload = match self.stream.receive_i3_message() {
            Ok((_, payload)) => payload,
            Err(e) => { return Err(MessageError::Receive(e)); }
        };
        let ids: Vec<String> = match json::from_str(&payload) {
            Ok(v) => v,
            Err(e) => { return Err(MessageError::JsonCouldntParse(e)); }
        };
        Ok(reply::BarIds { ids: ids })
    }

    /// Gets the configuration of the workspace bar with the given ID.
    pub fn get_bar_config(&mut self, id: &str) -> Result<reply::BarConfig, MessageError> {
        if let Err(e) = self.stream.send_i3_message(6, id) {
            return Err(MessageError::Send(e));
        }
        let payload = match self.stream.receive_i3_message() {
            Ok((_, payload)) => payload,
            Err(e) => { return Err(MessageError::Receive(e)); }
        };
        let j = match json::from_str::<json::Value>(&payload) {
            Ok(v) => v,
            Err(e) => { return Err(MessageError::JsonCouldntParse(e)); }
        };
        Ok(common::build_bar_config(&j))
    }

    /// Gets the version of i3. The reply will include the major, minor, patch and human-readable
    /// version.
    pub fn get_version(&mut self) -> Result<reply::Version, MessageError> {
        if let Err(e) = self.stream.send_i3_message(7, "") {
            return Err(MessageError::Send(e));
        }
        let payload = match self.stream.receive_i3_message() {
            Ok((_, payload)) => payload,
            Err(e) => { return Err(MessageError::Receive(e)); }
        };
        let j = match json::from_str::<json::Value>(&payload) {
            Ok(v) => v,
            Err(e) => { return Err(MessageError::JsonCouldntParse(e)); }
        };
        Ok(reply::Version {
            major: j.find("major").unwrap().as_i64().unwrap() as i32,
            minor: j.find("minor").unwrap().as_i64().unwrap() as i32,
            patch: j.find("patch").unwrap().as_i64().unwrap() as i32,
            human_readable: j.find("human_readable").unwrap().as_string().unwrap().to_owned(),
            loaded_config_file_name: j.find("loaded_config_file_name").unwrap().as_string()
                                                                      .unwrap().to_owned()
        })
    }
}


#[cfg(test)]
mod test {
    use I3Connection;
    use I3EventListener;
    use event;
    use Subscription;
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
        connection.get_bar_config(&ids[0]).unwrap();
    }

    #[test]
    fn get_version() {
        I3Connection::connect().unwrap().get_version().unwrap();
    }

    #[test]
    fn event_subscribe() {
        let s = I3EventListener::connect().unwrap().subscribe(&[Subscription::Workspace]).unwrap();
        assert_eq!(s.success, true);
    }

    #[test]
    fn from_str_workspace() {
        let json_str = r##"
        {
            "change": "focus",
            "current": {
                "id": 28489712,
                "name": "something",
                "type": "workspace",
                "border": "normal",
                "current_border_width": 2,
                "layout": "splith",
                "orientation": "none",
                "percent": 30.0,
                "rect": { "x": 1600, "y": 0, "width": 1600, "height": 1200 },
                "window_rect": { "x": 2, "y": 0, "width": 632, "height": 366 },
                "deco_rect": { "x": 1, "y": 1, "width": 631, "height": 365 },
                "geometry": { "x": 6, "y": 6, "width": 10, "height": 10 },
                "window": 1,
                "urgent": false,
                "focused": true
            },
            "old": null
        }"##;
        event::WorkspaceEventInfo::from_str(json_str).unwrap();
    }

    #[test]
    fn from_str_output() {
        let json_str = r##"{ "change": "unspecified" }"##;
        event::OutputEventInfo::from_str(json_str).unwrap();
    }

    #[test]
    fn from_str_mode() {
        let json_str = r##"{ "change": "default" }"##;
        event::ModeEventInfo::from_str(json_str).unwrap();
    }

    #[test]
    fn from_str_window() {
        let json_str = r##"
        {
            "change": "new",
            "container": {
                "id": 28489712,
                "name": "something",
                "type": "workspace",
                "border": "normal",
                "current_border_width": 2,
                "layout": "splith",
                "orientation": "none",
                "percent": 30.0,
                "rect": { "x": 1600, "y": 0, "width": 1600, "height": 1200 },
                "window_rect": { "x": 2, "y": 0, "width": 632, "height": 366 },
                "deco_rect": { "x": 1, "y": 1, "width": 631, "height": 365 },
                "geometry": { "x": 6, "y": 6, "width": 10, "height": 10 },
                "window": 1,
                "urgent": false,
                "focused": true
            }
        }"##;
        event::WindowEventInfo::from_str(json_str).unwrap();
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
        event::BarConfigEventInfo::from_str(json_str).unwrap();
    }

    #[test]
    fn from_str_binding_event() {
        let json_str = r##"
        {
            "change": "run",
            "binding": {
                "command": "nop",
                "event_state_mask": [
                    "shift",
                    "ctrl"
                ],
                "input_code": 0,
                "symbol": "t",
                "input_type": "keyboard"
            }
        }"##;
        event::BindingEventInfo::from_str(json_str).unwrap();
    }
}
