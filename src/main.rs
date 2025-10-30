use chrono::{Datelike, Days, Local, NaiveDate, Weekday};
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{canvas::Canvas, Block, Borders, Paragraph, Widget},
};
use std::{collections::HashMap, error::Error, io};

/// App holds the state of the application
struct App {
    /// A log of days where alcohol was consumed.
    alcohol_log: HashMap<NaiveDate, bool>,
    /// The currently selected date.
    cursor: NaiveDate,
    /// Should the application exit?
    should_quit: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it.
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

impl App {
    fn new() -> App {
        // Pre-populate with some dummy data for demonstration
        let mut alcohol_log = HashMap::new();
        let today = Local::now().date_naive();
        alcohol_log.insert(today.checked_sub_days(Days::new(1)).unwrap(), true);
        alcohol_log.insert(today.checked_sub_days(Days::new(2)).unwrap(), true);
        alcohol_log.insert(today.checked_sub_days(Days::new(5)).unwrap(), true);
        alcohol_log.insert(today.checked_sub_days(Days::new(10)).unwrap(), true);
        alcohol_log.insert(today.checked_sub_days(Days::new(12)).unwrap(), true);
        alcohol_log.insert(today.checked_sub_days(Days::new(13)).unwrap(), true);

        App {
            alcohol_log,
            cursor: today,
            should_quit: false,
        }
    }

    fn toggle_selected_day(&mut self) {
        let entry = self.alcohol_log.entry(self.cursor).or_insert(false);
        *entry = !*entry;
    }

    fn move_cursor_left(&mut self) {
        self.cursor = self.cursor.checked_sub_days(Days::new(7)).unwrap();
    }

    fn move_cursor_right(&mut self) {
        self.cursor = self.cursor.checked_add_days(Days::new(7)).unwrap();
    }

    fn move_cursor_up(&mut self) {
        self.cursor = self.cursor.checked_sub_days(Days::new(1)).unwrap();
    }

    fn move_cursor_down(&mut self) {
        self.cursor = self.cursor.checked_add_days(Days::new(1)).unwrap();
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Char(' ') => app.toggle_selected_day(),
                KeyCode::Left => app.move_cursor_left(),
                KeyCode::Right => app.move_cursor_right(),
                KeyCode::Up => app.move_cursor_down(),
                KeyCode::Down => app.move_cursor_up(),
                _ => {}
            }
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // For title
                Constraint::Min(0),    // For the graph
                Constraint::Length(1), // For cursor date
                Constraint::Length(3), // For instructions
                Constraint::Length(1), // For legend
            ]
            .as_ref(),
        )
        .split(f.area());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Year {}", Local::now().year()));
    f.render_widget(title_block, chunks[0]);

    let graph_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(4), Constraint::Min(0)])
        .split(chunks[1]);

    let day_labels = vec!["", "Mon", "", "Wed", "", "Fri", ""];
    let day_labels_paragraph = Paragraph::new(
        day_labels
            .iter()
            .map(|s| Line::from(*s))
            .collect::<Vec<_>>(),
    )
    .alignment(Alignment::Left);
    f.render_widget(day_labels_paragraph, graph_chunks[0]);

    let graph_block = Block::default().borders(Borders::ALL);
    let graph_area = graph_block.inner(graph_chunks[1]);
    f.render_widget(graph_block, graph_chunks[1]);

    let habit_graph = HabitGraph {
        data: &app.alcohol_log,
        cursor: app.cursor,
    };
    f.render_widget(habit_graph, graph_area);

    let cursor_date = app.cursor.format("%A %d.%m.%Y").to_string();
    let cursor_date_paragraph = Paragraph::new(cursor_date).alignment(Alignment::Center);
    f.render_widget(cursor_date_paragraph, chunks[2]);

    let instructions_block = Block::default()
        .borders(Borders::ALL)
        .title("Use arrow keys to move. Press <space> to toggle a day. Press <q> to quit.");
    f.render_widget(instructions_block, chunks[3]);

    let legend = Paragraph::new(Line::from(vec![
        Span::styled("■", Style::default().fg(Color::Red)),
        Span::raw(" Drank | "),
        Span::styled("■", Style::default().fg(Color::Green)),
        Span::raw(" Didn't Drink | "),
        Span::styled("■", Style::default().fg(Color::Yellow)),
        Span::raw(" First Day of Month | "),
        Span::styled("■", Style::default().fg(Color::Cyan)),
        Span::raw(" Today | "),
        Span::styled("■", Style::default().fg(Color::White)),
        Span::raw(" Cursor"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(legend, chunks[4]);
}

struct HabitGraph<'a> {
    data: &'a HashMap<NaiveDate, bool>,
    cursor: NaiveDate,
}

impl Widget for HabitGraph<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.width < 53 * 2 || area.height < 8 {
            // Not enough space to render the graph
            return;
        }

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        let month_area = layout[0];
        let graph_area = layout[1];

        HabitGraph::render_month_labels(&self, month_area, buf);
        HabitGraph::render_graph(&self, graph_area, buf);
    }
}

impl HabitGraph<'_> {
    fn render_month_labels(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let today = Local::now().date_naive();
        let start_of_year = NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap();

        let mut spans = vec![];
        let mut last_month_week = start_of_year.iso_week().week();

        for i in 0..12 {
            let month = i + 1;
            let month_date = NaiveDate::from_ymd_opt(today.year(), month, 1).unwrap();
            let month_week = month_date.iso_week().week();

            let week_diff = month_week - last_month_week;
            if week_diff > 0 {
                spans.push(Span::raw(" ".repeat(week_diff as usize * 2)));
            }
            spans.push(Span::styled(months[i as usize], Style::default()));
            last_month_week = month_week;
        }

        Paragraph::new(Line::from(spans)).render(area, buf);
    }

    fn render_graph(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let today = Local::now().date_naive();
        let year = today.year();
        let start_of_year = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();

        Canvas::default()
            .marker(ratatui::symbols::Marker::Block)
            .paint(|ctx| {
                let mut current_date = start_of_year;
                let mut week = 0.0;
                while current_date.year() == year {
                    let day_of_week = current_date.weekday().num_days_from_sunday() as f64;

                    let (symbol, mut color) = match self.data.get(&current_date) {
                        Some(true) => ("■", Color::Red),
                        Some(false) => ("■", Color::Green),
                        None => ("□", Color::Rgb(50, 50, 50)), // No data
                    };

                    if current_date.day() == 1 {
                        color = match self.data.get(&current_date) {
                            Some(true) => Color::Rgb(255, 165, 0), // Orange for drank
                            Some(false) => Color::Rgb(173, 255, 47), // GreenYellow for not drank
                            None => Color::Yellow,
                        };
                    }

                    if current_date == today {
                        color = Color::Cyan;
                    }

                    ctx.print(week, day_of_week, symbol.fg(color));

                    if current_date == self.cursor {
                        ctx.print(week, day_of_week, "■".fg(Color::White));
                    }

                    if current_date.weekday() == Weekday::Sat {
                        week += 1.0;
                    }
                    current_date = current_date.succ_opt().unwrap();
                }
            })
            .x_bounds([0.0, 52.0])
            .y_bounds([0.0, 6.0])
            .render(area, buf);
    }
}
