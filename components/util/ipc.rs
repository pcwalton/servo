/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use platform::unix::ipc::ServoUnixSocket;
use sbsf::{ServoDecoder, ServoEncoder};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use libc::c_int;
use rustc_serialize::{Decodable, Encodable};
use std::io::{self, Error, Read, Write};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, MutexGuard};

pub struct IpcReceiver<T> {
    pipe: Arc<Mutex<ServoUnixSocket>>,
    phantom: PhantomData<T>,
}

impl<T> Clone for IpcReceiver<T> {
    fn clone(&self) -> IpcReceiver<T> {
        IpcReceiver {
            pipe: self.pipe.clone(),
            phantom: PhantomData,
        }
    }
}

pub struct IpcSender<T> {
    pipe: Arc<Mutex<ServoUnixSocket>>,
    phantom: PhantomData<T>,
}

impl<T> Clone for IpcSender<T> {
    fn clone(&self) -> IpcSender<T> {
        IpcSender {
            pipe: self.pipe.clone(),
            phantom: PhantomData,
        }
    }
}

/// Creates a new IPC channel and returns the receiving and sending ends of it respectively.
pub fn channel<T>() -> (IpcReceiver<T>, IpcSender<T>) {
    let (first, second) = ServoUnixSocket::pair().unwrap();
    (IpcReceiver {
        pipe: Arc::new(Mutex::new(first)),
        phantom: PhantomData,
    }, IpcSender {
        pipe: Arc::new(Mutex::new(second)),
        phantom: PhantomData,
    })
}

impl<T> IpcReceiver<T> where T: Sized + for<'a> Decodable {
    /// Constructs one end of an IPC channel from a file descriptor.
    pub fn from_fd(fd: c_int) -> IpcReceiver<T> {
        IpcReceiver {
            pipe: Arc::new(Mutex::new(ServoUnixSocket::from_fd(fd))),
            phantom: PhantomData,
        }
    }

    /// Constructs an IPC receiver from a raw Unix socket.
    pub fn from_socket(socket: ServoUnixSocket) -> IpcReceiver<T> {
        IpcReceiver {
            pipe: Arc::new(Mutex::new(socket)),
            phantom: PhantomData,
        }
    }

    /// Returns the raw file descriptor backing this IPC receiver.
    pub fn fd(&self) -> c_int {
        self.pipe.lock().unwrap().fd()
    }

    /// Returns the raw Unix socket backing this IPC receiver.
    pub fn socket<'b>(&'b self) -> MutexGuard<'b,ServoUnixSocket> {
        self.pipe.lock().unwrap()
    }

    pub fn recv(&self) -> T {
        match self.recv_opt() {
            Ok(msg) => msg,
            Err(err) => panic!("failed to receive over IPC: {}", err),
        }
    }

    pub fn recv_opt(&self) -> Result<T,Error> {
        let mut pipe = self.pipe.lock().unwrap();
        let size = try!(pipe.read_u64::<LittleEndian>().map_err(Error::from));
        let bytes: Vec<u8> = try!(read_exact(&mut *pipe, size as usize));
        let mut reader: &[u8] = &*bytes;
        let mut decoder = ServoDecoder {
            reader: &mut reader,
        };
        Decodable::decode(&mut decoder)
    }
}

impl<T> IpcSender<T> where T: for<'a> Encodable {
    /// Constructs one end of an IPC channel from a file descriptor.
    pub fn from_fd(fd: c_int) -> IpcSender<T> {
        IpcSender {
            pipe: Arc::new(Mutex::new(ServoUnixSocket::from_fd(fd))),
            phantom: PhantomData,
        }
    }

    /// Constructs an IPC sender from a raw Unix socket.
    pub fn from_socket(socket: ServoUnixSocket) -> IpcSender<T> {
        IpcSender {
            pipe: Arc::new(Mutex::new(socket)),
            phantom: PhantomData,
        }
    }

    /// Returns the raw file descriptor backing this IPC sender.
    pub fn fd(&self) -> c_int {
        self.pipe.lock().unwrap().fd()
    }

    /// Returns the raw Unix socket backing this IPC sender.
    pub fn socket<'b>(&'b self) -> MutexGuard<'b,ServoUnixSocket> {
        self.pipe.lock().unwrap()
    }

    pub fn send(&self, msg: T) {
        match self.send_opt(msg) {
            Ok(()) => {}
            Err(err) => panic!("failed to send over IPC: {}", err),
        }
    }

    pub fn send_opt(&self, msg: T) -> Result<(),Error> {
        let mut writer = Vec::new();
        {
            let mut encoder = ServoEncoder {
                writer: &mut writer,
            };
            try!(msg.encode(&mut encoder));
        }
        let mut pipe = self.pipe.lock().unwrap();
        try!(pipe.write_u64::<LittleEndian>(writer.len() as u64));
        pipe.write(&writer).map(|_| ())
    }
}

pub fn read_exact<R: Read + ?Sized>(r: &mut R, sz: usize) -> io::Result<Vec<u8>> {
    let mut v = Vec::with_capacity(sz);
    try!(r.take(sz as u64).read_to_end(&mut v));
    assert_eq!(v.len(), sz);
    Ok(v)
}

