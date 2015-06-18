/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The Servo Binary Serialization Format: an extremely simple serialization format that is
//! optimized for speed above all else.

use ipc;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rustc_serialize::{Decoder, Encoder};
use std::char;
use std::io::{Error, Read, Write};

pub struct ServoEncoder<'a> {
    pub writer: &'a mut (Write + 'a),
}

impl<'a> Encoder for ServoEncoder<'a> {
    type Error = Error;

    #[inline]
    fn emit_nil(&mut self) -> Result<(),Error> {
        Ok(())
    }
    #[inline]
    fn emit_usize(&mut self, value: usize) -> Result<(),Error> {
        self.writer.write_u64::<LittleEndian>(value as u64).map_err(Error::from)
    }
    #[inline]
    fn emit_u64(&mut self, value: u64) -> Result<(),Error> {
        self.writer.write_u64::<LittleEndian>(value).map_err(Error::from)
    }
    #[inline]
    fn emit_u32(&mut self, value: u32) -> Result<(),Error> {
        self.writer.write_u32::<LittleEndian>(value).map_err(Error::from)
    }
    #[inline]
    fn emit_u16(&mut self, value: u16) -> Result<(),Error> {
        self.writer.write_u16::<LittleEndian>(value).map_err(Error::from)
    }
    #[inline]
    fn emit_u8(&mut self, value: u8) -> Result<(),Error> {
        self.writer.write_u8(value).map_err(Error::from)
    }
    #[inline]
    fn emit_isize(&mut self, value: isize) -> Result<(),Error> {
        self.writer.write_i64::<LittleEndian>(value as i64).map_err(Error::from)
    }
    #[inline]
    fn emit_i64(&mut self, value: i64) -> Result<(),Error> {
        self.writer.write_i64::<LittleEndian>(value).map_err(Error::from)
    }
    #[inline]
    fn emit_i32(&mut self, value: i32) -> Result<(),Error> {
        self.writer.write_i32::<LittleEndian>(value).map_err(Error::from)
    }
    #[inline]
    fn emit_i16(&mut self, value: i16) -> Result<(),Error> {
        self.writer.write_i16::<LittleEndian>(value).map_err(Error::from)
    }
    #[inline]
    fn emit_i8(&mut self, value: i8) -> Result<(),Error> {
        self.writer.write_i8(value).map_err(Error::from)
    }
    #[inline]
    fn emit_bool(&mut self, value: bool) -> Result<(),Error> {
        self.writer.write_u8(value as u8).map_err(Error::from)
    }
    #[inline]
    fn emit_f64(&mut self, value: f64) -> Result<(),Error> {
        self.writer.write_f64::<LittleEndian>(value).map_err(Error::from)
    }
    #[inline]
    fn emit_f32(&mut self, value: f32) -> Result<(),Error> {
        self.writer.write_f32::<LittleEndian>(value).map_err(Error::from)
    }
    #[inline]
    fn emit_char(&mut self, value: char) -> Result<(),Error> {
        self.writer.write_u32::<LittleEndian>(value as u32).map_err(Error::from)
    }
    #[inline]
    fn emit_str(&mut self, value: &str) -> Result<(),Error> {
        try!(self.writer.write_u64::<LittleEndian>(value.len() as u64));
        self.writer.write_all(value.as_bytes())
    }
    #[inline]
    fn emit_enum<F>(&mut self, _: &str, f: F) -> Result<(),Error>
                    where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        f(self)
    }
    #[inline]
    fn emit_enum_variant<F>(&mut self, _: &str, variant_id: usize, _: usize, f: F)
                            -> Result<(),Error>
                            where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        try!(self.writer.write_u16::<LittleEndian>(variant_id as u16));
        f(self)
    }
    #[inline]
    fn emit_enum_variant_arg<F>(&mut self, _: usize, f: F) -> Result<(),Error>
                                where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        f(self)
    }
    #[inline]
    fn emit_enum_struct_variant<F>(&mut self, _: &str, variant_id: usize, _: usize, f: F)
                                   -> Result<(),Error>
                                   where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        try!(self.writer.write_u16::<LittleEndian>(variant_id as u16));
        f(self)
    }
    #[inline]
    fn emit_enum_struct_variant_field<F>(&mut self, _: &str, _: usize, f: F) -> Result<(),Error>
                                         where F: FnOnce(&mut ServoEncoder<'a>)
                                                         -> Result<(),Error> {
        f(self)
    }
    #[inline]
    fn emit_struct<F>(&mut self, _: &str, _: usize, f: F) -> Result<(),Error>
                      where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        f(self)
    }
    #[inline]
    fn emit_struct_field<F>(&mut self, _: &str, _: usize, f: F) -> Result<(),Error>
                            where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        f(self)
    }
    #[inline]
    fn emit_tuple<F>(&mut self, _: usize, f: F) -> Result<(),Error>
                     where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        f(self)
    }
    #[inline]
    fn emit_tuple_arg<F>(&mut self, _: usize, f: F) -> Result<(),Error>
                         where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        f(self)
    }
    #[inline]
    fn emit_tuple_struct<F>(&mut self, _: &str, _: usize, f: F) -> Result<(),Error>
                            where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        f(self)
    }
    #[inline]
    fn emit_tuple_struct_arg<F>(&mut self, _: usize, f: F) -> Result<(),Error>
                                where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        f(self)
    }
    #[inline]
    fn emit_option<F>(&mut self, f: F) -> Result<(),Error>
                      where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        f(self)
    }
    #[inline]
    fn emit_option_none(&mut self) -> Result<(),Error> {
        self.writer.write_u8(0).map_err(Error::from)
    }
    #[inline]
    fn emit_option_some<F>(&mut self, f: F) -> Result<(),Error>
                           where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        try!(self.writer.write_u8(1).map_err(Error::from));
        f(self)
    }
    #[inline]
    fn emit_seq<F>(&mut self, len: usize, f: F) -> Result<(), Error>
                   where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        try!(self.writer.write_u64::<LittleEndian>(len as u64).map_err(Error::from));
        f(self)
    }
    #[inline]
    fn emit_seq_elt<F>(&mut self, _: usize, f: F) -> Result<(),Error>
                       where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        f(self)
    }
    #[inline]
    fn emit_map<F>(&mut self, len: usize, f: F) -> Result<(),Error>
                   where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        try!(self.writer.write_u64::<LittleEndian>(len as u64).map_err(Error::from));
        f(self)
    }
    #[inline]
    fn emit_map_elt_key<F>(&mut self, _: usize, f: F) -> Result<(),Error>
                           where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        f(self)
    }
    #[inline]
    fn emit_map_elt_val<F>(&mut self, _: usize, f: F) -> Result<(),Error>
                           where F: FnOnce(&mut ServoEncoder<'a>) -> Result<(),Error> {
        f(self)
    }
}

pub struct ServoDecoder<'a> {
    pub reader: &'a mut (Read + 'a),
}

impl<'a> Decoder for ServoDecoder<'a> {
    type Error = Error;

    #[inline]
    fn read_nil(&mut self) -> Result<(),Error> {
        Ok(())
    }
    #[inline]
    fn read_usize(&mut self) -> Result<usize,Error> {
        match self.reader.read_u64::<LittleEndian>() {
            Ok(value) => Ok(value as usize),
            Err(error) => Err(Error::from(error)),
        }
    }
    #[inline]
    fn read_u64(&mut self) -> Result<u64,Error> {
        self.reader.read_u64::<LittleEndian>().map_err(Error::from)
    }
    #[inline]
    fn read_u32(&mut self) -> Result<u32,Error> {
        self.reader.read_u32::<LittleEndian>().map_err(Error::from)
    }
    #[inline]
    fn read_u16(&mut self) -> Result<u16,Error> {
        self.reader.read_u16::<LittleEndian>().map_err(Error::from)
    }
    #[inline]
    fn read_u8(&mut self) -> Result<u8,Error> {
        self.reader.read_u8().map_err(Error::from)
    }
    #[inline]
    fn read_isize(&mut self) -> Result<isize,Error> {
        match self.reader.read_i64::<LittleEndian>() {
            Ok(value) => Ok(value as isize),
            Err(error) => Err(Error::from(error)),
        }
    }
    #[inline]
    fn read_i64(&mut self) -> Result<i64,Error> {
        self.reader.read_i64::<LittleEndian>().map_err(Error::from)
    }
    #[inline]
    fn read_i32(&mut self) -> Result<i32,Error> {
        self.reader.read_i32::<LittleEndian>().map_err(Error::from)
    }
    #[inline]
    fn read_i16(&mut self) -> Result<i16,Error> {
        self.reader.read_i16::<LittleEndian>().map_err(Error::from)
    }
    #[inline]
    fn read_i8(&mut self) -> Result<i8,Error> {
        self.reader.read_i8().map_err(Error::from)
    }
    #[inline]
    fn read_bool(&mut self) -> Result<bool,Error> {
        Ok(try!(self.reader.read_u8()) != 0)
    }
    #[inline]
    fn read_f64(&mut self) -> Result<f64,Error> {
        self.reader.read_f64::<LittleEndian>().map_err(Error::from)
    }
    #[inline]
    fn read_f32(&mut self) -> Result<f32,Error> {
        self.reader.read_f32::<LittleEndian>().map_err(Error::from)
    }
    #[inline]
    fn read_char(&mut self) -> Result<char,Error> {
        Ok(char::from_u32(try!(self.reader.read_u32::<LittleEndian>())).unwrap())
    }
    #[inline]
    fn read_str(&mut self) -> Result<String,Error> {
        let len = try!(self.reader.read_u64::<LittleEndian>().map_err(Error::from));
        let bytes = try!(ipc::read_exact(self.reader, len as usize));
        Ok(String::from_utf8(bytes).unwrap())
    }
    #[inline]
    fn read_enum<T,F>(&mut self, _: &str, f: F) -> Result<T,Error>
                      where F: FnOnce(&mut ServoDecoder<'a>) -> Result<T,Error> {
        f(self)
    }
    #[inline]
    fn read_enum_variant<T,F>(&mut self, _: &[&str], f: F) -> Result<T,Error>
                              where F: FnOnce(&mut ServoDecoder<'a>, usize) -> Result<T,Error> {
        let index = try!(self.reader.read_u16::<LittleEndian>());
        f(self, index as usize)
    }
    #[inline]
    fn read_enum_variant_arg<T,F>(&mut self, _: usize, f: F) -> Result<T,Error>
                                  where F: FnOnce(&mut ServoDecoder<'a>) -> Result<T,Error> {
        f(self)
    }
    #[inline]
    fn read_enum_struct_variant<T,F>(&mut self, _: &[&str], f: F) -> Result<T,Error>
                                     where F: FnOnce(&mut ServoDecoder<'a>, usize)
                                                     -> Result<T,Error> {
        let index = try!(self.reader.read_u16::<LittleEndian>());
        f(self, index as usize)
    }
    #[inline]
    fn read_enum_struct_variant_field<T,F>(&mut self, _: &str, _: usize, f: F) -> Result<T,Error>
                                           where F: FnOnce(&mut ServoDecoder<'a>)
                                                           -> Result<T,Error> {
        f(self)
    }
    #[inline]
    fn read_struct<T,F>(&mut self, _: &str, _: usize, f: F) -> Result<T,Error>
                        where F: FnOnce(&mut ServoDecoder<'a>) -> Result<T,Error> {
        f(self)
    }
    #[inline]
    fn read_struct_field<T,F>(&mut self, _: &str, _: usize, f: F) -> Result<T,Error>
                              where F: FnOnce(&mut ServoDecoder<'a>) -> Result<T,Error> {
        f(self)
    }
    #[inline]
    fn read_tuple<T,F>(&mut self, _: usize, f: F) -> Result<T,Error>
                       where F: FnOnce(&mut ServoDecoder<'a>) -> Result<T,Error> {
        f(self)
    }
    #[inline]
    fn read_tuple_arg<T,F>(&mut self, _: usize, f: F) -> Result<T,Error>
                           where F: FnOnce(&mut ServoDecoder<'a>) -> Result<T,Error> {
        f(self)
    }
    #[inline]
    fn read_tuple_struct<T,F>(&mut self, _: &str, _: usize, f: F) -> Result<T,Error>
                              where F: FnOnce(&mut ServoDecoder<'a>) -> Result<T,Error> {
        f(self)
    }
    #[inline]
    fn read_tuple_struct_arg<T,F>(&mut self, _: usize, f: F) -> Result<T,Error>
                                  where F: FnOnce(&mut ServoDecoder<'a>) -> Result<T,Error> {
        f(self)
    }
    #[inline]
    fn read_option<T,F>(&mut self, f: F) -> Result<T,Error>
                        where F: FnOnce(&mut ServoDecoder<'a>, bool) -> Result<T,Error> {
        let is_some = try!(self.reader.read_u8()) != 0;
        f(self, is_some)
    }
    #[inline]
    fn read_seq<T,F>(&mut self, f: F) -> Result<T,Error>
                     where F: FnOnce(&mut ServoDecoder<'a>, usize) -> Result<T,Error> {
        let len = try!(self.reader.read_u64::<LittleEndian>().map_err(Error::from));
        f(self, len as usize)
    }
    #[inline]
    fn read_seq_elt<T,F>(&mut self, _: usize, f: F) -> Result<T,Error>
                         where F: FnOnce(&mut ServoDecoder<'a>) -> Result<T,Error> {
        f(self)
    }
    #[inline]
    fn read_map<T,F>(&mut self, f: F) -> Result<T,Error>
                     where F: FnOnce(&mut ServoDecoder<'a>, usize) -> Result<T,Error> {
        let len = try!(self.reader.read_u64::<LittleEndian>().map_err(Error::from));
        f(self, len as usize)
    }
    #[inline]
    fn read_map_elt_key<T,F>(&mut self, _: usize, f: F) -> Result<T,Error>
                             where F: FnOnce(&mut ServoDecoder<'a>) -> Result<T,Error> {
        f(self)
    }
    #[inline]
    fn read_map_elt_val<T,F>(&mut self, _: usize, f: F) -> Result<T,Error>
                             where F: FnOnce(&mut ServoDecoder<'a>) -> Result<T,Error> {
        f(self)
    }
    #[inline]
    fn error(&mut self, _: &str) -> Error {
        Error::from_raw_os_error(0)
    }
}

