use elytra_conf::{field::FieldValue, command::{CommandError, CommandHandler}};

use crate::{Action, PropField, InfoField, MOCK_CONF};

pub struct MockHandler;

impl MockHandler {
    pub const fn new() -> Self {
        Self{}
    }
}

impl CommandHandler<PropField, InfoField, Action> for MockHandler {
    async fn noop(&mut self) {
        eprintln!("CMD: noop")
    }

    async fn read_prop(&mut self, prop_field: PropField) 
        -> Result<FieldValue, CommandError> {
       eprintln!("CMD: read_prop: {:?}", prop_field);
       Ok(FieldValue::from_store(MOCK_CONF.prop_field(prop_field), [0u8; 64]))
    }

    async fn write_prop(&mut self, prop_field: PropField, value: FieldValue) 
        -> Result<(), CommandError> {
       eprintln!("CMD: write_prop: {:?}", prop_field);
       eprintln!(" => {:x?}", value);
       Ok(())
    }

    async fn read_info(&mut self, info_field: InfoField) 
        -> Result<FieldValue, CommandError>  {
       eprintln!("CMD: read_info: {:?}", info_field);
       Ok(FieldValue::from_store(MOCK_CONF.info_field(info_field), [0u8; 64]))
    }

    async fn write_info(&mut self, info_field: InfoField, value: FieldValue) 
        -> Result<(), CommandError>  {
       eprintln!("CMD: write_info {:?}", info_field);
       eprintln!(" => {:x?}", value);
       Ok(())
    }

    async fn do_action(&mut self, action: Action)
        -> Result<(), CommandError> {
       eprintln!("CMD: action: {:?}", action);
       Ok(())
    }
}