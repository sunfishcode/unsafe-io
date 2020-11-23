//! This example is similar to the tcp_stream test, but writes to a `PipeWriter`.

#![cfg(all(not(target_os = "wasi"), any(not(windows), feature = "os_pipe")))]

use os_pipe::{pipe, PipeReader};
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd};
#[cfg(target_os = "wasi")]
use std::os::wasi::io::{AsRawFd, FromRawFd};
use std::{
    io::{Read, Write},
    mem::ManuallyDrop,
    thread,
};
use unsafe_io::{AsUnsafeFile, AsUnsafeHandle, FromUnsafeFile};
#[cfg(windows)]
use {
    std::os::windows::io::FromRawHandle,
    unsafe_io::{AsRawHandleOrSocket, RawHandleOrSocket},
};

#[test]
#[cfg_attr(miri, ignore)] // pipe I/O calls foreign functions
fn os_pipe_write() -> anyhow::Result<()> {
    let (mut input, output) = pipe()?;

    let _t = thread::spawn(move || -> anyhow::Result<()> {
        // Obtain an `UnsafeWriteable` and use it to write.
        writeln!(
            unsafe { output.as_unsafe_handle().as_writeable() },
            "Write via UnsafeWriteable"
        )?;

        // Obtain an `UnsafeSocket` and use it to construct a temporary manually-drop
        // `PipeWriter` to write.
        writeln!(output.as_file(), "Write via as_file")?;

        // Similar, but do it manually.
        writeln!(
            ManuallyDrop::new(unsafe { std::fs::File::from_unsafe_file(output.as_unsafe_file()) }),
            "Write via unsafe handle"
        )?;

        // Similar, but use the Posix-ish-specific type.
        #[cfg(not(windows))]
        writeln!(
            ManuallyDrop::new(unsafe {
                std::fs::File::from_raw_fd(output.as_unsafe_handle().as_raw_fd())
            }),
            "Write via raw fd"
        )?;

        // Similar, but use the Windows-specific type.
        #[cfg(windows)]
        writeln!(
            ManuallyDrop::new(unsafe {
                std::fs::File::from_raw_handle(
                    match output.as_unsafe_handle().as_raw_handle_or_socket() {
                        RawHandleOrSocket::Handle(handle) => handle,
                        RawHandleOrSocket::Socket(_) => panic!(),
                    },
                )
            }),
            "Write via raw socket"
        )?;

        Ok(())
    });

    let mut buf = String::new();
    input.read_to_string(&mut buf)?;

    #[cfg(not(windows))]
    assert_eq!(
        buf,
        "Write via UnsafeWriteable\n\
                Write via as_file\n\
                Write via unsafe handle\n\
                Write via raw fd\n"
    );

    #[cfg(windows)]
    assert_eq!(
        buf,
        "Write via UnsafeWriteable\n\
                Write via as_file\n\
                Write via unsafe handle\n\
                Write via raw socket\n"
    );

    Ok(())
}

fn write_to_pipe() -> anyhow::Result<PipeReader> {
    let (input, mut output) = pipe()?;

    let _t = thread::spawn(move || -> anyhow::Result<()> {
        write!(output, "hello, world")?;
        Ok(())
    });

    Ok(input)
}

#[test]
#[cfg_attr(miri, ignore)] // pipe I/O calls foreign functions
fn os_pipe_read() -> anyhow::Result<()> {
    // Obtain an `UnsafeReadable` and use it to read.
    let stream = write_to_pipe()?;
    let mut buf = String::new();
    unsafe { stream.as_unsafe_handle().as_readable() }.read_to_string(&mut buf)?;
    assert_eq!(buf, "hello, world");

    // Obtain an `UnsafeSocket` and use it to construct a temporary manually-drop
    // `PipeReader` to read.
    let stream = write_to_pipe()?;
    let mut buf = String::new();
    stream.as_file().read_to_string(&mut buf)?;
    assert_eq!(buf, "hello, world");

    // Similar, but do it manually.
    let stream = write_to_pipe()?;
    let mut buf = String::new();
    ManuallyDrop::new(unsafe { std::fs::File::from_unsafe_file(stream.as_unsafe_file()) })
        .read_to_string(&mut buf)?;
    assert_eq!(buf, "hello, world");

    // Similar, but use the Posix-ish-specific type.
    #[cfg(not(windows))]
    {
        let stream = write_to_pipe()?;
        let mut buf = String::new();
        ManuallyDrop::new(unsafe {
            std::fs::File::from_raw_fd(stream.as_unsafe_handle().as_raw_fd())
        })
        .read_to_string(&mut buf)?;
        assert_eq!(buf, "hello, world");
    }

    // Similar, but use the Windows-specific type.
    #[cfg(windows)]
    {
        let stream = write_to_pipe()?;
        let mut buf = String::new();
        ManuallyDrop::new(unsafe {
            std::fs::File::from_raw_handle(
                match stream.as_unsafe_handle().as_raw_handle_or_socket() {
                    RawHandleOrSocket::Handle(handle) => handle,
                    RawHandleOrSocket::Socket(_) => panic!(),
                },
            )
        })
        .read_to_string(&mut buf)?;
        assert_eq!(buf, "hello, world");
    }

    Ok(())
}
