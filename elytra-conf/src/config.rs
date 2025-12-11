use core::{prelude::rust_2024::*};
use num_enum::TryFromPrimitive;

use crate::{
   command::{CommandError, CommandResponse}, 
   entry::{Constraints, EntryDesc, Field}, 
   traits::{ActionIndex, PropIndex, InfoIndex, SectionIndex}
};
use core::marker::PhantomData;

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
pub enum QueryTargetKey {
    Field = b'f',
    Icon = b'i',
    Help = b'h',
    Layout = b'l',
    Option = b'o'
}

#[derive(Debug)]
pub enum QueryTarget {
    Field,
    Icon,
    Help,
    Layout,
    Option(u16),
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EntryIndex<AI: ActionIndex, PI: PropIndex, II: InfoIndex, SI: SectionIndex> {
    Action(AI),
    Prop(PI),
    Info(II),
    Section(SI)
}

impl <AI: ActionIndex, PI: PropIndex, II: InfoIndex, SI: SectionIndex> EntryIndex<AI, PI, II, SI> {
    pub fn get_entry(self) -> &'static EntryDesc {
        match self {
            EntryIndex::Action(ai) => ai.get_entry(),
            EntryIndex::Prop(pi) => pi.get_entry(),
            EntryIndex::Info(ii) => ii.get_entry(),
            EntryIndex::Section(si) => si.get_entry(),
        }
    }
}

pub struct Config<
    const L: usize,
    SI: SectionIndex, 
    PI: PropIndex, 
    II: InfoIndex, 
    AI: ActionIndex
> {
    pub layout: [(SI, Field<PI, II>); L],
    _field_index: PhantomData<PI>,
    _status_index: PhantomData<II>,
    _action_index: PhantomData<AI>
}



impl <'s: 'static, const L: usize, S: SectionIndex, P: PropIndex, I: InfoIndex, A: ActionIndex>  Config<L, S, P, I, A> {

    const PROTO_VERSION: u8 = 1;

    pub const fn new(
            layout: [(S, Field<P, I>); L]) -> Self {
        Self {
            layout,
            _field_index: PhantomData,
            _status_index: PhantomData,
            _action_index: PhantomData
        }
    }

    // async fn _parse_command<'a, CH: CommandHandler<PI, II, AI>>(&'s self, mut bytes: slice::Iter<'a, u8>, handler: &mut CH) -> Result<CommandResponse, CommandError> {
        
    //     let command = bytes.next()
    //         .and_then(|b| CommandKey::try_from(*b).ok())
    //         .ok_or(CommandError::InvalidCommand)?;

    //     match command {
    //         CommandKey::Action =>{
    //             let ai: AI = get_action(&mut bytes)?;
    //             handler.do_action(ai).await?;
    //             Ok(CommandResponse::OK)
    //         },
    //         CommandKey::ReadProp => {
    //             let prop_field = get_prop_field(&mut bytes)?;
    //             handler.read_prop(prop_field).await.map(CommandResponse::from_field_value)
    //         },
    //         CommandKey::WriteProp => { 
    //             let prop_field = get_prop_field(&mut bytes)?;
    //             let payload = get_payload(&mut bytes)?;
    //             let desc = self.prop_field(prop_field);
    //             let field_value = FieldValue::from_message(desc, payload);
    //             handler.write_prop(prop_field, field_value).await.and(Ok(CommandResponse::OK))
    //         },
    //         CommandKey::ReadInfo => {
    //             let info_field = get_info_field(&mut bytes)?;
    //             handler.read_info(info_field).await.map(CommandResponse::from_field_value)
    //         },
    //         CommandKey::WriteInfo => {
    //             let info_field = get_info_field(&mut bytes)?;
    //             let payload = get_payload(&mut bytes)?;
    //             let desc = self.info_field(info_field);
    //             let field_value = FieldValue::from_message(desc, payload);
    //             handler.write_info(info_field, field_value).await.and(Ok(CommandResponse::OK))
    //         },
    //         CommandKey::Query => {
    //             let entry_type = Self::get_entry_type(&mut bytes)?;
    //             debug!("Entry type: {:?}", entry_type);
    //             let entry_index = Self::get_entry_index(&mut bytes, entry_type)?;
    //             debug!("Entry index: {:?}", entry_index);
    //             let query_prop = Self::get_query_prop(&mut bytes)?;
    //             debug!("Query prop: {:?}", query_prop);

    //             info!("Entry type: {:?}, index: {:?}, prop: {:?}", entry_type, entry_index, query_prop);

    //             let entry = match entry_index {
    //                 EntryIndex::Action(ai) => self.action_name(ai),
    //                 EntryIndex::Prop(ci) => self.prop_field(ci),
    //                 EntryIndex::Info(ii) => self.info_field(ii),
    //                 EntryIndex::Section(si) => self.section_desc(si)
    //             };
                
    //             match query_prop {
    //                 QueryTargetKey::Field => Ok(entry.into()),
    //                 QueryTargetKey::Help => entry.help.ok_or(CommandError::NoContent).map(Into::into),
    //                 QueryTargetKey::Icon => entry.icon.ok_or(CommandError::NoContent).map(Into::into),
    //                 QueryTargetKey::Option => {
    //                     let Constraints::Values(ValueConstraints{value_provider, ..}) = &entry.constraints else {
    //                         return Err(CommandError::NotSupported)
    //                     };
    //                     // TODO: Replace with .next_chunk when stable
    //                     let Some(index_bytes) = bytes.next().map(|b| bytes.next().map(|b2| [*b, *b2])).flatten() else {
    //                         return Err(CommandError::MissingArgument)
    //                     };
    //                     let option_index: u16 = u16::from_le_bytes(index_bytes);
    //                     value_provider.get(option_index as usize).ok_or(CommandError::InvalidOption).map(|s| (*s).into())
    //                 },
    //                 QueryTargetKey::Layout => match entry_index {
    //                     EntryIndex::Section(si) => Ok(self.section_layout(si)),
    //                     _ => Err(CommandError::InvalidQuery)
    //                 }
    //             }
    //         },
    //         CommandKey::Noop => {
    //             handler.noop().await;
    //             Ok(CommandResponse::OK)
    //         },
    //         CommandKey::Meta => {
    //             Ok(self.info_response())
    //         },
    //     }
    // }

    // pub async fn parse_command<'a, CH: CommandHandler<PI, II, AI>>(&'s self, bytes: &'a [u8], handler: &mut CH) -> CommandResponse {
    //     let bytes = bytes.into_iter();
    //     self._parse_command(bytes, handler).await.unwrap_or_else(CommandResponse::error)
    // }

    pub fn handle_meta(&'s self) -> CommandResponse {
        let mut res = CommandResponse::new();
        // Protocol version (1 byte)
        res.push(Self::PROTO_VERSION);

        // Field section count (1 byte)
        res.push(S::count() as u8);

        // Prop field count (1 byte)
        res.push(P::count() as u8);

        // Info field count (1 byte)
        res.push(I::count() as u8);

        // Action count (1 byte)
        res.push(A::count() as u8);

        res
    }

    pub fn handle_query(&'s self, entry_index: EntryIndex<A, P, I, S>, target: QueryTarget) -> Result<CommandResponse, CommandError> {
        let entry = entry_index.get_entry();
        use QueryTarget::{*};
        match target {
            Field => Ok(entry.into()),
            Help => entry.help.ok_or(CommandError::NoContent).map(Into::into),
            Icon => entry.icon.ok_or(CommandError::NoContent).map(Into::into),
            Option(option_index) => {
                let Constraints::Values(constr) = &entry.constraints else {
                    return Err(CommandError::NotSupported)
                };
                constr.value_provider.get(option_index as usize)
                    .ok_or(CommandError::InvalidOption).map(|s| (*s).into())
            },
            Layout => match entry_index {
                        EntryIndex::Section(si) => Ok( self.section_layout(si)),
                        _ => Err(CommandError::InvalidQuery)
            }
        }
    }

    pub fn prop_field(&'s self, index: P) -> &'s EntryDesc {
        index.get_entry()
    }

    pub fn info_field(&'s self, index: I) -> &'s EntryDesc {
        index.get_entry()
    }

    pub fn section_desc(&'s self, index: S) -> &'s EntryDesc {
        index.get_entry()
    }

    pub fn action_name(&'s self, index: A) -> &'s EntryDesc {
        index.get_entry()
    }

    pub fn section_layout(&'s self, section: S) -> CommandResponse {
        let mut res = CommandResponse::new();
        self.layout.iter()
            .filter(|(si, _)| *si == section)
            .take(31)
            .for_each(|(_, field)| res.extend(field.bits()));
        res
    }
}
