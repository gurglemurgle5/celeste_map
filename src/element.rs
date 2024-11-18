use bytes::{Buf, Bytes};
use std::{collections::HashMap, path::Path};

#[derive(Debug)]
pub struct Element {
    pub name: String,
    pub attributes: HashMap<String, Value>,
    pub children: Vec<Element>,
}

impl Element {
    pub fn from_file<T: AsRef<Path>>(path: T) -> Element {
        let data = std::fs::read(path).unwrap();
        let mut data = Bytes::from(data);

        data.get_string();
        let _package = data.get_string();

        let num_strings = data.get_i16_le() as usize;
        let string_lookup: Vec<String> = (0..num_strings).map(|_| data.get_string()).collect();
        let element = Element::read(&mut data, &string_lookup);
        element
    }

    fn read(data: &mut Bytes, string_lookup: &[String]) -> Element {
        let name = string_lookup[data.get_i16_le() as usize].clone();

        let num_attributes = data.get_u8();
        let attributes = (0..num_attributes)
            .map(|_| {
                let key = string_lookup[data.get_i16_le() as usize].clone();
                let value_type = data.get_u8();
                let value = match value_type {
                    0 => Value::Bool(data.get_u8() != 0),
                    1 => Value::Int(data.get_u8().into()),
                    2 => Value::Int(data.get_i16_le().into()),
                    3 => Value::Int(data.get_i32_le()),
                    4 => Value::Float(data.get_f32()),
                    5 => Value::String(string_lookup[data.get_i16_le() as usize].clone()),
                    6 => Value::String(data.get_string()),
                    7 => {
                        let count = data.get_i16_le() / 2;
                        let buf = (0..count)
                            .map(|_| {
                                let repeat = data.get_u8();
                                let byte = (data.get_u8() as char).to_string();
                                byte.repeat(repeat as usize)
                            })
                            .collect();
                        Value::String(buf)
                    }
                    _ => todo!("error"),
                };
                (key, value)
            })
            .collect();

        let num_children = data.get_i16_le();
        let children = (0..num_children)
            .map(|_| Element::read(data, string_lookup))
            .collect();

        Element {
            name,
            attributes,
            children,
        }
    }
}

#[derive(Debug)]

pub enum Value {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
}

impl Value {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(val) => Some(*val),
            _ => None,
        }
    }
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::Int(val) => Some(*val),
            _ => None,
        }
    }
    pub fn as_float(&self) -> Option<f32> {
        match self {
            Value::Float(val) => Some(*val),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(val) => Some(val),
            _ => None,
        }
    }
}

trait BufExt {
    fn get_string(&mut self) -> String;
    fn get_var_int(&mut self) -> i32;
}

impl<T: Buf> BufExt for T {
    fn get_string(&mut self) -> String {
        let length = self.get_var_int();
        let mut buf = Vec::with_capacity(length as usize);
        for _ in 0..length {
            buf.push(self.get_u8());
        }
        String::from_utf8(buf).unwrap()
    }

    // stolen from wiki.vg/Data_types#VarInt_and_VarLong
    fn get_var_int(&mut self) -> i32 {
        let mut value = 0;
        let mut position = 0;
        let mut current_byte;

        loop {
            current_byte = self.get_u8() as i32;
            value |= (current_byte & 0x7F) << position;

            if (current_byte & 0x80) == 0 {
                break;
            }

            position += 7;

            if position >= 32 {
                todo!()
            };
        }

        value
    }
}
