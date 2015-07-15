//! Prints out every event that comes from i3.
//! What displays on screen isn't json, but the data as understood by this library.

extern crate i3ipc;

use i3ipc::I3EventListener;
use i3ipc::Subscription;

fn main() {
    let mut listener = I3EventListener::connect().ok().expect("failed to connect");
    let subs = [Subscription::Workspace, Subscription::Output, Subscription::Mode,
                Subscription::Window, Subscription::BarConfig, Subscription::Binding];
    listener.subscribe(&subs).ok().expect("failed to subscribe");
    for event in listener.listen() {
        println!("{:?}\n", event.ok().expect("failed to get event"))
    }
}
