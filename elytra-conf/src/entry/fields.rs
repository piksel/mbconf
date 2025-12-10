use core::ops::Range;

use crate::{
    traits::{PropIndex, InfoIndex},
    entry::{Constraints, EntryDesc, EntryVariant, ValueConstraints}, 
    prelude::OptionValueProvider, 
    config::EntryType, 
    values::{DefaultValue, ValueType}
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Field<PI: PropIndex, II: InfoIndex> {
    Prop(PI),
    Info(II)
}

impl <PI: PropIndex, II: InfoIndex> Field<PI, II> {
    pub fn bits(&self) -> [u8; 2] {
        match self {
            Self::Prop(pi) => [EntryType::Prop as u8, pi.as_index() as u8],
            Self::Info(ii) => [EntryType::Info as u8, ii.as_index() as u8],
        }
    }
}


#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FieldEntry {
    pub value_type: ValueType,
    pub readonly: Option<bool>,
    pub name: &'static str,
    pub constraints: Constraints,
    pub help: Option<&'static str>,
    pub icon: Option<&'static str>,
    pub default: DefaultValue,
    pub multi: bool,
}

#[allow(unused)]
impl FieldEntry {
    pub const fn as_entry(self) -> EntryDesc {
        let Some(readonly) = self.readonly else {
            panic!("proto field writable configuration is ambigous: use .writable() or .readonly()") 
        };
        EntryDesc::new(
            self.name, 
            EntryVariant::Field(self.value_type), 
            readonly,
            self.constraints, 
            self.help, 
            self.icon,
            self.default,
            self.multi,
        )
    }
    pub const fn with_icon(self, icon: &'static str) -> Self {
        Self {
            icon: Some(icon),
            ..self
        }
    }
    pub const fn with_help(self, help: &'static str) -> Self {
        Self {
            help: Some(help),
            ..self
        }
    }
    pub const fn with_range(self, range: Range<i32>) -> Self {
        Self {
            constraints: Constraints::Range(range),
            ..self
        }
    }
    pub const fn with_type(self, value_type: ValueType) -> Self {
        Self {
            value_type,
            ..self
        }
    }
    pub const fn writable(self) -> Self {
        Self {
            readonly: Some(false),
            ..self
        }
    }
    pub const fn readonly(self) -> Self {
        Self {
            readonly: Some(true),
            ..self
        }
    }

    pub const fn with_values(self, value_provider: &'static dyn OptionValueProvider, suggested: bool, min: u16) -> Self {
        Self {
            constraints: Constraints::Values(ValueConstraints { 
                value_provider, 
                min,
                max_or_suggested: if suggested {1} else {0}
             }),
            ..self
        }
    }

    pub const fn with_suggestions(self, value_provider: &'static dyn OptionValueProvider) -> Self {
        self.with_values(value_provider, true, 1)
    }

    pub const fn with_options(self, value_provider: &'static dyn OptionValueProvider) -> Self {
        self.with_values(value_provider, false, 1)
    }

    pub const fn with_default_text(self, value: &'static str) -> Self {
        Self {
            default: DefaultValue::Text(value),
            ..self
        }
    }

    pub const fn with_default_integer(self, value: i64) -> Self {
        Self {
            default: DefaultValue::Integer(value),
            ..self
        }
    }

    pub const fn with_default_bytes(self, value: &'static [u8]) -> Self {
        Self {
            default: DefaultValue::Bytes(value),
            ..self
        }
    }

    pub const fn with_default_options(self, value: &'static [u16]) -> Self {
        Self {
            default: DefaultValue::Options(value),
            ..self
        }
    }

    pub const fn with_max_options(self, max: u16) -> Self {
        let Constraints::Values(values) = self.constraints else {
            panic!("field does not have values constraint");
        };

        Self {
            constraints: Constraints::Values(
                ValueConstraints{
                    max_or_suggested: max,
                    ..values
            }
            ),
            ..self
        }
    }
}

#[allow(unused)]
pub const fn bytes(name: &'static str, size: u8) -> FieldEntry {
    FieldEntry {
        name,
        value_type: ValueType::Bytes,
        constraints: Constraints::Length(size as u64),
        readonly: Some(true),
        help: None,
        icon: None,
        default: DefaultValue::Empty,
        multi: false,
    }
}

#[allow(unused)]
pub const fn secret(name: &'static str) -> PropEntry {
    PropEntry {
        name,
        value_type: ValueType::Secret,
        constraints: Constraints::None,
        readonly: Some(false),
        help: None,
        icon: None,
        default: DefaultValue::Empty,
        multi: false,
    }
}

#[allow(unused)]
pub const fn status(name: &'static str) -> InfoEntry {
    InfoEntry {
        name,
        value_type: ValueType::Status,
        constraints: Constraints::None,
        readonly: Some(true),
        help: None,
        icon: None,
        default: DefaultValue::Empty,
        multi: false,
    }
}

#[allow(unused)]
pub const fn integer(name: &'static str) -> InfoEntry {
    InfoEntry {
        name,
        value_type: ValueType::Integer,
        constraints: Constraints::None,
        readonly: None,
        help: None,
        icon: None,
        default: DefaultValue::Empty,
        multi: false,
    }
}

#[allow(unused)]
pub const fn option(name: &'static str, value_provider: &'static dyn OptionValueProvider) -> PropEntry {
    PropEntry {
        name,
        value_type: ValueType::Options,
        constraints: Constraints::Values(ValueConstraints { 
            value_provider,
            min: 1, max_or_suggested: 1
        }),
        readonly: Some(false),
        help: None,
        icon: None,
        default: DefaultValue::Empty,
        multi: false,
    }
}

pub type InfoEntry = FieldEntry;
#[allow(unused)]
pub const fn info(name: &'static str) -> InfoEntry {
    InfoEntry {
        name,
        value_type: ValueType::Text,
        constraints: Constraints::None,
        readonly: Some(true),
        help: None,
        icon: None,
        default: DefaultValue::Empty,
        multi: false,
    }
}

pub type PropEntry = FieldEntry;
#[allow(unused)]
pub const fn prop(name: &'static str) -> PropEntry {
    FieldEntry {
        name,
        value_type: ValueType::Text,
        constraints: Constraints::None,
        readonly: Some(false),
        help: None,
        icon: None,
        default: DefaultValue::Empty,
        multi: false,
    }
}