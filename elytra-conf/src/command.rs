use num_enum::TryFromPrimitive;
use core::panic;
use core::prelude::rust_2024::{*};
use core::future::Future;

use crate::{
    config::{MESSAGE_LENGTH, PAYLOAD_SIZE},
    field::FieldValue
};

pub struct CommandResponse {
    bytes: [u8; MESSAGE_LENGTH],
    len: usize,
}

impl CommandResponse {
    pub const OK: Self = Self::new();

    pub fn error(error: CommandError) -> Self {
        let mut cr = Self::new();
        cr.bytes[0] = 0;
        cr.push(error as u8);
        #[cfg(feature = "alloc")]
        {
            let error_str = alloc::format!("{:?}", error);
            cr.extend(error_str.bytes());
        }
        cr
    }

    pub const fn new() -> Self {
        let mut bytes = [0u8; MESSAGE_LENGTH];
        bytes[0] = 1;
        Self {
            bytes,
            len: 1
        }
    }

    pub fn from_payload<T: IntoIterator<Item = u8>>(payload: T) -> Self {
        let mut cr = CommandResponse::new();
        cr.extend(payload);
        cr
    }

    pub fn from_field_value(field_value: FieldValue) -> Self {
        Self {
            bytes: field_value.into_message_bytes(),
            len: MESSAGE_LENGTH,
        }
    }

    pub fn push(&mut self, value: u8) {
        if self.len >= MESSAGE_LENGTH {
            panic!("Command response exceeded maximum size");
        }
        self.bytes[self.len] = value;
        self.len += 1;
    }

    pub fn extend<T: IntoIterator<Item = u8>>(&mut self, value: T) {
        for b in value.into_iter() {
            self.push(b)
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl From<Option<&'static str>> for CommandResponse {
    fn from(value: Option<&'static str>) -> Self {
        match value {
            Some(text) => text.into(),
            None => CommandResponse::new(),
        }
    }
}

impl From<&'static str> for CommandResponse {
    fn from(value: &'static str) -> Self {
        let str_len: usize = value.floor_char_boundary(value.len().min(PAYLOAD_SIZE));
        CommandResponse::from_payload(value.bytes().take(str_len))
    }
}

#[repr(u8)]
#[derive(TryFromPrimitive)]
pub enum Command {
    ReadProp = 'r' as u8,
    DescProp = 'c' as u8,
    WriteProp = 'w' as u8,
    ReadInfo = 'R' as u8,
    DescInfo = 'I' as u8,
    WriteInfo = 'W' as u8,
    DescSection = 's' as u8,
    DescAction = 'a' as u8,
    Query = 'q' as u8,
    Action = 'A' as u8,
    Info = 'i' as u8,
    Noop = 0,
}

#[repr(u8)]
#[derive(Debug, strum::Display)]
pub enum CommandError {
    InvalidCommand = 1,
    MissingArgument = 2,
    InvalidData = 3,
    InvalidField = 4,
    InvalidSection = 5,
    InvalidAction = 6,
    InvalidEntry = 7,
    InvalidQuery = 8,
    InvalidOption = 9,
    NotSupported = 10,
    Failed = 11,
    NoContent = 12,
}

pub trait CommandHandler<PI, II, AI> {
    fn noop(&mut self) -> 
        impl Future<Output = ()> + Send;
        
    fn read_prop(&mut self, prop_field: PI) 
        -> impl Future<Output = Result<FieldValue, CommandError>> + Send;

    fn write_prop(&mut self, prop_field: PI, value: FieldValue) 
        -> impl Future<Output = Result<(), CommandError>> + Send;

    fn read_info(&mut self, info_field: II) 
        -> impl Future<Output = Result<FieldValue, CommandError>> + Send;

    fn write_info(&mut self, info_field: II, value: FieldValue) 
        -> impl Future<Output = Result<(), CommandError>> + Send;
    
    fn do_action(&mut self, action: AI)
        -> impl Future<Output = Result<(), CommandError>> + Send;
}