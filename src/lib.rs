//! # QEMU QSL
//!
//! `qemu_qsl` listens to a QEMU QMP socket. It can print messages received from QEMU as well as
//! send some requests to the hypervisor.
//!
//! See the [README](https://github.com/wwkeyboard/qemu-QMP-test#readme) for CLI usage information.
pub mod cli;
pub mod connection;
pub mod messages {
    pub mod client;
    pub mod server;
}
