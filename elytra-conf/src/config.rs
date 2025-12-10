use core::{prelude::rust_2024::*};
use log::{debug, info};
use num_enum::TryFromPrimitive;

use crate::{
   field::FieldValue, 
   command::{Command, CommandError, CommandHandler, CommandResponse}, 
   entry::{Constraints, EntryDesc, Field, ValueConstraints}, 
   traits::{ActionIndex, PropIndex, InfoIndex, SectionIndex}
};
use core::{marker::PhantomData, slice};

pub const MESSAGE_LENGTH: usize = 64;
pub const PAYLOAD_SIZE: usize = MESSAGE_LENGTH - 1;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, TryFromPrimitive, strum::EnumString)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EntryType {
    Action = b'a',
    Prop = b'c',
    Info = b'i',
    Section = b's',
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, TryFromPrimitive, strum::EnumString)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum QueryTarget {
    Basic = b'b',
    Icon = b'i',
    Help = b'h',
    Layout = b'l',
    Option = b'o'
}

// impl FromStr for QueryProp {
//     type Err = &'static str;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "basic" | "b" => Ok(Self::Basic),
//             "icon" | "i" => Ok(Self::Icon),
//             "help" | "h" => Ok(Self::Help),
//             "layout" | "l" => Ok(Self::Layout),
//             "option" | "o" => Ok(Self::Option),
//             _ => Err("invalid query prop")
//         }
//     }
// }

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum EntryIndex<AI, PI, II, SI> {
    Action(AI),
    Prop(PI),
    Info(II),
    Section(SI)
}

pub struct Config<
    const S: usize, 
    const C: usize, 
    const I: usize, 
    const A: usize,
    const L: usize,
    SI: SectionIndex, 
    PI: PropIndex, 
    II: InfoIndex, 
    AI: ActionIndex
> {
    pub sections: [EntryDesc; S],
    pub prop_fields: [EntryDesc; C],
    pub info_fields: [EntryDesc; I],
    pub actions: [EntryDesc; A],
    pub layout: [(SI, Field<PI, II>); L],
    _field_index: PhantomData<PI>,
    _status_index: PhantomData<II>,
    _action_index: PhantomData<AI>
}

fn get_prop_field<'a, PI: PropIndex>(bytes: &mut slice::Iter<'a, u8>) -> Result<PI, CommandError> {
    let index = *bytes.next().ok_or(CommandError::MissingArgument)?;
    PI::from_byte(index).ok_or(CommandError::InvalidField)
}

fn get_info_field<'a, II: InfoIndex>(bytes: &mut slice::Iter<'a, u8>) -> Result<II, CommandError> {
    let index = *bytes.next().ok_or(CommandError::MissingArgument)?;
    II::from_byte(index).ok_or(CommandError::InvalidField)
}

fn get_section<'a, SI: SectionIndex>(bytes: &mut slice::Iter<'a, u8>) -> Result<SI, CommandError> {
    let index = *bytes.next().ok_or(CommandError::MissingArgument)?;
    SI::from_byte(index).ok_or(CommandError::InvalidSection)
}

fn get_action<'a, AI: ActionIndex>(bytes: &mut slice::Iter<'a, u8>) -> Result<AI, CommandError> {
    let index = *bytes.next().ok_or(CommandError::MissingArgument)?;
    AI::from_byte(index).ok_or(CommandError::InvalidAction)
}

fn get_payload<'a>(bytes: &mut slice::Iter<'a, u8>) -> Result<&'a [u8], CommandError> {
    let trail = bytes.as_slice();
    if trail.len() < 1 {
        Err(CommandError::InvalidData)
    } else {
        Ok(trail)
    }
}

fn get_entry_type<'a>(bytes: &mut slice::Iter<'a, u8>) -> Result<EntryType, CommandError> {
    let byte = *bytes.next().ok_or(CommandError::MissingArgument)?;
    EntryType::try_from_primitive(byte).or(Err(CommandError::InvalidEntry))
}
fn get_entry_index<'a, AI, PI, II, SI>(bytes: &mut slice::Iter<'a, u8>, entry_type: EntryType) -> Result<EntryIndex<AI, PI, II, SI>, CommandError> where 
    AI: ActionIndex, PI: PropIndex, II: InfoIndex, SI: SectionIndex
{
    let index = *bytes.next().ok_or(CommandError::MissingArgument)?;
    match entry_type {
        EntryType::Action => Ok(EntryIndex::Action(AI::from_byte(index).ok_or(CommandError::InvalidAction)?)),
        EntryType::Prop => Ok(EntryIndex::Prop(PI::from_byte(index).ok_or(CommandError::InvalidAction)?)),
        EntryType::Info   => Ok(EntryIndex::Info(II::from_byte(index).ok_or(CommandError::InvalidAction)?)),
        EntryType::Section => Ok(EntryIndex::Section(SI::from_byte(index).ok_or(CommandError::InvalidAction)?)),
    }
}
fn get_query_prop<'a>(bytes: &mut slice::Iter<'a, u8>) -> Result<QueryTarget, CommandError> {
    let byte = *bytes.next().ok_or(CommandError::MissingArgument)?;
    QueryTarget::try_from_primitive(byte).or(Err(CommandError::InvalidQuery))
}


impl <'s: 'static, const S: usize, const P: usize, const I: usize, const A: usize, const L: usize,
    SI: SectionIndex, PI: PropIndex, II: InfoIndex, AI: ActionIndex>  Config<S, P, I, A, L, SI, PI, II, AI> {

    const PROTO_VERSION: u8 = 1;

    pub const fn new(
            sections: [EntryDesc; S], 
            prop_fields: [EntryDesc; P], 
            info_fields: [EntryDesc; I], 
            actions: [EntryDesc; A],
            layout: [(SI, Field<PI, II>); L]) -> Self {
        Self {
            sections,
            prop_fields: prop_fields,
            info_fields,
            actions,
            layout,
            _field_index: PhantomData,
            _status_index: PhantomData,
            _action_index: PhantomData
        }
    }

    fn info_response(&'s self) -> CommandResponse {
        let mut res = CommandResponse::new();
        // Protocol version (1 byte)
        res.push(Self::PROTO_VERSION);

        // Field section count (1 byte)
        res.push(self.sections.len() as u8);

        // Prop field count (1 byte)
        res.push(self.prop_fields.len() as u8);

        // Info field count (1 byte)
        res.push(self.info_fields.len() as u8);

        // Action count (1 byte)
        res.push(self.actions.len() as u8);

        res
    }

    pub async fn parse_command2<'a, CH: CommandHandler<PI, II, AI>>(&'s self, mut bytes: slice::Iter<'a, u8>, handler: &mut CH) -> Result<CommandResponse, CommandError> {
        
        let command = bytes.next()
            .and_then(|b| Command::try_from(*b).ok())
            .ok_or(CommandError::InvalidCommand)?;

        match command {
            Command::Action =>{
                let ai = get_action(&mut bytes)?;
                handler.do_action(ai).await?;
                Ok(CommandResponse::OK)
            },
            Command::ReadProp => {
                let prop_field = get_prop_field(&mut bytes)?;
                handler.read_prop(prop_field).await.map(CommandResponse::from_field_value)
            },
            Command::DescProp => {
                let prop_field = get_prop_field(&mut bytes)?;
                Ok(self.prop_field(prop_field).into())
            },
            Command::WriteProp => { 
                let prop_field = get_prop_field(&mut bytes)?;
                let payload = get_payload(&mut bytes)?;
                let desc = self.prop_field(prop_field);
                let field_value = FieldValue::from_message(desc, payload);
                handler.write_prop(prop_field, field_value).await.and(Ok(CommandResponse::OK))
            },
            Command::ReadInfo => {
                let info_field = get_info_field(&mut bytes)?;
                handler.read_info(info_field).await.map(CommandResponse::from_field_value)
            },
            Command::DescInfo => {
                let info_field = get_info_field(&mut bytes)?;
                Ok(self.info_field(info_field).into())
            },
            Command::WriteInfo => {
                let info_field = get_info_field(&mut bytes)?;
                let payload = get_payload(&mut bytes)?;
                let desc = self.info_field(info_field);
                let field_value = FieldValue::from_message(desc, payload);
                handler.write_info(info_field, field_value).await.and(Ok(CommandResponse::OK))
            },
            Command::DescSection => {
                let section = get_section(&mut bytes)?;
                Ok(self.section_desc(section).into())
            },
            Command::DescAction => {
                let action = get_action(&mut bytes)?;
                Ok(self.action_name(action).into())
            },
            Command::Query => {
                let entry_type = get_entry_type(&mut bytes)?;
                debug!("Entry type: {:?}", entry_type);
                let entry_index = get_entry_index(&mut bytes, entry_type)?;
                debug!("Entry index: {:?}", entry_index);
                let query_prop = get_query_prop(&mut bytes)?;
                debug!("Query prop: {:?}", query_prop);

                info!("Entry type: {:?}, index: {:?}, prop: {:?}", entry_type, entry_index, query_prop);

                let entry = match entry_index {
                    EntryIndex::Action(ai) => self.action_name(ai),
                    EntryIndex::Prop(ci) => self.prop_field(ci),
                    EntryIndex::Info(ii) => self.info_field(ii),
                    EntryIndex::Section(si) => self.section_desc(si)
                };
                
                match query_prop {
                    QueryTarget::Basic => Ok(entry.into()),
                    QueryTarget::Help => entry.help.ok_or(CommandError::NoContent).map(Into::into),
                    QueryTarget::Icon => entry.icon.ok_or(CommandError::NoContent).map(Into::into),
                    QueryTarget::Option => {
                        let Constraints::Values(ValueConstraints{value_provider, ..}) = &entry.constraints else {
                            return Err(CommandError::NotSupported)
                        };
                        // TODO: Replace with .next_chunk when stable
                        let Some(index_bytes) = bytes.next().map(|b| bytes.next().map(|b2| [*b, *b2])).flatten() else {
                            return Err(CommandError::MissingArgument)
                        };
                        let option_index: u16 = u16::from_le_bytes(index_bytes);
                        value_provider.get(option_index as usize).ok_or(CommandError::InvalidOption).map(|s| (*s).into())
                    },
                    QueryTarget::Layout => match entry_index {
                        EntryIndex::Section(si) => Ok(self.section_layout(si)),
                        _ => Err(CommandError::InvalidQuery)
                    }
                }
            },
            Command::Noop => {
                handler.noop().await;
                Ok(CommandResponse::OK)
            },
            Command::Info => {
                Ok(self.info_response())
            },
        }
    }

    pub async fn parse_command<'a, CH: CommandHandler<PI, II, AI>>(&'s self, bytes: &'a [u8], handler: &mut CH) -> CommandResponse {
        let bytes = bytes.into_iter();
        self.parse_command2(bytes, handler).await.unwrap_or_else(CommandResponse::error)
    }

    pub fn prop_field(&'s self, index: PI) -> &'s EntryDesc {
        &self.prop_fields[index.as_index()]
    }

    pub fn info_field(&'s self, index: II) -> &'s EntryDesc {
        &self.info_fields[index.as_index()]
    }

    pub fn section_desc(&'s self, section: SI) -> &'s EntryDesc {
        &self.sections[section.as_index()]
    }

    pub fn action_name(&'s self, action: AI) -> &'s EntryDesc {
        &self.actions[action.as_index()]
    }

    pub fn section_layout(&'s self, section: SI) -> CommandResponse {
        let mut res = CommandResponse::new();
        self.layout.iter()
            .filter(|(si, _)| *si == section)
            .take(63)
            .for_each(|(_, field)| res.extend(field.bits()));
        res
    }
}
