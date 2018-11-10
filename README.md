# i3ipc-rs

[![Build Status](https://travis-ci.org/tmerr/i3ipc-rs.svg?branch=master)](https://travis-ci.org/tmerr/i3ipc-rs)
[![Crate](http://meritbadge.herokuapp.com/i3ipc)](https://crates.io/crates/i3ipc)
[![Docs](https://docs.rs/i3ipc/badge.svg)](https://docs.rs/i3ipc)

A Rust library for controlling i3-wm through its [IPC interface](https://i3wm.org/docs/ipc.html).

## Usage
Add this to your Cargo.toml
```toml
[dependencies.i3ipc]
version = "0.9.0"
```

## Messages:

```rust
extern crate i3ipc;
use i3ipc::I3Connection;

fn main() {
    // establish a connection to i3 over a unix socket
    let mut connection = I3Connection::connect().unwrap();
    
    // request and print the i3 version
    println!("{}", connection.get_version().unwrap().human_readable);
    
    // fullscreen the focused window
    connection.run_command("fullscreen").unwrap();
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

## Versioning

By default i3ipc-rs targets minimum i3 version 4.11. To unlock additional features you can increase this by selecting one of `"i3-4-12"`, ..., `"i3-4-14"` in Cargo.toml.

```
[dependencies.i3ipc]
version = "0.9.0"
features = ["i3-4-14"]
```

Additions to the i3 IPC interface that are not understood by your compiled binary will generally return an `Unknown` value and log a warning to the target `"i3ipc"` using the [log crate](http://doc.rust-lang.org/log). Binaries using this library should [install a logger](https://doc.rust-lang.org/log/log/index.html#in-executables) to view details of such additions.
