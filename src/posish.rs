//! All Posix-ish platforms have `RawFd` and related traits. Re-export them
//! so that users don't need target-specific code to import them.

#[cfg(unix)]
pub use std::os::unix::io::{AsRawFd, IntoRawFd, FromRawFd, RawFd};
#[cfg(target_os = "wasi")]
pub use std::os::wasi::io::{AsRawFd, IntoRawFd, FromRawFd, RawFd};

// In theory we could do something similar for
// `std::os::fortanix_sgx::io::{AsRawFd, FromRawFd, RawFd}`, however it lacks
// `IntoRawFd`, and `std::fs::File` doesn't implement its `AsRawFd`, so it
// would need more than a simple re-export.
