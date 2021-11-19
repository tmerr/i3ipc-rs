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

#![cfg_attr(feature = "dox", feature(doc_cfg))]

extern crate byteorder;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_json;

use std::error::Error;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::str::FromStr;
use std::{env, fmt, io, process};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde_json as json;

mod common;
pub mod event;
pub mod reply;

/// An error initializing a connection.
///
/// It first involves first getting the i3 socket path, then connecting to the socket. Either part
/// could go wrong which is why there are two possibilities here.
#[derive(Debug)]
pub enum EstablishError {
    /// An error while getting the socket path
    GetSocketPathError(io::Error),
    /// An error while accessing the socket
    SocketError(io::Error),
}

impl Error for EstablishError {
    fn description(&self) -> &str {
        match *self {
            EstablishError::GetSocketPathError(_) => "Couldn't determine i3's socket path",
            EstablishError::SocketError(_) => "Found i3's socket path but failed to connect",
        }
    }
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            EstablishError::GetSocketPathError(ref e) | EstablishError::SocketError(ref e) => {
                Some(e)
            }
        }
    }
}

impl fmt::Display for EstablishError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
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
            MessageError::JsonCouldntParse(_) => {
                "Got a response from i3 but couldn't parse the JSON"
            }
        }
    }
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            MessageError::Send(ref e) | MessageError::Receive(ref e) => Some(e),
            MessageError::JsonCouldntParse(ref e) => Some(e),
        }
    }
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

fn get_socket_path() -> io::Result<String> {
    if let Ok(sockpath) = env::var("I3SOCK") {
        return Ok(sockpath);
    }
    // Sway support is an untested and unsupported feature
    if let Ok(sockpath) = env::var("SWAYSOCK") {
        return Ok(sockpath);
    }

    let output = process::Command::new("i3").arg("--get-socketpath").output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end_matches('\n')
            .to_owned())
    } else {
        let prefix = "i3 --get-socketpath didn't return 0";
        let error_text = if !output.stderr.is_empty() {
            format!("{}. stderr: {:?}", prefix, output.stderr)
        } else {
            prefix.to_owned()
        };
        let error = io::Error::new(io::ErrorKind::Other, error_text);
        Err(error)
    }
}

trait I3Funcs {
    fn send_i3_message(&mut self, message_type: u32, payload: &str) -> io::Result<()>;
    fn receive_i3_message(&mut self) -> io::Result<(u32, String)>;
    fn send_receive_i3_message<T: serde::de::DeserializeOwned>(
        &mut self,
        message_type: u32,
        payload: &str,
    ) -> Result<T, MessageError>;
}

impl I3Funcs for UnixStream {
    fn send_i3_message(&mut self, message_type: u32, payload: &str) -> io::Result<()> {
        let mut bytes = Vec::with_capacity(14 + payload.len());
        bytes.extend("i3-ipc".bytes()); // 6 bytes
        bytes.write_u32::<LittleEndian>(payload.len() as u32)?; // 4 bytes
        bytes.write_u32::<LittleEndian>(message_type)?; // 4 bytes
        bytes.extend(payload.bytes()); // payload.len() bytes
        self.write_all(&bytes[..])
    }

    /// returns a tuple of (message type, payload)
    fn receive_i3_message(&mut self) -> io::Result<(u32, String)> {
        let mut magic_data = [0_u8; 6];
        self.read_exact(&mut magic_data)?;
        let magic_string = String::from_utf8_lossy(&magic_data);
        if magic_string != "i3-ipc" {
            let error_text = format!(
                "unexpected magic string: expected 'i3-ipc' but got {}",
                magic_string
            );
            return Err(io::Error::new(io::ErrorKind::Other, error_text));
        }
        let payload_len = self.read_u32::<LittleEndian>()?;
        let message_type = self.read_u32::<LittleEndian>()?;
        let mut payload_data = vec![0_u8; payload_len as usize];
        self.read_exact(&mut payload_data[..])?;
        let payload_string = String::from_utf8_lossy(&payload_data).into_owned();
        Ok((message_type, payload_string))
    }

    fn send_receive_i3_message<T: serde::de::DeserializeOwned>(
        &mut self,
        message_type: u32,
        payload: &str,
    ) -> Result<T, MessageError> {
        if let Err(e) = self.send_i3_message(message_type, payload) {
            return Err(MessageError::Send(e));
        }
        let received = match self.receive_i3_message() {
            Ok((received_type, payload)) => {
                assert_eq!(message_type, received_type);
                payload
            }
            Err(e) => {
                return Err(MessageError::Receive(e));
            }
        };
        match json::from_str(&received) {
            Ok(v) => Ok(v),
            Err(e) => Err(MessageError::JsonCouldntParse(e)),
        }
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

    fn next(&mut self) -> Option<Self::Item> {
        /// the msgtype passed in should have its highest order bit stripped
        /// makes the i3 event
        fn build_event(msgtype: u32, payload: &str) -> Result<event::Event, json::Error> {
            Ok(match msgtype {
                0 => {
                    event::Event::WorkspaceEvent(event::WorkspaceEventInfo::from_str(payload)?)
                }
                1 => event::Event::OutputEvent(event::OutputEventInfo::from_str(payload)?),
                2 => event::Event::ModeEvent(event::ModeEventInfo::from_str(payload)?),
                3 => event::Event::WindowEvent(event::WindowEventInfo::from_str(payload)?),
                4 => {
                    event::Event::BarConfigEvent(event::BarConfigEventInfo::from_str(payload)?)
                }
                5 => event::Event::BindingEvent(event::BindingEventInfo::from_str(payload)?),

                #[cfg(feature = "i3-4-14")]
                6 => event::Event::ShutdownEvent(event::ShutdownEventInfo::from_str(payload)?),

                _ => unreachable!("received an event we aren't subscribed to!"),
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
    Binding,
    #[cfg(feature = "i3-4-14")]
    #[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-14")))]
    Shutdown,
}

/// Abstraction over an ipc socket to i3. Handles events.
#[derive(Debug)]
pub struct I3EventListener {
    stream: UnixStream,
}

impl I3EventListener {
    /// Establishes the IPC connection.
    pub fn connect() -> Result<I3EventListener, EstablishError> {
        match get_socket_path() {
            Ok(path) => match UnixStream::connect(path) {
                Ok(stream) => Ok(I3EventListener { stream }),
                Err(error) => Err(EstablishError::SocketError(error)),
            },
            Err(error) => Err(EstablishError::GetSocketPathError(error)),
        }
    }

    /// Subscribes your connection to certain events.
    pub fn subscribe(&mut self, events: &[Subscription]) -> Result<reply::Subscribe, MessageError> {
        let json = "[ ".to_owned()
            + &events
                .iter()
                .map(|s| match *s {
                    Subscription::Workspace => "\"workspace\"",
                    Subscription::Output => "\"output\"",
                    Subscription::Mode => "\"mode\"",
                    Subscription::Window => "\"window\"",
                    Subscription::BarConfig => "\"barconfig_update\"",
                    Subscription::Binding => "\"binding\"",
                    #[cfg(feature = "i3-4-14")]
                    Subscription::Shutdown => "\"shutdown\"",
                })
                .collect::<Vec<_>>()
                .join(", ")[..]
            + " ]";
        let j: json::Value = self.stream.send_receive_i3_message(2, &json)?;
        let is_success = j.get("success").unwrap().as_bool().unwrap();
        Ok(reply::Subscribe {
            success: is_success,
        })
    }

    /// Iterate over subscribed events forever.
    pub fn listen(&mut self) -> EventIterator {
        EventIterator {
            stream: &mut self.stream,
        }
    }
}

/// Abstraction over an ipc socket to i3. Handles messages/replies.
#[derive(Debug)]
pub struct I3Connection {
    stream: UnixStream,
}

impl I3Connection {
    /// Establishes the IPC connection.
    pub fn connect() -> Result<I3Connection, EstablishError> {
        match get_socket_path() {
            Ok(path) => match UnixStream::connect(path) {
                Ok(stream) => Ok(I3Connection { stream }),
                Err(error) => Err(EstablishError::SocketError(error)),
            },
            Err(error) => Err(EstablishError::GetSocketPathError(error)),
        }
    }

    #[deprecated(since = "0.8.0", note = "Renamed to run_command")]
    pub fn command(&mut self, string: &str) -> Result<reply::Command, MessageError> {
        self.run_command(string)
    }

    /// The payload of the message is a command for i3 (like the commands you can bind to keys
    /// in the configuration file) and will be executed directly after receiving it.
    pub fn run_command(&mut self, string: &str) -> Result<reply::Command, MessageError> {
        let j: json::Value = self.stream.send_receive_i3_message(0, string)?;
        let commands = j.as_array().unwrap();
        let vec: Vec<_> = commands
            .iter()
            .map(|c| reply::CommandOutcome {
                success: c.get("success").unwrap().as_bool().unwrap(),
                error: match c.get("error") {
                    Some(val) => Some(val.as_str().unwrap().to_owned()),
                    None => None,
                },
            })
            .collect();

        Ok(reply::Command { outcomes: vec })
    }

    /// Gets the current workspaces.
    pub fn get_workspaces(&mut self) -> Result<reply::Workspaces, MessageError> {
        let j: json::Value = self.stream.send_receive_i3_message(1, "")?;
        let jworkspaces = j.as_array().unwrap();
        let workspaces: Vec<_> = jworkspaces
            .iter()
            .map(|w| reply::Workspace {
                num: w.get("num").unwrap().as_i64().unwrap() as i32,
                name: w.get("name").unwrap().as_str().unwrap().to_owned(),
                visible: w.get("visible").unwrap().as_bool().unwrap(),
                focused: w.get("focused").unwrap().as_bool().unwrap(),
                urgent: w.get("urgent").unwrap().as_bool().unwrap(),
                rect: common::build_rect(w.get("rect").unwrap()),
                output: w.get("output").unwrap().as_str().unwrap().to_owned(),
            })
            .collect();
        Ok(reply::Workspaces { workspaces })
    }

    /// Gets the current outputs.
    pub fn get_outputs(&mut self) -> Result<reply::Outputs, MessageError> {
        let j: json::Value = self.stream.send_receive_i3_message(3, "")?;
        let joutputs = j.as_array().unwrap();
        let outputs: Vec<_> = joutputs
            .iter()
            .map(|o| reply::Output {
                name: o.get("name").unwrap().as_str().unwrap().to_owned(),
                #[cfg(feature = "sway-1-1")]
                make: o.get("make").unwrap().as_str().unwrap().to_owned(),
                #[cfg(feature = "sway-1-1")]
                model: o.get("model").unwrap().as_str().unwrap().to_owned(),
                #[cfg(feature = "sway-1-1")]
                serial: o.get("serial").unwrap().as_str().unwrap().to_owned(),
                #[cfg(feature = "sway-1-1")]
                scale: o.get("scale").map(|s| s.as_f64().unwrap().to_owned()),
                #[cfg(feature = "sway-1-1")]
                subpixel_hinting: o.get("subpixel_hinting").map(|s| s.as_str() .unwrap().to_owned()),
                #[cfg(feature = "sway-1-1")]
                transform: o.get("transform").map(|s| s.as_str().unwrap().to_owned()),
                #[cfg(feature = "sway-1-1")]
                modes: common::build_modes(o.get("modes").unwrap()),
                #[cfg(feature = "sway-1-1")]
                current_mode: o.get("current_mode").map(|s| common::build_mode(s)),
                active: o.get("active").unwrap().as_bool().unwrap(),
                primary: o.get("primary").unwrap().as_bool().unwrap(),
                current_workspace: match o.get("current_workspace").unwrap().clone() {
                    json::Value::String(c_w) => Some(c_w),
                    json::Value::Null => None,
                    _ => unreachable!(),
                },
                #[cfg(feature = "sway-1-1")]
                dpms: o.get("dpms").unwrap().as_bool().unwrap(),
                rect: common::build_rect(o.get("rect").unwrap()),
            })
            .collect();
        Ok(reply::Outputs { outputs })
    }

    /// Gets the layout tree. i3 uses a tree as data structure which includes every container.
    pub fn get_tree(&mut self) -> Result<reply::Node, MessageError> {
        let val: json::Value = self.stream.send_receive_i3_message(4, "")?;
        Ok(common::build_tree(&val))
    }

    /// Gets a list of marks (identifiers for containers to easily jump to them later).
    pub fn get_marks(&mut self) -> Result<reply::Marks, MessageError> {
        let marks: Vec<String> = self.stream.send_receive_i3_message(5, "")?;
        Ok(reply::Marks { marks })
    }

    /// Gets an array with all configured bar IDs.
    pub fn get_bar_ids(&mut self) -> Result<reply::BarIds, MessageError> {
        let ids: Vec<String> = self.stream.send_receive_i3_message(6, "")?;
        Ok(reply::BarIds { ids })
    }

    /// Gets the configuration of the workspace bar with the given ID.
    pub fn get_bar_config(&mut self, id: &str) -> Result<reply::BarConfig, MessageError> {
        let ids: json::Value = self.stream.send_receive_i3_message(6, id)?;
        Ok(common::build_bar_config(&ids))
    }

    /// Gets the version of i3. The reply will include the major, minor, patch and human-readable
    /// version.
    pub fn get_version(&mut self) -> Result<reply::Version, MessageError> {
        let j: json::Value = self.stream.send_receive_i3_message(7, "")?;
        Ok(reply::Version {
            major: j.get("major").unwrap().as_i64().unwrap() as i32,
            minor: j.get("minor").unwrap().as_i64().unwrap() as i32,
            patch: j.get("patch").unwrap().as_i64().unwrap() as i32,
            human_readable: j
                .get("human_readable")
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned(),
            loaded_config_file_name: j
                .get("loaded_config_file_name")
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned(),
        })
    }

    /// Gets the list of currently configured binding modes.
    #[cfg(feature = "i3-4-13")]
    #[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-13")))]
    pub fn get_binding_modes(&mut self) -> Result<reply::BindingModes, MessageError> {
        let modes: Vec<String> = self.stream.send_receive_i3_message(8, "")?;
        Ok(reply::BindingModes { modes })
    }

    /// Returns the last loaded i3 config.
    #[cfg(feature = "i3-4-14")]
    #[cfg_attr(feature = "dox", doc(cfg(feature = "i3-4-14")))]
    pub fn get_config(&mut self) -> Result<reply::Config, MessageError> {
        let j: json::Value = self.stream.send_receive_i3_message(9, "")?;
        let cfg = j.get("config").unwrap().as_str().unwrap();
        Ok(reply::Config {
            config: cfg.to_owned(),
        })
    }
}

#[cfg(test)]
mod test {
    use event;
    use std::str::FromStr;
    use I3Connection;
    use I3EventListener;
    use Subscription;

    // for the following tests send a request and get the reponse.
    // response types are specific so often getting them at all indicates success.
    // can't do much better without mocking an i3 installation.

    #[test]
    fn connect() {
        I3Connection::connect().unwrap();
    }

    #[test]
    fn run_command_nothing() {
        let mut connection = I3Connection::connect().unwrap();
        let result = connection.run_command("").unwrap();
        assert_eq!(result.outcomes.len(), 0);
    }

    #[test]
    fn run_command_single_sucess() {
        let mut connection = I3Connection::connect().unwrap();
        let a = connection.run_command("exec /bin/true").unwrap();
        assert_eq!(a.outcomes.len(), 1);
        assert!(a.outcomes[0].success);
    }

    #[test]
    fn run_command_multiple_success() {
        let mut connection = I3Connection::connect().unwrap();
        let result = connection
            .run_command("exec /bin/true; exec /bin/true")
            .unwrap();
        assert_eq!(result.outcomes.len(), 2);
        assert!(result.outcomes[0].success);
        assert!(result.outcomes[1].success);
    }

    #[test]
    fn run_command_fail() {
        let mut connection = I3Connection::connect().unwrap();
        let result = connection.run_command("ThisIsClearlyNotACommand").unwrap();
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

    #[cfg(feature = "i3-4-13")]
    #[test]
    fn get_binding_modes() {
        I3Connection::connect()
            .unwrap()
            .get_binding_modes()
            .unwrap();
    }

    #[cfg(feature = "i3-4-14")]
    #[test]
    fn get_config() {
        I3Connection::connect().unwrap().get_config().unwrap();
    }

    #[test]
    fn event_subscribe() {
        let s = I3EventListener::connect()
            .unwrap()
            .subscribe(&[Subscription::Workspace])
            .unwrap();
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
                "window_properties": { "class": "Firefox", "instance": "Navigator", "window_role": "browser", "title": "github.com - Mozilla Firefox", "transient_for": null },
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
