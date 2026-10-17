#![forbid(unsafe_code)]

// Intentionally no background service installation or screen capture yet.
// Privileged IPC and consent gates are prerequisites (issues #49 and #54).
fn main() {
    println!(
        "Seven Desk host service prototype. Protocol v{}. Remote control is disabled.",
        seven_desk_protocol::PROTOCOL_VERSION
    );
}
