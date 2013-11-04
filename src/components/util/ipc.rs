/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Low-level inter-process communication support. This is done using `libc` for two reasons: (1)
//! the compositor end is runtimeless; (2) Rust is currently missing features necessary for this
//! to work, namely selecting over pipes and `uv` channels.
//!
//! Most of this should be moved to the Rust standard library eventually.

use extra::ebml;
use extra::serialize::{Decodable, Decoder, Encodable, Encoder};
use std::cast;
use std::libc::{c_int, c_void, size_t, ssize_t, time_t};
use std::ptr;
use std::rt::io::mem::{BufReader, MemWriter};
use std::rt::io::{Decorator, Reader, Writer};
use std::vec;

// C function declarations.
extern {
    fn socketpair(domain: c_int, Type: c_int, protocol: c_int, socket_vector: *mut c_int) -> c_int;
    fn recv(socket: c_int, buffer: *c_void, length: size_t, flags: c_int) -> ssize_t;
    fn send(socket: c_int, buffer: *c_void, length: size_t, flags: c_int) -> ssize_t;
    fn close(socket: c_int) -> c_int;
    fn select(nfds: c_int,
              readfds: *mut fd_set,
              writefds: *mut fd_set,
              errorfds: *mut fd_set,
              timeout: *mut timeval)
              -> c_int;
}

static AF_UNIX: u8 = 1;

static SOCK_STREAM: c_int = 1;

type fd_set = [i32, ..FD_SETSIZE / NFDBITS];

static FD_SETSIZE: uint = 1024;

static NFDBITS: uint = 32;

type suseconds_t = i32;

struct timeval {
    tv_sec: time_t,
    tv_usec: suseconds_t,
}

/// Allows serialized messages to be read from I/O readers.
pub trait MsgReader {
    /// Reads and deserializes a message from a byte stream.
    fn read_msg<M:Decodable<ebml::reader::Decoder>>(&mut self) -> M;
}

impl<R:Reader> MsgReader for R {
    fn read_msg<M:Decodable<ebml::reader::Decoder>>(&mut self) -> M {
        let mut len_buf = [0u8, ..4];
        self.read(len_buf);
        let mut msg_buf = vec::from_elem(BufReader::new(len_buf).read_le_u32() as uint, 0u8);
        self.read(msg_buf);
        let mut decoder = ebml::reader::Decoder(ebml::reader::Doc(@msg_buf));
        Decodable::decode(&mut decoder)
    }
}

/// Allows serialized messages to be written to I/O writers.
pub trait MsgWriter {
    /// Serializes and writes a message to a byte stream.
    fn write_msg<M:Encodable<ebml::writer::Encoder>>(&mut self, msg: M);
}

impl<W:Writer> MsgWriter for W {
    fn write_msg<M:Encodable<ebml::writer::Encoder>>(&mut self, msg: M) {
        let mem_writer = @mut MemWriter::new();
        {
            let mut encoder = ebml::writer::Encoder(mem_writer);
            msg.encode(&mut encoder);
        }
        let msg_buf = (*mem_writer.inner_ref()).clone();
        assert!(msg_buf.len() <= 0xffffffff);
        let mut len_buf = vec::from_fn(4, |i| (msg_buf.len() >> (i * 8)) as u8);
        len_buf.push_all_move(msg_buf);
        self.write(len_buf);
    }
}

/// A Unix client connection.
#[deriving(Clone)]
pub struct NativeUnixStream {
    socket: c_int,
}

impl NativeUnixStream {
    /// Opens a pair of connected sockets.
    ///
    /// Be warned: This performs synchronous I/O.
    #[fixed_stack_segment]
    pub fn pair() -> (NativeUnixStream, NativeUnixStream) {
        unsafe {
            let mut sockets = [0, 0];
            assert!(socketpair(AF_UNIX as c_int, SOCK_STREAM, 0, &mut sockets[0]) == 0);
            let first = NativeUnixStream {
                socket: sockets[0],
            };
            let second = NativeUnixStream {
                socket: sockets[1],
            };
            (first, second)
        }
    }

    /// Returns true if a message is waiting to be received and false otherwise.
    #[fixed_stack_segment]
    pub fn peek(&self) -> bool {
        unsafe {
            let mut fd_set = [0, ..32];
            fd_set[self.socket as uint / NFDBITS] = 1 << (self.socket as uint % NFDBITS);
            let mut timeval = timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            select(self.socket + 1, &mut fd_set, ptr::mut_null(), ptr::mut_null(), &mut timeval) !=
                0
        }
    }

    /// Closes the socket. After this call, subsequent attempts to read will fail.
    #[fixed_stack_segment]
    pub fn close(&self) {
        unsafe {
            let _ = close(self.socket);
        }
    }
}

impl Reader for NativeUnixStream {
    /// Receives data from the server. Fails on error.
    ///
    /// Be warned: This performs synchronous I/O.
    #[fixed_stack_segment]
    fn read(&mut self, buf: &mut [u8]) -> Option<uint> {
        unsafe {
            let mut offset = 0;
            while offset < buf.len() {
                let nread = recv(self.socket,
                                 cast::transmute(ptr::mut_offset(vec::raw::to_mut_ptr(buf),
                                                                 offset as int)),
                                 (buf.len() - offset) as size_t,
                                 0);
                if nread == 0 {
                    return None
                }
                assert!(nread > 0);
                offset += nread as uint;
            }
            Some(offset as uint)
        }
    }

    fn eof(&mut self) -> bool {
        false
    }
}

impl Writer for NativeUnixStream {
    /// Sends data to the server. Fails on error.
    ///
    /// Be warned: This performs synchronous I/O.
    #[fixed_stack_segment]
    fn write(&mut self, buf: &[u8]) {
        unsafe {
            let mut offset = 0;
            while offset < buf.len() {
                let nwritten = send(self.socket,
                                    cast::transmute(&buf[offset]),
                                    (buf.len() - offset) as size_t,
                                    0);
                assert!(nwritten > 0);
                offset += nwritten as uint;
            }
        }
    }

    fn flush(&mut self) {}
}

