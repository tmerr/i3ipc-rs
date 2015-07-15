# i3ipc-rs
[![Build Status](https://travis-ci.org/tmerr/i3ipc-rs.svg?branch=master)](https://travis-ci.org/tmerr/i3ipc-rs)
[![](http://meritbadge.herokuapp.com/i3ipc)](https://crates.io/crates/i3ipc)

A Rust library for controlling i3-wm through its IPC interface as per https://i3wm.org/docs/ipc.html.

##Usage
Add this to your Cargo.toml
```toml
[dependencies]
i3ipc = "*"
```

##Messages:

````rust
extern crate i3ipc;
use i3ipc::I3Connection;

fn main() {
    // establish a connection to i3 over a unix socket
    let mut connection = I3Connection::connect().unwrap();
    
    // request and print the i3 version
    println!("{}", connection.get_version().unwrap().human_readable);
    
    // fullscreen the focused window
    connection.command("fullscreen").unwrap();
}
```

## Events:

```rust
extern crate i3ipc;
use i3ipc::I3EventListener;
use i3ipc::Subscription;
use i3ipc::event::Event;

fn main() {
    // establish connection.
    let mut listener = I3EventListener::connect().unwrap();

    // subscribe to a couple events.
    let subs = [Subscription::Mode, Subscription::Binding];
    listener.subscribe(&subs).unwrap();

    // handle them
    for event in listener.listen() {
        match event.unwrap() {
            Event::ModeEvent(e) => println!("new mode: {}", e.change),
            Event::BindingEvent(e) => println!("user input triggered command: {}", e.binding.command),
            _ => unreachable!()
        }
    }
}
```

## Status
The library is complete as far as functionality goes. However the error handling and the API layout are still in the works.
