use std::thread;
use std::{sync::mpsc::Receiver};

use std::sync::mpsc::{Sender, channel};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use elytra_conf::entry::ExtraFlags;
use ratatui::text::Span;
use ratatui::prelude::*;
use ratatui::widgets::{Clear, List, ListDirection, ListItem, Padding, Row, Table};
use ratatui::{
    DefaultTerminal, Frame, buffer::Buffer, layout::{Constraint, Layout, Rect}, style::Stylize, symbols::border, text::{Line, Text}, widgets::{Block, Paragraph, Widget}
};

use crate::{ElytraDevice, Entry, Info, LayoutEntry, Section};


type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

enum Progress {
    Working((String, Vec<([u8; 64], [u8; 64])>)),
    Failed((String, Vec<([u8; 64], [u8; 64])>)),
    Done(DeviceInfo)
}

enum AppState {
    Working(LoadingWidget),
    Done(DeviceInfo)
}

pub fn run(mut device: Box<dyn ElytraDevice + 'static>) -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();

    // let info = device.get_info()?;
    let (tx, rx) = channel();

    thread::spawn(move || {
        let final_progress = match run_worker(&mut device, tx.clone()) {
            Ok(di) => Progress::Done(di),
            Err(e) => Progress::Failed((format!("{:?}", e), device.get_log()))
        };
        tx.send(final_progress).unwrap();
    });

    let result = App{ rx, state: AppState::Working(LoadingWidget::new()), exit: false }.run(&mut terminal);
    ratatui::restore();
    Ok(result?)
}

fn run_worker(mut device: &mut Box<dyn ElytraDevice + 'static>, tx: Sender<Progress>) -> Result<DeviceInfo> {
    let _ = tx.send(Progress::Working(("Getting device info".to_owned(), vec![])));
    let info = device.get_info()?;
    

    let mut sections = get_entries(&mut device, &tx, b's', info.section_count as usize, "sections")?;
    get_layout(&mut device, &tx, &mut sections)?;
    let props = get_entries(&mut device, &tx, b'c', info.prop_count as usize, "prop fields")?;
    let infos = get_entries(&mut device, &tx, b'i', info.info_count as usize, "info fields")?;
    let actions = get_entries(&mut device, &tx, b'a', info.action_count as usize, "actions")?;


    tx.send(Progress::Working(("Assembling sections".to_owned(), device.get_log())))?;

    // Err(format!("Misc error: {:#?}", actions.len()))?;

    let sections = sections.into_iter().map(|section_entry| {
        let layout = section_entry.layout.clone().unwrap_or_default();
        let layout = layout.into_iter().map(|le| {
            match le {
                LayoutEntry::Prop(ci) => (le, props[ci as usize].clone()),
                LayoutEntry::Info(ii) => (le, infos[ii as usize].clone())
            }
        }).collect();

        Section {entry: section_entry, layout}
    }).collect();
    
    
    Ok(DeviceInfo{
        info,
        sections,
        actions,
        section_index: 0
    })
}

fn get_extras(
        device: &mut Box<dyn ElytraDevice + 'static>, 
        tx: &Sender<Progress>, 
        entries: &mut Vec<Entry>, 
        extra_type: u8, 
        cond: impl Fn(&Entry) -> bool, 
        apply: impl Fn(&mut Entry, String) -> (), 
        name: &str) -> Result<()> {
    let count = entries.iter().filter(|e| cond(e)).count();
    let _ = tx.send(Progress::Working((format!("  Getting {} {}", count, name), device.get_log())));
    for (index, entry) in entries.iter_mut().enumerate().filter(|(_, e)| cond(e)) {
        apply(entry, device.get_extra(entry.entry_type, index as u8, extra_type)?);
    }
    // thread::sleep(std::time::Duration::from_secs(1));
    Ok(())
}

fn get_layout(
        device: &mut Box<dyn ElytraDevice + 'static>, 
        tx: &Sender<Progress>,
        sections: &mut Vec<Entry>) -> Result<()> {
    let _ = tx.send(Progress::Working((format!("  Getting section layout"), device.get_log())));

    
    for (index, entry) in sections.iter_mut().enumerate() {
        let layout = device.get_layout(index as u8)?;
        entry.layout = Some(layout)
    }
    // thread::sleep(std::time::Duration::from_secs(1));
    Ok(())
}

fn get_entries(device: &mut Box<dyn ElytraDevice + 'static>, tx: &Sender<Progress>, entry_type: u8, count: usize, n: &str) -> Result<Vec<Entry>> {
    let _ = tx.send(Progress::Working((format!("Getting {} {}", count, n), device.get_log())));
    // thread::sleep(std::time::Duration::from_secs(2));
    let mut entries = device.get_entries(entry_type, count as usize)?;

    get_extras(device, tx, &mut entries, 
        b'h',
        |e| e.flags.contains(ExtraFlags::HasHelp), 
        |e, extra| e.help = Some(extra),
        "help texts")?;

    get_extras(device, tx, &mut entries, 
        b'i',
        |e| e.flags.contains(ExtraFlags::HasIcon), 
        |e, extra| e.icon = Some(extra),
        "icons")?;

    Ok(entries)
}

struct DeviceInfo {
    info: Info,
    sections: Vec<Section>,
    section_index: usize,
    #[allow(unused)]
    actions: Vec<Entry>
}

pub struct App {
    exit: bool,
    state: AppState,
    rx: Receiver<Progress>,
}

impl App {

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {

        

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            if let Ok(progress) = self.rx.try_recv() {
                match progress {
                    Progress::Done(di) => {
                        self.state = AppState::Done(di)
                    },
                    Progress::Working((status, mut items)) => {
                        match &mut self.state {
                            AppState::Working(loading_widget) => {
                                loading_widget.statuses.push(status);
                                loading_widget.log.append(&mut items);
                            },
                            _ => {
                                self.state = AppState::Working(LoadingWidget { 
                                    log: items, 
                                    statuses: vec![status], 
                                    failure: None 
                                });
                            }
                        }
                    },
                    Progress::Failed((err, mut items)) => {
                        match &mut self.state {
                            AppState::Working(loading_widget) => {
                                loading_widget.log.append(&mut items);
                                loading_widget.failure = Some(err);
                            },
                            _ => {
                                self.state = AppState::Working(LoadingWidget { 
                                    log: items, 
                                    statuses: vec!["Initialization".into()],
                                    failure: Some(err)
                                });
                            }
                        }
                    },
                }
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());        
    }

    fn handle_events(&mut self) -> Result<()> {
        if ! event::poll(std::time::Duration::from_millis(100))? {
            return Ok(())
        }
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }

            _ => Ok(())
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up => self.update_selection(-1),
            KeyCode::Down => self.update_selection(1),
            _ => Ok(())
        }
    }

    fn exit(&mut self) -> Result<()> {
        self.exit = true;
        Ok(())
    }
    
    fn update_selection(&mut self, arg: i32) -> Result<()> {
        if let AppState::Done(dev_info) = &mut self.state {
            if arg > 0 {
                dev_info.section_index = (dev_info.section_index + 1).min(dev_info.sections.len() - 1);
            } else {
                dev_info.section_index = dev_info.section_index.saturating_sub(1);
            }
        }
        Ok(())
    }

}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {

        let title = Line::from(" Elytra ".bold());
        let block = Block::bordered()
            .title(title.centered())
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);


        let area = block.inner(area);
        block.render(area, buf);
        match &self.state {
            AppState::Working(progress) => progress.render(area, buf),
            AppState::Done(device_info) => device_info.render(area, buf),
            // AppState::Failed(error) => FailedWidget(error.clone()).render(area, buf),
        }
    }
}

impl Widget for &DeviceInfo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        
        let vertical = Layout::vertical([
            Constraint::Length(6), 
            Constraint::Fill(1)
            // Constraint::Percentage(50), 
        ]).spacing(0)
        .vertical_margin(1)
        .horizontal_margin(2);
        
        let rows = vertical.split(area);

        Paragraph::new(Text::from_iter([
            Line::from_iter([ 
                Span::from("Version:"), 
                Span::from(format!("{}", self.info.proto_version))
            ])
        ]))
        .block(Block::bordered().title(" Info ").padding(Padding::uniform(1)))
        
        .render(rows[0], buf);

        let max_section_name = self.sections.iter().map(|s| s.entry.name.len()).max().unwrap_or(20);

        let horz = Layout::horizontal([Constraint::Length(max_section_name as u16 + 4), Constraint::Fill(1)])
            .spacing(1);
        let horz = horz.split(rows[1]);


        
        // let tabs = Tabs::new(self.sections.iter().map(|s| s.entry.name.clone()))
        //         .block(Block::bordered().title(" Sections "))
        //         .select(0);
        let tabs = Paragraph::new(
           Text::from_iter(self.sections.iter().enumerate().map(|(index, section)| {
            Line::from(format!(" {:max_section_name$} ", section.entry.name)).style(if index == self.section_index {
                Style::new().bg(Color::White).fg(Color::Black)
            } else {
                Style::new()
            })
        })))
        .block(Block::bordered().title(" Sections ").padding(Padding::symmetric(0, 0)).title_alignment(Alignment::Center));
        tabs.render(horz[0], buf);

        if let Some(section) = self.sections.get(self.section_index) {
            let section_text = Text::from_iter(section.layout.iter().flat_map(|(_, e)|
                [
                    Line::from_iter([ 
                        Span::from(format!("{}", e.name)), 
                    ]),
                    // Line::from("                 ").underlined(),
                    Line::from_iter([ 
                        Span::from(format!("{}", e.help.clone().unwrap_or_default())).fg(Color::DarkGray)
                    ]),
                    Line::from(""),
                ]
            ));
            let para = Paragraph::new(section_text)
                .left_aligned()
                .block(Block::bordered().padding(Padding::symmetric(2, 1))
                    .title(Line::from(format!(" {} ", section.entry.name))))
                ;
            Widget::render(Clear, horz[1], buf);
            para.render(horz[1], buf);
        }
        

        // let horizontal = Layout::horizontal((0..2).map(|_| Constraint::Fill(1))).spacing(1);
        // let vertical = Layout::vertical((0..3).map(|_| Constraint::Min(20))).spacing(1);
        // let rows = vertical.split(rows[1]);
        // let cells = rows.iter().flat_map(|&row| horizontal.split(row).to_vec());

        // // let section_layout = Layout::default().direction(Direction::Vertical).constraints(
        // //     (0..self.sections.len()).map(|_| Constraint::Max(10))
        // // ).split(rows[1]);

        // for eor in cells.zip_longest(self.sections.iter()) {
        //     let (area, section) = match eor {
        //         itertools::EitherOrBoth::Both(area, section) => (area, section),
        //         itertools::EitherOrBoth::Left(area) => {
        //             Widget::render(Clear, area, buf);
        //             // Clear::default().render(area, buf);
        //             break;
        //         },
        //         itertools::EitherOrBoth::Right(_) => {break},
        //     };
        //     let section_text = Text::from_iter(section.layout.iter().flat_map(|(_, e)|
        //         [
        //             Line::from_iter([ 
        //                 Span::from(format!("{}", e.name)), 
        //             ]),
        //             // Line::from("                 ").underlined(),
        //             Line::from_iter([ 
        //                 Span::from(format!("{}", e.help.clone().unwrap_or_default())).fg(Color::DarkGray)
        //             ]),
        //             Line::from(""),
        //         ]
        //     ));
        //     let para = Paragraph::new(section_text)
        //         .left_aligned()
        //         .block(Block::bordered().padding(Padding::symmetric(2, 1))
        //             .title(Line::from(format!(" {} ", section.entry.name))))
        //         ;
        //     Widget::render(Clear, area, buf);
        //     para.render(area, buf);
        // };


        
       // Widget::render(list, rows[1], buf);

    }
}

struct LoadingWidget {
    log: Vec<([u8; 64], [u8; 64])>,
    statuses: Vec<String>,
    failure: Option<String>
}
impl LoadingWidget {
    fn new() -> Self {
        Self { log: vec![], statuses: vec![], failure: None }
    }
}

impl Widget for &LoadingWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {

        let vertical = Layout::vertical([
            Constraint::Percentage(50), 
            Constraint::Percentage(50), 
        ]).spacing(0)
        .vertical_margin(1)
        .horizontal_margin(2);
        let rows = vertical.split(area);

        let loading_text = 
            self.statuses.iter().enumerate().map(|(i, status)| 
                ListItem::from(if i == self.statuses.len() -1 {
                    if let Some(failure) = &self.failure {Line::from_iter([
                        Span::from(format!("{:width$}🅧", "", width = status.chars().take_while(|c| c.is_whitespace()).count())).fg(Color::LightRed),
                        Span::from(format!(" {}", status.trim_start())).fg(Color::Red),
                        Span::from(" Failed: "),
                        Span::from(format!("{}", failure)).fg(Color::White),
                        ])
                    } else {Line::from_iter([
                        Span::from(format!("  {}", status)).style(Style::new().white()),
                        Span::from("...")])
                    }
                } else {Line::from_iter([
                    Span::from(format!("{:width$}✔", "", width = status.chars().take_while(|c| c.is_whitespace()).count())).style(Style::new().green()), 
                    Span::from(format!(" {}", status.trim_start())).style(Style::new().dark_gray()), 
                ])})
            
        );

        let list = List::new(loading_text)
            .direction(ListDirection::TopToBottom)
             .block(Block::bordered().title(" Status ")
             .padding(Padding::proportional(1)));

        Widget::render(list, rows[0], buf);

        let widths = [
            Constraint::Length(2),
            Constraint::Percentage(100),
            Constraint::Length(32),
        ];

        let table = Table::new(self.log.iter().rev().flat_map(|(bout, bin)| {
            let bout = fmt_hex_bytes(bout);
            let bin = fmt_hex_bytes(bin);
            [
                Row::new([ Text::from(">>"), bout.0, bout.1 ]).height(2),
                Row::new([ Text::from("<<"), bin.0, bin.1 ]).height(2),
            ]
        }), widths)
        .block(Block::bordered().title(" Device Communication ").padding(Padding::proportional(1)))
        .header(Row::new(vec!["Dir", "Bytes (hex)", "ASCII"])
            .style(Style::new().bold())
            .bottom_margin(0)
        );
        Widget::render(table, rows[1], buf);

    }
}

fn fmt_hex_bytes(bytes: &[u8; 64]) -> (Text<'_>, Text<'_>) {

        // let (h, a): (Vec<Span<'_>>, Vec<Span<'_>>) = fmt_chunk(bytes).iter()
        // .map(|(b, c, color)| 
        //     (
        //         Span::from(format!("{:02x} ", b.color(*color))),
        //         Span::from(format!("{} ", c.color(*color)))
        //     )
        // ).enumerate().partition(|(i, _)| *i >= 32);
    


    let (h, a) = bytes.chunks(32)
        .map(fmt_chunk)
        .map(|row| 

            (
                Line::from_iter(row.iter().map(|(b, _, style)|
                    Span::from(format!("{:02x} ", b)).style(*style))),

                Line::from_iter(row.iter().map(|(_, c, style)|
                    Span::from(format!("{}", c)).style(*style))) 
            )
            //(Text::from(""), Text::from(""))
        ).fold((Vec::new(), Vec::new()), |(mut hex, mut ascii), x| {
            hex.push(x.0);
            ascii.push(x.1);
            (hex, ascii)
        });
    (Text::from_iter(h), Text::from_iter(a))
}

fn fmt_chunk(chunk: &[u8]) -> Vec<(u8, char, Style)> {
    chunk.iter().copied().map(|b| {
        let c = char::try_from(b).unwrap_or('.');
        if c.is_control() {
            if (1..=9).contains(&b) {
                (b, char::from_digit(b as u32, 10).unwrap(), Style::new().light_cyan())
            } else { 
                (b, '.', Style::new().dark_gray())
            }
        } else { 
            (b, c, Style::new().light_yellow())
        }
    }).collect()
}

// fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
//     let [area] = Layout::horizontal([horizontal])
//         .flex(Flex::Center)
//         .areas(area);
//     let [area] = Layout::vertical([vertical])
//         .flex(Flex::Center)
//         .areas(area);
//     area
// }