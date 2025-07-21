mod utils;

use std::{collections::VecDeque, time::Duration};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Axis, Block, Borders, Chart, Clear, Dataset, Paragraph},
};
use sysinfo::{Disks, System};

use crate::utils::bytes_to_gib;

fn main() {
    let mut terminal = ratatui::init();

    let mut app = App::new();
    app.run(&mut terminal);

    ratatui::restore();
}

pub struct App {
    exit: bool,
    sys: System,
    ram_history: VecDeque<f64>,
    cpu_history: VecDeque<f64>,
    max_capacity: u8,
    ram_layout_color: Color,
    cpu_layout_color: Color,
}

impl App {
    fn new() -> Self {
        Self {
            exit: false,
            sys: System::new_all(),
            ram_history: VecDeque::new(),
            cpu_history: VecDeque::new(),
            max_capacity: 60,
            ram_layout_color: Color::White,
            cpu_layout_color: Color::White,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) {
        while !self.exit {
            let ram_data = self.get_ram_data();
            let disk_data = self.get_disk_data();
            self.push_cpu_value();

            terminal
                .draw(|frame| self.render(frame, &ram_data, &disk_data))
                .expect("Unable to Draw");

            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    if key.code == KeyCode::Char('q') {
                        self.exit = true;
                        break;
                    }
                }
            }

            std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        }
    }

    fn get_ram_data(&mut self) -> String {
        self.sys.refresh_memory();
        let total_memory = bytes_to_gib(self.sys.total_memory());
        let used_memeory = bytes_to_gib(self.sys.used_memory());
        self.push_ram_value(used_memeory, total_memory);
        format!("T:{:.2}GiB\nU:{:.2}GiB", total_memory, used_memeory,)
    }

    fn get_disk_data(&self) -> String {
        let disks = Disks::new_with_refreshed_list();
        let mut disk_data = String::from("");

        let mut total_space: u64 = 0;
        let mut total_avail_space = 0;
        let mut total_used_space = 0;

        for disk in disks.list() {
            total_space += disk.total_space();
            total_avail_space += disk.available_space();
            total_used_space += disk.total_space() - disk.available_space();
        }

        disk_data.push_str(&format!(
            "T:{:.2}G\nU:{:.2}G\nA:{:.2}G",
            bytes_to_gib(total_space),
            bytes_to_gib(total_used_space),
            bytes_to_gib(total_avail_space)
        ));
        disk_data
    }

    fn push_cpu_value(&mut self) {
        self.sys.refresh_cpu_usage();
        if self.ram_history.len() == self.max_capacity as usize {
            let _ = self.ram_history.pop_front();
        }
        self.cpu_history
            .push_back(self.sys.global_cpu_usage() as f64);
    }

    fn push_ram_value(&mut self, used: f64, total: f64) {
        if self.ram_history.len() == self.max_capacity as usize {
            let _ = self.ram_history.pop_front();
        }
        let percentage = (used) / (total) * 100.0;
        self.ram_history.push_back(percentage);
    }

    fn draw_ram_chart(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let points = self
            .ram_history
            .iter()
            .enumerate()
            .map(|(i, y)| (i as f64, *y))
            .collect::<Vec<(f64, f64)>>();

        let data_set = Dataset::default()
            .name("")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::LightGreen))
            .data(&points);

        let x_max = self.ram_history.len().max(1) as f64;
        let x_min = if x_max > 50.0 { x_max - 50.0 } else { 0.0 };

        let chart = Chart::new(vec![data_set])
            .block(
                Block::new()
                    .title("Memory")
                    .bold()
                    .fg(self.ram_layout_color)
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("")
                    .style(Style::default().fg(Color::White))
                    .bounds([x_min, x_max])
                    .labels(vec![Line::from("12s"), Line::from("0s")]),
            )
            .y_axis(
                Axis::default()
                    .title("")
                    .style(Style::default().fg(Color::White))
                    .bounds([0.0, 100.0])
                    .labels(vec![
                        Line::from("0%"),
                        Line::from("50%"),
                        Line::from("100%"),
                    ]),
            );

        frame.render_widget(chart, area);
    }

    fn draw_cpu_chart(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let points = self
            .cpu_history
            .iter()
            .enumerate()
            .map(|(i, y)| (i as f64, *y))
            .collect::<Vec<(f64, f64)>>();

        let data_set = Dataset::default()
            .name("")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Red))
            .data(&points);

        let x_max = self.cpu_history.len().max(1) as f64;
        let x_min = if x_max > 50.0 { x_max - 50.0 } else { 0.0 };

        let chart = Chart::new(vec![data_set])
            .block(
                Block::new()
                    .title(format!("CPU - {}", System::cpu_arch()))
                    .bold()
                    .fg(self.cpu_layout_color)
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("")
                    .style(Style::default().fg(Color::White))
                    .bounds([x_min, x_max])
                    .labels(vec![Line::from("12s"), Line::from("0s")]),
            )
            .y_axis(
                Axis::default()
                    .title("")
                    .style(Style::default().fg(Color::White))
                    .bounds([0.0, 100.0])
                    .labels(vec![
                        Line::from("0%"),
                        Line::from("50%"),
                        Line::from("100%"),
                    ]),
            );

        frame.render_widget(chart, area);
    }

    fn render(&self, frame: &mut Frame, ram_data: &str, disk_data: &str) {
        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(vec![Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(frame.area());

        frame.render_widget(Clear, frame.area());

        let ram_layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(outer_layout[1]);

        let ram_data_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(vec![Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(ram_layout[1]);

        self.draw_ram_chart(frame, ram_layout[0]);

        self.draw_cpu_chart(frame, outer_layout[0]);

        frame.render_widget(
            Paragraph::new(ram_data).block(
                Block::new()
                    .title("Usage")
                    .bold()
                    .fg(self.ram_layout_color)
                    .borders(Borders::ALL),
            ),
            ram_data_layout[0],
        );

        frame.render_widget(
            Paragraph::new(disk_data).block(
                Block::new()
                    .title("Disk")
                    .bold()
                    .fg(self.ram_layout_color)
                    .borders(Borders::ALL),
            ),
            ram_data_layout[1],
        );
    }
}
