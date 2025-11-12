use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span}, // ← FIXED
    widgets::{Block, Borders, Paragraph},
};
use std::io;
use std::time::{Duration, Instant};

//app state
#[derive(Default)]
struct App {
    sample: String,
    typed: String,
    start_time: Option<Instant>,
    finished: bool, // ← fixed typo
    quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            sample: "The quick brown fox jumps over the lazy dog.".to_string(),
            ..Default::default()
        }
    }
}

// ===== Main =====
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Setup ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(); // ← create app BEFORE loop

    // --- Main Loop ---
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if app.quit {
            break;
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_key(key, &mut app);
            }
        }
    }

    // --- Cleanup ---
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

// ui rendering

fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.area());

    // Top: colored text to type
    let mut spans = Vec::new();
    for (i, s_char) in app.sample.chars().enumerate() {
        let typed_char = app.typed.chars().nth(i);
        let style = match typed_char {
            Some(t) if t == s_char => Style::default().fg(Color::Green),
            Some(_) => Style::default().fg(Color::Red),
            None => Style::default().fg(Color::DarkGray),
        };
        spans.push(Span::styled(s_char.to_string(), style));
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .block(Block::default().title("Type this").borders(Borders::ALL));
    f.render_widget(paragraph, chunks[0]);

    // Bottom: stats
    let stats_text = if app.finished {
        let minutes = app.start_time.unwrap().elapsed().as_secs_f64() / 60.0;
        let words = app.sample.chars().filter(|c| *c == ' ').count() + 1; // rough word count
        let wpm = (words as f64) / minutes;
        let accuracy = accuracy(app);
        format!("Finished! WPM: {:.0} | Accuracy: {:.1}%", wpm, accuracy)
    } else if app.start_time.is_some() {
        let secs = app.start_time.unwrap().elapsed().as_secs_f64();
        format!("Typing... {:.1} seconds", secs)
    } else {
        "Press any key to start typing!".to_string()
    };

    let stats =
        Paragraph::new(stats_text).block(Block::default().title("Stats").borders(Borders::ALL));
    f.render_widget(stats, chunks[1]);
}

// ===== Input Handling =====
fn handle_key(key: crossterm::event::KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Esc => app.quit = true,
        KeyCode::Backspace => {
            app.typed.pop();
        }
        KeyCode::Char(c) => {
            if !app.finished {
                if app.start_time.is_none() {
                    app.start_time = Some(Instant::now());
                }
                app.typed.push(c);
                try_finish(app);
            }
        }
        _ => {}
    }
}

fn try_finish(app: &mut App) {
    if app.typed.len() >= app.sample.len() && app.typed == app.sample {
        app.finished = true;
    }
}

fn accuracy(app: &App) -> f64 {
    if app.typed.is_empty() {
        return 100.0;
    }
    let correct: usize = app
        .typed
        .chars()
        .zip(app.sample.chars())
        .filter(|(t, s)| t == s)
        .count();
    (correct as f64 / app.typed.len() as f64) * 100.0
}

