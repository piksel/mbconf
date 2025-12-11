use std::net::ToSocketAddrs;
use std::{error::Error, net::SocketAddr};
use std::path::PathBuf;
use elytra_conf::entry::ExtraFlags;
use elytra_conf::config::QueryTargetKey;
use elytra_conf::values::ValueType;

use owo_colors::{AnsiColors, OwoColorize};

use clap::{Args, Parser, Subcommand};

use elytra_cli::{ElytraDevice, tcp::TcpDevice, wasm::WasmDevice, Entry, LayoutEntry, Section, tui};

#[derive(Debug, Clone)]
enum DeviceType {
    Tcp(Vec<SocketAddr>),
    Wasm(PathBuf),
    Serial
}

fn parse_query_prop(s: &str) -> Result<QueryTargetKey, String> {
    QueryTargetKey::try_from(s).map_err(|e| format!("{:?}", e))
}

fn parse_device_type(s: &str) -> Result<DeviceType, String> {
    if let Ok(addrs) = ToSocketAddrs::to_socket_addrs(s) {
        return Ok(DeviceType::Tcp(addrs.collect()))
    }

    if s == "serial" {
       return Ok(DeviceType::Serial)
    }

    let path = PathBuf::from(s);
    if path.exists() && path.is_file() {
        Ok(DeviceType::Wasm(path))
    } else {
        Err(format!("Not a valid device or wasm file path: \"{}\"", s))
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Interactive browser of configuration
    Tui,

    /// Query for basic information
    Info,

    /// Send a specific query command
    Query(QueryArgs),

    /// View a section summary
    Sections
}

/// Elytra command line tool
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct AppArgs {
    /// Device
    #[arg(short, long, value_parser = parse_device_type)]
    device: DeviceType,

    #[command(subcommand)]
    command: Option<Commands>
}

fn main() -> Result<(), Box<dyn Error>> {
    
    let cli = AppArgs::parse();

    let device: Box<dyn ElytraDevice> = match cli.device {
        DeviceType::Wasm(path) => Box::new(WasmDevice::new(&path)?),
        DeviceType::Tcp(addrs) => Box::new(TcpDevice::new(addrs.as_slice())?),
        DeviceType::Serial =>  Err("Serial support is not implemented".to_owned())?
    };

    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Tui => tui::run(device),
        Commands::Query(args) => run_query(device, args),
        Commands::Info => run_info(device),
        Commands::Sections => run_sections(device),
    }

}

#[derive(Debug, Args)]
struct QueryArgs {
    entry: char,
    index: u8,
    #[arg(value_parser = parse_query_prop)]
    prop: QueryTargetKey
}

fn run_info(mut device: Box<dyn ElytraDevice + 'static>) -> Result<(), Box<dyn Error>> {
    let info = device.get_info()?;
    println!("Version: {}", info.proto_version);

    println!("Sections: {}", info.section_count);
    println!("Prop fields: {}", info.prop_count);
    println!("Info fields: {}", info.info_count);
    println!("Actions: {}", info.action_count);
    Ok(())
}

fn run_sections(mut device: Box<dyn ElytraDevice + 'static>) -> Result<(), Box<dyn Error>> {
    let info = device.get_info()?;
    print_log(device.get_log());
    println!("Querying {} section(s)...", info.section_count.bright_blue());

    // println!("")
    let mut sections = Vec::with_capacity(info.section_count as usize);

    for i in 0..info.section_count {
        println!(" ~ Querying section #{} basic", i.bright_blue());
        let mut section_entry = device.get_entry(b's', i)?;
        print_log(device.get_log());
        println!(" ~ Querying section #{} layout", i.bright_blue());
        let layout_ids = device.get_layout(i)?;
        print_log(device.get_log());
        println!(" ~ Querying section extras...");
        if section_entry.flags.contains(ExtraFlags::HasHelp) {
            section_entry.help = Some(device.get_extra(b's', i, b'h')?);
            print_log(device.get_log());
        }
        if section_entry.flags.contains(ExtraFlags::HasIcon) {
            section_entry.icon = Some(device.get_extra(b's', i, b'i')?);
            print_log(device.get_log());
        }

        println!(" ~ Querying {} fields(s)...", layout_ids.len().bright_blue());
        let mut layout: Vec<(LayoutEntry, Entry)>  = layout_ids.into_iter().map(|layout| {
            match layout {
                LayoutEntry::Prop(li) => device.get_entry(b'c', li),
                LayoutEntry::Info(li) => device.get_entry(b'i', li),
            }.map(|entry| (layout, entry) ).unwrap()
        }).collect();
        print_log(device.get_log());
        
        println!(" ~ Querying field extras...");
        for (le, entry) in layout.iter_mut() {
            let (vt, index) = match le {
                LayoutEntry::Prop(li) => (b'c', li),
                LayoutEntry::Info(li) => (b'i', li),
            };
    
            if entry.flags.contains(ExtraFlags::HasHelp) {
                entry.help = Some(device.get_extra(vt, *index, b'h')?)
            }
            if entry.flags.contains(ExtraFlags::HasIcon) {
                entry.icon = Some(device.get_extra(vt, *index, b'i')?)
            }
        }
        print_log(device.get_log());

        let section = Section {
            entry: section_entry,
            layout,
        };
        sections.push(section);
    }

    println!();
    println!("{}", "Sections:".bright_white());
    for (i, section) in sections.iter().enumerate() {
        print!("- Section #{}: {}", i.bright_cyan(), section.entry.name.bright_yellow());
        if let Some(help) = &section.entry.help {
            println!(" {}", help.bright_black());
        } else {
            println!();
        }
        if let Some(icon) = &section.entry.icon {
            println!("  Icon: {}", icon.bright_white());
        }
        println!();

        for (l, entry) in &section.layout {
            let (field_type, ft_col) = match l {
                LayoutEntry::Prop(_) => ("C", AnsiColors::BrightGreen),
                LayoutEntry::Info(_) => ("I", AnsiColors::BrightMagenta),
            };
            let vt = ValueType::try_from(entry.variant).unwrap();
            
            print!("  [{}] {} {}", 
                field_type.color(ft_col), 
                vt.to_string().bright_blue(), 
                entry.name.bright_yellow());

            if entry.flags.contains(ExtraFlags::ReadOnly) {
                print!(" ({})", "ReadOnly".bright_red());
            } else {
                print!(" ({})", "Writable".bright_green());
            }
            if let Some(help) = &entry.help {
                println!(" {}", help.bright_black());
            } else {
                println!();
            }

            // print!("      Flags: ");
    
            if let Some(icon) = &entry.icon {
                println!("      Icon: {}", icon.bright_white());
            }

            println!();
            
            
        }
        println!();
    }
    Ok(())
}


fn run_query(mut device: Box<dyn ElytraDevice + 'static>, args: QueryArgs) -> Result<(), Box<dyn Error>> {
    
    let entry = args.entry;
    let index: u8 = args.index;
    let prop: QueryTargetKey = args.prop;
    let _ = device.send_command(&[b'q', entry as u8, index, prop as u8])?;
    print_log(device.get_log());

    Ok(())
}

fn print_log(log: Vec<([u8; 64],[u8; 64])>) {
    for (out_bytes, in_bytes) in log {
        eprint!("\r{} ", "~>".bright_green());
        print_bytes(&out_bytes);
        eprint!("\r{} ", "<~".bright_magenta());
        print_bytes(&in_bytes);
        eprintln!();
    }
}

fn print_bytes(bytes: &[u8]) {
    for chunk in bytes.chunks(32) {
        let colored_chunk: Vec<_> = chunk.iter().copied().map(|b| {
            let c = char::try_from(b).unwrap_or('.');
            if c.is_control() {
                if (1..=9).contains(&b) {
                    (b, char::from_digit(b as u32, 10).unwrap(), AnsiColors::BrightCyan)
                } else { 
                    (b, '.', AnsiColors::BrightBlack)
                }
            } else { 
                (b, c, AnsiColors::BrightYellow)
            }
        }).collect();

        for (b, _, color) in &colored_chunk {
            eprint!("{:02x} ", (*b).color(*color));
        }

        for (_, c, color) in colored_chunk {
            eprint!("{}", c.color(color));
        }

        eprint!("\n   ");
    }
}