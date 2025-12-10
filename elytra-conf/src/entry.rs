use core::prelude::rust_2024::{*};
use core::{ops::Range};
use bitflags::bitflags;
use elytra_bytepack::{Buf, pack};
use crate::{
    values::DefaultValue,
    command::CommandResponse, 
    entry::options::{OptionValueProvider}, 
    config::MESSAGE_LENGTH
};

pub mod options;
mod sections;
mod fields;
mod actions;

pub use self::sections::*;
pub use self::fields::*;
pub use self::actions::*;

use super::{
    values::ValueType,
};

bitflags! {
    #[derive(Debug, Eq, PartialEq, Clone, Copy)]
    pub struct ExtraFlags: u8 {
        const ReadOnly = 1 << 0;
        const HasHelp = 1 << 1;
        const HasIcon = 1 << 2;
        const HasOptions = 1 << 3;
        const IsMulti = 1 << 4;
    }
}

#[cfg(feature = "defmt")]
impl Format for ExtraFlags {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "[");
        for (i, (name, _)) in self.iter_names().enumerate() {
            if i != 0 {
                defmt::write!(fmt, ", ");
            }
            defmt::write!(fmt, "{}", name)
        }
        defmt::write!(fmt, "]");
    }
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EntryVariant {
    Action(ActionVariant),
    Field(ValueType),
    Section
}

impl EntryVariant {
    pub fn bits(&self) -> u8 {
        match self {
            EntryVariant::Action(action_variant) => *action_variant as u8,
            EntryVariant::Field(value_type) => *value_type as u8,
            EntryVariant::Section => 0u8,
        }
    }
}

#[derive(Debug)]
pub struct ValueConstraints {
    pub(crate) value_provider: &'static dyn OptionValueProvider,
    pub(crate) min: u16,
    /// For **indexd values** (options), this is the maxumim amount of values the user can select.
    /// 
    /// For **content values** (text, integer etc.), this instead indicates whether the listed values
    /// are the only ones that are accepted (`0`), or if they are merly suggestions (`1`).
    pub(crate) max_or_suggested: u16,
}

impl ValueConstraints {
    pub fn is_suggested(&self) -> bool {
        self.max_or_suggested == 1
    }
}

#[derive(Debug)]
pub enum Constraints {
    None,
    Range(Range<i32>),
    Length(u64),
    Values(ValueConstraints)
}

#[cfg(feature = "defmt")]
impl defmt::Format for Constraints {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Constraints::None => defmt::write!(fmt, "NoneConstraints"),
            Constraints::Range(range) => defmt::write!(fmt, "RangeConstraint({}, {})", range.start, range.end),
            Constraints::Length(len) => defmt::write!(fmt, "LengthConstraints({})", len),
            Constraints::Values(ovp) => defmt::write!(fmt, "ValuesConstraints({}, {}, {}, {})", ovp.value_provider.len(), ovp.suggested, ovp.min, ovp.max),
        }
    }
}

impl Constraints {

    pub fn bits(&self) -> [u8; 8] {
        match self {
            Constraints::None => [0; 8],
            Constraints::Range(Range { start, end}) => {
                pack!(
                    start.to_le_bytes(),
                    end.to_le_bytes()
                )
            },
            Constraints::Length(len) => len.to_le_bytes(),
            Constraints::Values(constr) => {
                pack!(
                    u32::to_le_bytes(constr.value_provider.len() as u32),
                    constr.min.to_le_bytes(),
                    constr.max_or_suggested.to_le_bytes()
                )
            },
        }
    }

    pub fn is_values(&self) -> bool {
        match self {
            Self::Values(_) => {true},
            _ => {false},
        }
    }

    pub fn is_length(&self) -> bool {
        match self {
            Self::Length(_) => {true},
            _ => {false},
        }
    }

    pub fn is_range(&self) -> bool {
        match self {
            Self::Range(_) => {true},
            _ => {false},
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EntryDesc {
    pub variant: EntryVariant,
    pub readonly: bool,
    pub name: &'static str,
    pub constraints: Constraints,
    pub help: Option<&'static str>,
    pub icon: Option<&'static str>,
    pub default: DefaultValue,
    pub multi: bool,
}

impl EntryDesc {
    pub const fn new(
        name: &'static str,
        variant: EntryVariant,
        readonly: bool,
        constraints: Constraints,
        help: Option<&'static str>,
        icon: Option<&'static str>,
        default: DefaultValue,
        multi: bool,
    ) -> Self {
        if name.len() == 0 { panic!("name is required"); }
        if name.as_bytes().len() > Self::MAX_ENTRY_NAME_LEN { panic!("name is too long") }
        if let EntryVariant::Field(value_type) = variant {
            match (value_type, &default) {
                (ValueType::Bytes, DefaultValue::Bytes(_)) => {},
                (ValueType::Integer, DefaultValue::Integer(_)) => {},
                (ValueType::Text, DefaultValue::Text(_)) => {},
                (ValueType::Secret, DefaultValue::Text(_)) => {},
                (ValueType::Options, DefaultValue::Options(_)) => {},
                (_, DefaultValue::Empty) => {},
                (ValueType::Status, _) => panic!("Status value type cannot have a default value"),
                (_, DefaultValue::Integer(_)) => panic!("Integer is not a valid default value for this field"),
                (_, DefaultValue::Bytes(_)) => panic!("Bytes is not a valid default value for this field"),
                (_, DefaultValue::Text(_)) => panic!("Text is not a valid default value for this field"),
                (_, DefaultValue::Options(_)) => panic!("Options is not a valid default value for this field"),
            }
        }
        match (&constraints, &default) {
            (Constraints::Values(_), DefaultValue::Empty) => panic!("Option value type needs a default value"),
            _ => {},
        }

        Self {
            name,
            variant,
            readonly,
            constraints,
            help,
            icon,
            default,
            multi,
        }
    }

    pub fn flags(&self) -> ExtraFlags {
        let mut flags = ExtraFlags::empty();
        flags.set(ExtraFlags::ReadOnly, self.readonly);
        flags.set(ExtraFlags::HasHelp, self.help.is_some());
        flags.set(ExtraFlags::HasIcon, self.icon.is_some());
        flags.set(ExtraFlags::HasOptions, self.constraints.is_values());
        flags.set(ExtraFlags::IsMulti, self.multi);
        flags
    }

    const MAX_ENTRY_NAME_LEN: usize = MESSAGE_LENGTH - (
        1 // flags
        +
        1 // variant
        +
        8 // constraints
    );
}

impl From<&EntryDesc> for CommandResponse {
    fn from(value: &EntryDesc) -> Self {
        let mut res = CommandResponse::new();
        res.push(value.flags().bits()); // readonly (1 byte)
        res.push(value.variant.bits());  // type (1 byte)
        res.extend(value.constraints.bits()); // 8 byte
        // use the rest of the message buffer for field name
        res.extend(value.name.bytes());
        res
    }
}