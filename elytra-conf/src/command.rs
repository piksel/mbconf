use log::{debug, info};
use num_enum::TryFromPrimitive;
use core::{panic, slice};
use core::prelude::rust_2024::{*};

use crate::config::{EntryIndex, EntryType, QueryTarget, QueryTargetKey};
use crate::{ActionIndex, InfoIndex, PropIndex, SectionIndex};
use crate::{
    config::{MESSAGE_LENGTH, PAYLOAD_SIZE},
    field::FieldValue
};

pub struct CommandResponse {
    bytes: [u8; MESSAGE_LENGTH],
    len: usize,
}

impl CommandResponse {
    pub fn ok() -> Self { Self::new() }

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

impl From<Result<CommandResponse, CommandError>> for CommandResponse {
    fn from(result: Result<CommandResponse, CommandError>) -> Self {
        result.unwrap_or_else(CommandResponse::error)
    }
}

#[repr(u8)]
#[derive(TryFromPrimitive)]
pub enum CommandKey {
    ReadProp = 'r' as u8,
    WriteProp = 'w' as u8,
    ReadInfo = 'R' as u8,
    WriteInfo = 'W' as u8,
    Query = 'q' as u8,
    Action = 'a' as u8,
    Meta = 'm' as u8,
    Noop = 0,
}

// pub enum QueryArgs {
//     entry_type: EntryType
// }

pub enum Command<A: ActionIndex, P: PropIndex, I: InfoIndex, S: SectionIndex> {
    ReadProp(P),
    WriteProp((P, FieldValue)),
    ReadInfo(I),
    WriteInfo((I, FieldValue)),
    Query((EntryIndex<A, P, I, S>, QueryTarget)),
    Action(A),
    Meta,
    Noop,
}

impl <A: ActionIndex, P: PropIndex, I: InfoIndex, S: SectionIndex> Command<A, P, I, S> {
    pub fn from_bytes<'a>(bytes: &[u8]) -> Result<Command<A, P, I, S>, CommandError> {
        let mut bytes = bytes.into_iter();
        let key = bytes.next()
            .and_then(|b| CommandKey::try_from(*b).ok())
            .ok_or(CommandError::InvalidCommand)?;

        match key {
            CommandKey::Action =>{
                Ok(Command::Action(Self::get_action_index(&mut bytes)?))
            },
            CommandKey::ReadProp => {
                Ok(Command::ReadProp(Self::get_prop_index(&mut bytes)?))
            },
            CommandKey::WriteProp => { 
                let prop_field = Self::get_prop_index(&mut bytes)?;
                let payload = Self::get_payload(&mut bytes)?;
                let desc = P::get_entry(prop_field);
                let field_value = FieldValue::from_message(desc, payload);
                Ok(Command::WriteProp((prop_field, field_value)))
            },
            CommandKey::ReadInfo => {
                Ok(Command::ReadInfo(Self::get_info_index(&mut bytes)?))
            },
            CommandKey::WriteInfo => {
                let info_field = Self::get_info_index(&mut bytes)?;
                let payload = Self::get_payload(&mut bytes)?;
                let desc = I::get_entry(info_field);
                let field_value = FieldValue::from_message(desc, payload);
                Ok(Command::WriteInfo((info_field, field_value)))
            },
            CommandKey::Query => {
                let entry_type = Self::get_entry_type(&mut bytes)?;
                debug!("Entry type: {:?}", entry_type);
                let entry_index = Self::get_entry_index(&mut bytes, entry_type)?;
                debug!("Entry index: {:?}", entry_index);
                let query_prop = Self::get_query_prop(&mut bytes)?;
                debug!("Query prop: {:?}", query_prop);

                info!("Entry type: {:?}, index: {:?}, prop: {:?}", entry_type, entry_index, query_prop);
                let target = match query_prop {
                    QueryTargetKey::Field => Ok(QueryTarget::Field),
                    QueryTargetKey::Help => Ok(QueryTarget::Help),
                    QueryTargetKey::Icon => Ok(QueryTarget::Icon),
                    QueryTargetKey::Option => {
                        // let entry = entry_index.get_entry();
                        // let Constraints::Values(ValueConstraints{value_provider, ..}) = &entry.constraints else {
                        //     return Err(CommandError::NotSupported)
                        // };
                        // TODO: Replace with .next_chunk when stable
                        let Some(index_bytes) = bytes.next().map(|b| bytes.next().map(|b2| [*b, *b2])).flatten() else {
                            return Err(CommandError::MissingArgument)
                        };
                        let option_index: u16 = u16::from_le_bytes(index_bytes);
                        Ok(QueryTarget::Option(option_index))
                    },
                    QueryTargetKey::Layout =>  match entry_index {
                        EntryIndex::Section(_si) => Ok(QueryTarget::Layout),
                        _ => Err(CommandError::InvalidQuery)
                    }
                }?;
                Ok(Command::Query((entry_index, target)))
            },
            CommandKey::Noop => Ok(Command::Noop),
            CommandKey::Meta => Ok(Command::Meta),
        }
    }

    fn get_prop_index(bytes: &mut slice::Iter<'_, u8>) -> Result<P, CommandError> {
        let index = *bytes.next().ok_or(CommandError::MissingArgument)?;
        P::from_byte(index).ok_or(CommandError::InvalidField)
    }

    fn get_info_index(bytes: &mut slice::Iter<'_, u8>) -> Result<I, CommandError> {
        let index = *bytes.next().ok_or(CommandError::MissingArgument)?;
        I::from_byte(index).ok_or(CommandError::InvalidField)
    }

    fn get_section_index(bytes: &mut slice::Iter<'_, u8>) -> Result<S, CommandError> {
        let index = *bytes.next().ok_or(CommandError::MissingArgument)?;
        S::from_byte(index).ok_or(CommandError::InvalidSection)
    }

    fn get_action_index(bytes: &mut slice::Iter<'_, u8>) -> Result<A, CommandError> {
        let index = *bytes.next().ok_or(CommandError::MissingArgument)?;
        A::from_byte(index).ok_or(CommandError::InvalidAction)
    }

    fn get_payload<'a>(bytes: &mut slice::Iter<'a, u8>) -> Result<&'a [u8], CommandError> {
        let trail = bytes.as_slice();
        if trail.len() < 1 {
            Err(CommandError::InvalidData)
        } else {
            Ok(trail)
        }
    }

    fn get_entry_type(bytes: &mut slice::Iter<'_, u8>) -> Result<EntryType, CommandError> {
        let byte = *bytes.next().ok_or(CommandError::MissingArgument)?;
        EntryType::try_from_primitive(byte).or(Err(CommandError::InvalidEntry))
    }

    fn get_entry_index(bytes: &mut slice::Iter<'_, u8>, entry_type: EntryType) -> Result<EntryIndex<A, P, I, S>, CommandError> where 
        A: ActionIndex, P: PropIndex, I: InfoIndex, S: SectionIndex
    {
        use EntryType::{*};
        match entry_type {
            Action => Ok(EntryIndex::Action(Self::get_action_index(bytes)?)),
            Prop => Ok(EntryIndex::Prop(Self::get_prop_index(bytes)?)),
            Info   => Ok(EntryIndex::Info(Self::get_info_index(bytes)?)),
            Section => Ok(EntryIndex::Section(Self::get_section_index(bytes)?)),
        }
    }
    fn get_query_prop(bytes: &mut slice::Iter<'_, u8>) -> Result<QueryTargetKey, CommandError> {
        let byte = *bytes.next().ok_or(CommandError::MissingArgument)?;
        QueryTargetKey::try_from_primitive(byte).or(Err(CommandError::InvalidQuery))
    }
}




#[repr(u8)]
#[derive(Debug, strum::Display, Clone, Copy)]
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