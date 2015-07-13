//! A library for controlling i3-wm through its ipc interface.

extern crate unix_socket;
extern crate byteorder;
extern crate serde;

use std::process;
use unix_socket::UnixStream;
use std::io;
use std::io::prelude::*;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use serde::json;

mod readhelp;
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

pub struct I3Connection {
    stream: UnixStream
}

impl I3Connection {

    /// Establishes an IPC connection to i3.
    pub fn connect() -> Result<I3Connection, I3ConnectError> {
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
    
    fn send_message(&mut self, message_type: u32, payload: &str) -> io::Result<()> {
        let mut bytes = Vec::with_capacity(14 + payload.len());
        bytes.extend("i3-ipc".bytes());                              // 6 bytes
        try!(bytes.write_u32::<LittleEndian>(payload.len() as u32)); // 4 bytes
        try!(bytes.write_u32::<LittleEndian>(message_type));         // 4 bytes
        bytes.extend(payload.bytes());                               // payload.len() bytes
        self.stream.write_all(&bytes[..])
    }

    fn receive_message(&mut self) -> io::Result<String> {
        let magic_data = try!(readhelp::read_exact(&mut self.stream, 6));
        let magic_string = String::from_utf8_lossy(&magic_data);
        if magic_string != "i3-ipc" {
            let error_text = format!("unexpected magic string: expected 'i3-ipc' but got {}",
                                      magic_string);
            return Err(io::Error::new(io::ErrorKind::Other, error_text));
        }
        let payload_len = try!(self.stream.read_u32::<LittleEndian>());
        let message_type = try!(self.stream.read_u32::<LittleEndian>());
        let payload_data = try!(readhelp::read_exact(&mut self.stream, payload_len as usize));
        Ok(String::from_utf8_lossy(&payload_data).into_owned())
    }

    /// The payload of the message is a command for i3 (like the commands you can bind to keys
    /// in the configuration file) and will be executed directly after receiving it.
    pub fn command(&mut self, string: &str) -> io::Result<reply::Command> {
        try!(self.send_message(0, string));
        let payload = try!(self.receive_message());

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

    /// Gets the current workspaces.
    pub fn get_workspaces(&mut self) -> io::Result<reply::Workspaces> {
        try!(self.send_message(1, ""));
        let payload = try!(self.receive_message());

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
                                  rect: {
                                      let jrect = w.find("rect").unwrap();
                                      (jrect.find("x").unwrap().as_i64().unwrap() as i32,
                                       jrect.find("y").unwrap().as_i64().unwrap() as i32,
                                       jrect.find("width").unwrap().as_i64().unwrap() as i32,
                                       jrect.find("height").unwrap().as_i64().unwrap() as i32)
                                  },
                                  output: w.find("output").unwrap().as_string().unwrap().to_owned()
                              })
                         .collect();
        Ok(reply::Workspaces { workspaces: workspaces })
    }

    /// Subscribes your connection to certain events.
    pub fn subscribe(&mut self) -> io::Result<reply::Subscribe> {
        panic!("not implemented");
    }

    /// Gets the current outputs.
    pub fn get_outputs(&mut self) -> io::Result<reply::Outputs> {
        panic!("not implemented");
    }

    /// Gets the layout tree. i3 uses a tree as data structure which includes every container.
    pub fn get_tree(&mut self) -> io::Result<reply::Tree> {
        panic!("not implemented");
    }

    /// Gets a list of marks (identifiers for containers to easily jump to them later).
    pub fn get_marks(&mut self) -> io::Result<reply::Marks> {
        panic!("not implemented");
    }

    /// Gets an array with all configured bar IDs.
    pub fn get_bar_ids(&mut self) -> io::Result<reply::BarIds> {
        panic!("not implemented");
    }

    /// Gets the configuration of the workspace bar with the given ID.
    pub fn get_bar_config(&mut self, id: &str) -> io::Result<reply::BarConfig> {
        panic!("not implemented");
    }

    /// Gets the version of i3. The reply will include the major, minor, patch and human-readable
    /// version.
    pub fn get_version(&mut self) -> io::Result<reply::Version> {
        panic!("not implemented");
    }
}


#[cfg(test)]
mod test {
    use I3Connection;

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
    fn subscribe() {
        I3Connection::connect().unwrap().subscribe().unwrap();
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
}
