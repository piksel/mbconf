use std::{error::Error, io::Write};

use color_eyre::eyre::{eyre};
use elytra_conf::{command::CommandKey, config::QueryTargetKey, entry::ExtraFlags};

pub mod wasm;
pub mod tcp;
pub mod tui;

pub trait ElytraDevice: Send {
    fn send_command_raw(&mut self, bytes: [u8; 64]) -> Result<[u8; 64], Box<dyn Error>>;
    fn log_chat(&mut self, bytes_out: [u8; 64], bytes_in: [u8; 64]);
    fn get_log(&mut self) -> Vec<([u8; 64], [u8; 64])>;
}

pub struct Section {
    pub entry: Entry,
    pub layout: Vec<(LayoutEntry, Entry)>
}

#[derive(Clone)]
pub enum LayoutEntry {
    Info(u8),
    Prop(u8),
}

#[derive(Clone)]
pub struct Entry {
    pub name: String,
    pub flags: ExtraFlags,
    pub variant: u8,
    pub constraints: [u8; 8],
    pub icon: Option<String>,
    pub help: Option<String>,
    pub entry_type: u8,
    pub layout: Option<Vec<LayoutEntry>>
}

pub struct Info {
    pub proto_version: u8,
    pub prop_count: u8,
    pub info_count: u8,
    pub section_count: u8,
    pub action_count: u8,
}

fn err_msg(bytes: &[u8]) -> String {
    String::from_utf8_lossy(&bytes[2..]).trim_end_matches('\0').to_owned()
}

impl dyn ElytraDevice {
    pub fn get_entry(&mut self, entry_type: u8, index: u8) -> Result<Entry, Box<dyn Error>> {
        let res = self.send_command( &[
            CommandKey::Query as u8, 
            entry_type, index, 
            QueryTargetKey::Field as u8
        ])?;
        if res[0] != 1 { return Err(eyre!("Got error response: {} ({:02x?}) ", err_msg(&res), &res[1]))? }
        let flags = ExtraFlags::from_bits_truncate(res[1]);
        let variant = res[2];
        let mut constraints = [0u8; 8];
        constraints.copy_from_slice(&res[3..11]);
        let name = str::from_utf8(&res[11..])?.trim_end_matches('\0').to_owned();

        Ok(Entry {
            name,
            flags,
            variant,
            constraints,
            entry_type,
            help: None,
            icon: None,
            layout: None,
        })
    }

    pub fn get_entries(&mut self, entry_type: u8, count: usize) -> Result<Vec<Entry>, Box<dyn Error>> {
        (0..count).map(|index| self.get_entry(entry_type, index as u8)).collect()
    }


    pub fn get_info(&mut self) -> Result<Info, Box<dyn Error>>  {
        let mut res = self.send_command(&[CommandKey::Meta as u8])?.into_iter();
        if res.next() != Some(1) {
            Err(eyre!("Got fail response from device!"))?;
        }
        
        let proto_version = res.next().unwrap();
        let section_count = res.next().unwrap();
        let prop_count = res.next().unwrap();
        let info_count = res.next().unwrap();
        let action_count = res.next().unwrap();
        Ok(Info {
            proto_version,
            prop_count,
            info_count,
            section_count,
            action_count,
        })
    }

    pub fn get_extra(&mut self, vt: u8, index: u8, q: u8) -> Result<String, Box<dyn Error>>  {
        let res = self.send_command(&[b'q', vt, index, q])?;
        Ok(String::from_utf8_lossy(&res[1..]).trim_end_matches('\0').to_string())
    }

    pub fn get_layout(&mut self, index: u8) -> Result<Vec<LayoutEntry>, Box<dyn Error>>  {
        let mut res = self.send_command(&[b'q', b's', index, b'l'])?.into_iter();
        assert_eq!(1, res.next().unwrap());
        let mut entries = Vec::new();
        loop {
            let Some(ft) = res.next() else {
                break;
            };
            let Some(ix) = res.next() else {
                break;
            };
            if ft == 0 {
                break;
            }
            entries.push(match ft {
                b'c' => LayoutEntry::Prop(ix),
                b'i' => LayoutEntry::Info(ix),
                ft => panic!("Unknown field type: {:02x}", ft)
            });
        }
        Ok(entries)
    }

    pub fn send_command(&mut self, bytes: &[u8]) -> Result<[u8; 64], Box<dyn Error>> {
        let mut out_bytes= [0u8; 64];
        let _ = out_bytes.as_mut_slice().write(bytes)?;

        let in_bytes = self.send_command_raw(out_bytes)?;
        self.log_chat(out_bytes, in_bytes);

        Ok(in_bytes)
    }
}
