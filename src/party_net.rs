//! Bounded, length-prefixed JSON over nonblocking TCP. No socket I/O on a render frame waits.
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    net::TcpStream,
};

pub const MAX_FRAME: usize = 2 * 1024 * 1024;
pub const MAX_COMMAND: usize = 2048;

pub struct Connection {
    stream: TcpStream,
    incoming: Vec<u8>,
    outgoing: VecDeque<Vec<u8>>,
    offset: usize,
    queued: usize,
    limit: usize,
    eof: bool,
}

impl Connection {
    pub fn new(stream: TcpStream, limit: usize) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;
        Ok(Self {
            stream,
            incoming: vec![],
            outgoing: VecDeque::new(),
            offset: 0,
            queued: 0,
            limit,
            eof: false,
        })
    }

    pub fn pending(&self) -> bool {
        self.queued != 0
    }

    pub fn send(&mut self, value: &impl Serialize) -> io::Result<()> {
        let data = serde_json::to_vec(value).map_err(io::Error::other)?;
        if data.len() > MAX_FRAME || self.queued + data.len() + 4 > MAX_FRAME * 2 {
            return Err(io::Error::other("Connection send queue exceeded its limit"));
        }
        let mut frame = Vec::with_capacity(data.len() + 4);
        frame.extend_from_slice(&(data.len() as u32).to_be_bytes());
        frame.extend_from_slice(&data);
        self.queued += frame.len();
        self.outgoing.push_back(frame);
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        let mut budget: usize = 256 * 1024;
        while let Some(frame) = self.outgoing.front() {
            let end = frame.len().min(self.offset + budget);
            match self.stream.write(&frame[self.offset..end]) {
                Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                Ok(n) => {
                    self.offset += n;
                    self.queued -= n;
                    budget -= n;
                    if self.offset == frame.len() {
                        self.outgoing.pop_front();
                        self.offset = 0;
                    }
                    if budget == 0 {
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub fn receive<T: DeserializeOwned>(&mut self) -> io::Result<Vec<T>> {
        let mut bytes = [0; 8192];
        for _ in 0..32 {
            match self.stream.read(&mut bytes) {
                Ok(0) => {
                    self.eof = true;
                    break;
                }
                Ok(n) => {
                    self.incoming.extend_from_slice(&bytes[..n]);
                    if self.incoming.len() > self.limit + 256 * 1024 {
                        return Err(io::Error::other("Receive buffer exceeded its limit"));
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
        let mut result = vec![];
        let mut consumed = 0;
        while self.incoming.len() - consumed >= 4 {
            let len = u32::from_be_bytes(self.incoming[consumed..consumed + 4].try_into().unwrap())
                as usize;
            if len == 0 || len > self.limit {
                return Err(io::Error::other("Invalid network frame length"));
            }
            if self.incoming.len() - consumed < len + 4 {
                break;
            }
            result.push(
                serde_json::from_slice(&self.incoming[consumed + 4..consumed + 4 + len])
                    .map_err(io::Error::other)?,
            );
            consumed += len + 4;
            if result.len() >= 128 {
                break;
            }
        }
        self.incoming.drain(..consumed);
        if self.eof && result.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Server or player disconnected",
            ));
        }
        Ok(result)
    }
}
