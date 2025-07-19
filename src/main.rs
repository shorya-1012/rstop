use std::time::Duration;

use crossterm::event::{self, Event};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Stylize},
    widgets::{Block, Borders, Clear, Paragraph},
};
use sysinfo::System;

fn bytes_to_gib(value: u64) -> f64 {
    value as f64 / 1024.0 / 1024.0 / 1024.0
}

fn main() {
    let mut sys = System::new_all();

    let mut terminal = ratatui::init();

    loop {
        sys.refresh_memory();
        sys.refresh_cpu_usage();
        let ram_data = get_ram_data(&sys);
        let cpu_data = get_cpu_data(&sys);

        terminal
            .draw(|frame| render(frame, &ram_data, &cpu_data))
            .expect("Unable to Draw");

        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(_) = event::read().unwrap() {
                break;
            }
        }

        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    }

    ratatui::restore();
}

fn render(frame: &mut Frame, ram_data: &str, cpu_data: &str) {
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.area());

    frame.render_widget(Clear, frame.area());

    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(outer_layout[0]);

    frame.render_widget(
        Paragraph::new("Graph Goes here")
            .block(Block::new().title("RAM").bold().borders(Borders::ALL)),
        inner_layout[0],
    );

    frame.render_widget(
        Paragraph::new(ram_data).block(
            Block::new()
                .title("Usage")
                .bold()
                .fg(Color::Blue)
                .borders(Borders::ALL),
        ),
        inner_layout[1],
    );

    frame.render_widget(
        Paragraph::new(cpu_data).block(Block::new().title("CPU").bold().borders(Borders::ALL)),
        outer_layout[1],
    );
}

fn get_ram_data(sys: &System) -> String {
    // format!(
    //     "Total Ram : {:.2} Gib\nUsed Ram : {:.2} Gib\nTotal Swap : {:.2} Gib \nUsed Swap : {:.2} Gib",
    //     bytes_to_gib(sys.total_memory()),
    //     bytes_to_gib(sys.used_memory()),
    //     bytes_to_gib(sys.total_swap()),
    //     bytes_to_gib(sys.used_swap())
    // )
    format!(
        "{:.2}/{:.2}GiB",
        bytes_to_gib(sys.used_memory()),
        bytes_to_gib(sys.total_memory())
    )
}

fn get_cpu_data(sys: &System) -> String {
    let mut cpu_data = String::from("");
    let mut count = 0;
    for cpu in sys.cpus() {
        let cpu_usage = format!("CPU{} : {:.2}%", count, cpu.cpu_usage());
        cpu_data.push_str(&cpu_usage);
        cpu_data.push('\n');
        count += 1;
    }
    cpu_data
}
