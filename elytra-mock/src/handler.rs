use elytra_conf::{command::{CommandError, CommandResponse}, field::FieldValue, traits::*};
use log::debug;
use crate::{Action, InfoField, MOCK_CONF, PropField, Section};
type Command = elytra_conf::command::Command<Action, PropField, InfoField, Section>;

pub fn handle_command(command: Command) -> CommandResponse {
    match command {
        Command::ReadProp(p) => {
            debug!("CMD: ReadProp: {:?}", p);
            let bytes_from_store = [0u8; 64];
            FieldValue::from_store(p.get_entry(), bytes_from_store).into()
        },
        Command::WriteProp((prop_field, field_value)) => {
            debug!("CMD: WriteProp: {:?}", prop_field);
            debug!(" => {:x?}", field_value);
            CommandResponse::ok()
        },
        Command::ReadInfo(i) => {
            debug!("CMD: ReadInfo: {:?}", i);
            let mut fv = FieldValue::new(i.get_entry());
            use InfoField::*;
            match i {
                WifiStatus => fv.set_status(3, "Performing dark rituals"),
                FlashUUID => fv.set_bytes(&[0, 1, 2, 3, 4, 5, 6, 7]),
                FlashJEDEC => fv.set_bytes(&[0x0a, 0xbc, 0xde, 0xf0]),
                PicoROM => fv.set_text("ROM Version: 0 (BADC0FFE)"),
                Time => fv.set_text("01:23"),
            };
            fv.into()
        },
        Command::WriteInfo((info_field, field_value)) => {
            debug!("CMD: WriteInfo: {:?}", info_field);
            debug!(" => {:x?}", field_value);
            CommandResponse::error(CommandError::NotSupported)
        },
        Command::Query((entry_index, target)) => {
            debug!("CMD: query: {:?} {:?}", entry_index, target);
            MOCK_CONF.handle_query(entry_index, target).into()
        },
        Command::Action(action) => {
            debug!("CMD: action: {:?}", action);
            CommandResponse::ok()
        },
        Command::Meta => {
            debug!("CMD: meta");
            MOCK_CONF.handle_meta().into()
        },
        Command::Noop => {
            debug!("CMD: noop");
            CommandResponse::ok()
        },
    }

}