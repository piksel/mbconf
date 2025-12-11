use core::{prelude::rust_2024::*};
#[cfg(feature = "alloc")]
use alloc::borrow::ToOwned;
use log::warn;
use elytra_bytepack::Cursor;
use crate::{
    command::CommandResponse, config::MESSAGE_LENGTH, entry::{Constraints, EntryDesc, EntryVariant}, values::{DefaultValue, ValueType}
};

pub struct Options {
    buf: [u16; 31],
    len: u8
}
impl Options {
    fn as_slice(&self) -> &[u16] {
        &self.buf[0..self.len as usize]
    }
}

#[derive(Debug)]
pub struct FieldValue {
    desc: &'static EntryDesc,
    data: [u8; MESSAGE_LENGTH],
}

impl FieldValue{
    pub const fn new(desc: &'static EntryDesc) -> Self {
        Self {
            desc,
            data: [0u8; MESSAGE_LENGTH]
        }
    }

    #[allow(unused)]
    const fn len(&self) -> usize {
        self.data[0] as usize
    }

    const fn set_len(&mut self, len: usize) {
        self.data[0] = len as u8
    }

    pub fn from_store(desc: &'static EntryDesc, bytes: [u8; MESSAGE_LENGTH]) -> Self {
        let mut fv = Self {
            desc,
            data: bytes
        };
        if fv.is_empty() {
            use DefaultValue::{*};

            fv.data[0] = match desc.default {
                Bytes(bytes) => {
                    fv.data[1..].copy_from_slice(bytes);
                    bytes.len() as u8
                },
                Empty => 0,
                Text(text) => {
                    fv.data[1..].copy_from_slice(text.as_bytes());
                    text.len() as u8
                },
                Integer(integer) => {
                    fv.data[1..].copy_from_slice(&integer.to_le_bytes());
                    4
                }
                Options(items) => {
                    let mut cursor = Cursor::new(&mut fv.data[1..]);
                    for i in 0..items.len() {
                        cursor.write(&items[i].to_le_bytes()).unwrap();
                    }
                    items.len() as u8
                },
            }
        }
        fv
    }

    pub fn from_message(desc: &'static EntryDesc, bytes: &[u8]) -> Self {
        let mut fv = Self {
            desc,
            data: [0u8; 64]
        };
        
        fv.data[0] = match desc.variant {
            EntryVariant::Field(vt) if vt.is_options() => {
                bytes.len() as u8 / 2
            },
            _ => {
                bytes.len() as u8
            }
        };
        for i in 0..bytes.len() {
            fv.data[i + 1] = bytes[i]
        }
        fv.clamp();
        fv
    }

    pub fn into_store_bytes(self) -> [u8; MESSAGE_LENGTH] {
        self.data
    }

    pub fn into_message_bytes(mut self) -> [u8; MESSAGE_LENGTH] {
        if self.desc.variant == EntryVariant::Field(ValueType::Secret) {
            let max_len = MESSAGE_LENGTH.min(self.data[0] as usize);
            for i in 1..max_len {
                self.data[i] = '*' as u8;
            }
        }
        self.data[0] = 1;
        self.data
    }

    pub fn with_integer(mut self, value: i64) -> Self {
        self.set_integer(value);
        self
    }

    pub fn get_integer(&self) -> i64 {
        let value_bytes = self.data[1..=8].try_into().unwrap();
        i64::from_le_bytes(value_bytes)
    }

    pub fn set_integer(&mut self, value: i64) {
        let value_bytes = if let Constraints::Range(range) = &self.desc.constraints {
            value.clamp(range.start as i64, range.end as i64) 
        } else { 
            value 
        }.to_le_bytes();
        for i in 0..8 {
            self.data[i + 1] = value_bytes[i];
        }
        self.set_len(8);
    }

    pub fn get_options(&self) -> Options {
        let len = self.data[0];
        let mut buf = [0u16; 31];
        for i in 0..len as usize{
            let offset = 2 * i as usize;
            buf[i] = u16::from_le_bytes([self.data[offset+1], self.data[offset+2]]);
        }
        Options {
            len,
            buf
        }
    }

    pub fn set_options(&mut self, value: &[u16]) {
        let Constraints::Values(constraints) = &self.desc.constraints else {
            panic!("option has no values")
        };

        let max_val = constraints.value_provider.len() as u16 - 1;
        let mut size: usize = 0;

        for i in 0..value.len() {
            if size >= constraints.max_or_suggested as usize { 
                warn!("skipping the last {} values since the field only supports a max of {}", 
                    value.len() as u16 - constraints.max_or_suggested, constraints.max_or_suggested);
                break 
            }
            let opt_value = value[i];
            if opt_value <= max_val {
                let opt_bytes = opt_value.to_le_bytes();
                let offset = size * 2;
                self.data[offset + 1] = opt_bytes[0];
                self.data[offset + 2] = opt_bytes[1];
                size += 1;
            } else {
                warn!("skipped option value {} as it's not a valid option index", opt_value)
            }
        }
        self.set_len(size);
    }

    pub fn get_text(&self) -> &str {
        use core::str;

        let end = self.data.iter().position(|&b| b == 0).unwrap_or(self.data.len());
        str::from_utf8(&self.data[1..end]).unwrap_or_default()
    }

    pub fn set_status(&mut self, code: u8, text: &str) {
        let max_len = text.floor_char_boundary(62);
        let value = if text.len() != max_len {
            &text[0..max_len]
        } else {text};
        let value_bytes = value.as_bytes();
        self.data[1] = code;
        for i in 0..value_bytes.len() {
            self.data[i + 2] = value_bytes[i];
        }
        self.set_len(value_bytes.len() + 1);
    }

    pub fn set_bytes(&mut self, bytes: &[u8]) {
        let clamped_len: usize = bytes.len().min(63);
        for i in 0..clamped_len {
            self.data[i + 1] = bytes[i];
        }
        self.set_len(clamped_len);
    }

    pub fn set_text(&mut self, value: &str) {

        let max_len = match &self.desc.constraints {
            Constraints::Range(range) => range.end as usize,
            _ => value.len(),
        };
        let max_len = value.floor_char_boundary(max_len);
        let value = if value.len() != max_len {
            &value[0..max_len]
        } else {value};
        let value_bytes = value.as_bytes();
        for i in 0..value_bytes.len() {
            self.data[i + 1] = value_bytes[i];
        }
        self.set_len(value_bytes.len());

        // TODO: handle values constraints: not suggested and min/max
        // if let Constraints::Values(value_constr) = &self.desc.constraints {
             
        // }
    }

    pub fn clamp(&mut self) {
        match self.desc.variant {
            EntryVariant::Field(field_type) => match field_type {
                ValueType::Integer => {
                    self.set_integer(self.get_integer());
                },
                ValueType::Text => {
                    #[cfg(feature = "alloc")]
                    self.set_text(&self.get_text().to_owned());
                },
                ValueType::Secret => {
                    #[cfg(feature = "alloc")]
                    self.set_text(&self.get_text().to_owned());
                },
                ValueType::Status => {},
                ValueType::Bytes => {},
                ValueType::Options => {
                    self.set_options(self.get_options().as_slice());
                }
            },
            _ => {
                warn!("tried to clamp entity variant {:?}", self.desc.variant)
            }
        }
    }
    
    fn is_empty(&self) -> bool {
        if self.data[0] == 0 { return true }
        self.data.iter().all(|b| *b == 0xff)
    }
}

impl From<FieldValue> for CommandResponse {
    fn from(value: FieldValue) -> Self {
        Self::from_field_value(value)
    }
}

#[cfg(test)]
mod test {

    use crate::entry::{EntryDesc, integer};
    use crate::prelude::*;

    const DESC_STRVAL1: EntryDesc = prop("strval").as_entry();
    const DESC_INTVAL1: EntryDesc = integer("strval").writable().as_entry();
    const OPT1_PROVIDER: [&'static str; 3] = ["item 1", "item 2", "item 3"];
    const OPT1_DEFAULT: [u16; 0] = [];
    const DESC_OPTVAL1: EntryDesc = option("strval", &OPT1_PROVIDER)
        .with_default_options(&OPT1_DEFAULT)
        .with_max_options(3)
        .as_entry();

    #[test]
    fn field_value_str_roundtrip() {
        let mut fv = FieldValue::new(&DESC_STRVAL1);
        fv.set_text("bytes and magic");
        assert_eq!(15, fv.len());
        let bytes = fv.into_store_bytes();
        let fv = FieldValue::from_store(&DESC_STRVAL1, bytes);
        assert_eq!(15, fv.len());
        assert_eq!("bytes and magic", fv.get_text());
    }

    #[test]
    fn field_value_int_roundtrip() {
        let mut fv = FieldValue::new(&DESC_INTVAL1);
        fv.set_integer(-247);
        assert_eq!(8, fv.len());
        let bytes = fv.into_store_bytes();
        let fv = FieldValue::from_store(&DESC_INTVAL1, bytes);
        assert_eq!(8, fv.len());
        assert_eq!(-247, fv.get_integer());
    }

        #[test]
    fn field_value_option_roundtrip() {
        let mut fv = FieldValue::new(&DESC_OPTVAL1);
        fv.set_options(&[1, 2, 0]);
        assert_eq!(3, fv.len());
        let bytes = fv.into_store_bytes();
        let fv = FieldValue::from_store(&DESC_OPTVAL1, bytes);
        assert_eq!(3, fv.len());
        assert_eq!(&[1, 2, 0], fv.get_options().as_slice());
    }
}