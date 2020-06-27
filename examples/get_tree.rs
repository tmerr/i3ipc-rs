extern crate i3ipc;

use i3ipc::I3Connection;

fn main() {
    // Establish a connection to i3 over a unix socket.
    let mut connection = I3Connection::connect().unwrap();

    // Request and print the i3 version.
    eprintln!("{}", connection.get_version().unwrap().human_readable);

    // Get the tree.

    println!("current tree: {:#?}", connection.get_tree().unwrap());
}
