// O-Sovereign TUI (Terminal UI using Ratatui)
// Note: Dioxus removed TUI support in 0.5, we use Ratatui instead

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use o_sovereign::{create_provider, ACSAConfig, ACSARouter, AgentRole};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use tokio::runtime::Runtime;

struct App {
    input: String,
    output: String,
    status: String,
    is_processing: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            input: String::new(),
            output: "Ready. Type your request and press Enter to execute.".to_string(),
            status: "Idle".to_string(),
            is_processing: false,
        }
    }
}

impl App {
    fn execute(&mut self, rt: &Runtime) {
        if self.input.is_empty() || self.is_processing {
            return;
        }

        let input = self.input.clone();
        self.is_processing = true;
        self.status = "Processing...".to_string();
        self.output = "⏳ Executing ACSA pipeline...".to_string();

        // Execute in async runtime
        match rt.block_on(execute_acsa_tui(input)) {
            Ok(result) => {
                self.output = result;
                self.status = "Completed".to_string();
            }
            Err(e) => {
                self.output = format!("❌ Error: {}", e);
                self.status = "Error".to_string();
            }
        }

        self.is_processing = false;
        self.input.clear();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Create async runtime
    let rt = Runtime::new()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::default();

    // Run app
    let res = run_app(&mut terminal, &mut app, &rt);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rt: &Runtime,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            // Only process key press events (ignore release)
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    return Ok(());
                }
                KeyCode::Char(c) => {
                    app.input.push(c);
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Enter => {
                    app.execute(rt);
                }
                KeyCode::Esc => {
                    return Ok(());
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Agent status
            Constraint::Length(3), // Status bar
            Constraint::Length(3), // Input
            Constraint::Min(5),    // Output
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new("🤖 O-Sovereign ACSA System (TUI Mode)")
        .style(Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)));
    f.render_widget(header, chunks[0]);

    // Agent Status Bar
    let agents = Line::from(vec![
        Span::styled("[MOSS 🧠]", Style::default().fg(Color::Cyan)),
        Span::raw(" "),
        Span::styled("[L6 🔬]", Style::default().fg(Color::Green)),
        Span::raw(" "),
        Span::styled("[Ultron 🛡️]", Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled("[Omega ⚡]", Style::default().fg(Color::Magenta)),
    ]);
    let agent_status = Paragraph::new(agents)
        .block(Block::default().borders(Borders::ALL).title("Agents"));
    f.render_widget(agent_status, chunks[1]);

    // Status Bar
    let status_style = if app.is_processing {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else if app.status == "Completed" {
        Style::default().fg(Color::Green)
    } else if app.status == "Error" {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::Gray)
    };

    let status = Paragraph::new(format!("Status: {}", app.status))
        .style(status_style)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);

    // Input Box
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input (Press Enter to execute)")
                .border_style(Style::default().fg(Color::Cyan)),
        );
    f.render_widget(input, chunks[3]);

    // Output Box
    let output = Paragraph::new(app.output.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("ACSA Output")
                .border_style(Style::default().fg(Color::Green)),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(output, chunks[4]);

    // Footer
    let footer = Paragraph::new("v0.1.0 | Press Ctrl+C or ESC to exit | Powered by Ratatui")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[5]);
}

async fn execute_acsa_tui(input: String) -> anyhow::Result<String> {
    // Use mock mode for TUI (no API keys required)
    let moss = create_provider(AgentRole::MOSS, None, true)?;
    let l6 = create_provider(AgentRole::L6, None, true)?;
    let ultron = create_provider(AgentRole::Ultron, None, true)?;
    let omega = create_provider(AgentRole::Omega, None, true)?;

    let config = ACSAConfig::default();
    let router = ACSARouter::new(moss, l6, ultron, omega, config);

    let log = router.execute(input).await?;

    // Format output for terminal
    let mut result = String::new();
    result.push_str(&format!("🎯 Success: {}\n", log.success));
    result.push_str(&format!("⏱️  Time: {} ms\n", log.total_time_ms));
    result.push_str(&format!("💰 Cost: ${:.4}\n", log.total_cost));
    result.push_str(&format!("🔁 Iterations: {}\n\n", log.iterations));

    if let Some(audit) = &log.audit_result {
        result.push_str(&format!("🛡️  Risk Score: {}/100\n", audit.risk_score));
        result.push_str(&format!("✅ Safe: {}\n\n", audit.is_safe));
    }

    result.push_str("📝 Final Output:\n");
    result.push_str(&format!(
        "{}",
        log.final_output.unwrap_or_else(|| "N/A".to_string())
    ));

    Ok(result)
}
