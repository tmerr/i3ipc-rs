extern crate i3ipc;

use i3ipc::I3Connection;
use std::io;
use std::io::Write;

fn main() {
    println!("Executes i3 commands in a loop. Enter \"q\" at any time to quit.");
    let mut connection = I3Connection::connect().expect("failed to connect");
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        print!(">>> ");
        stdout.flush().unwrap();
        let mut command_text = String::new();
        stdin.read_line(&mut command_text).unwrap();
        command_text.pop(); // throw away the \n
        if command_text == "q" {
            break;
        }

        let outcomes = connection
            .run_command(&command_text)
            .expect("failed to send command")
            .outcomes;
        for outcome in outcomes {
            if outcome.success {
                println!("success");
            } else {
                println!("failure");
                if let Some(e) = outcome.error.as_ref() {
                    println!("{}", e);
                }
            }
        }
    }

    // the socket closes when `connection` goes out of scope
}
