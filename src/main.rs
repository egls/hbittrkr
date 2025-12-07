use chrono::{Datelike, Days, Local, NaiveDate};
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::{collections::HashMap, error::Error, io};

enum ViewMode {
    Year,
    Month,
}

/// App holds the state of the application
struct App {
    /// A log of days where alcohol was consumed.
    alcohol_log: HashMap<NaiveDate, bool>,
    /// The currently selected date.
    cursor: NaiveDate,
    /// The current view mode.
    view_mode: ViewMode,
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
            view_mode: ViewMode::Year,
            should_quit: false,
        }
    }

    fn set_view_mode(&mut self, view_mode: ViewMode) {
        self.view_mode = view_mode;
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

    fn move_cursor_left_month(&mut self) {
        self.cursor = self.cursor.checked_sub_days(Days::new(1)).unwrap();
    }

    fn move_cursor_right_month(&mut self) {
        self.cursor = self.cursor.checked_add_days(Days::new(1)).unwrap();
    }

    fn move_cursor_up_month(&mut self) {
        self.cursor = self.cursor.checked_sub_days(Days::new(7)).unwrap();
    }

    fn move_cursor_down_month(&mut self) {
        self.cursor = self.cursor.checked_add_days(Days::new(7)).unwrap();
    }

    fn next_month(&mut self) {
        let (year, month) = (self.cursor.year(), self.cursor.month());
        let next_month = if month == 12 { 1 } else { month + 1 };
        let year = if month == 12 { year + 1 } else { year };
        self.cursor = NaiveDate::from_ymd_opt(year, next_month, 1).unwrap();
    }

    fn prev_month(&mut self) {
        let (year, month) = (self.cursor.year(), self.cursor.month());
        let prev_month = if month == 1 { 12 } else { month - 1 };
        let year = if month == 1 { year - 1 } else { year };
        self.cursor = NaiveDate::from_ymd_opt(year, prev_month, 1).unwrap();
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Char(' ') => app.toggle_selected_day(),
                KeyCode::Left => match app.view_mode {
                    ViewMode::Year => app.move_cursor_left(),
                    ViewMode::Month => app.move_cursor_left_month(),
                },
                KeyCode::Right => match app.view_mode {
                    ViewMode::Year => app.move_cursor_right(),
                    ViewMode::Month => app.move_cursor_right_month(),
                },
                KeyCode::Up => match app.view_mode {
                    ViewMode::Year => app.move_cursor_up(),
                    ViewMode::Month => app.move_cursor_up_month(),
                },
                KeyCode::Down => match app.view_mode {
                    ViewMode::Year => app.move_cursor_down(),
                    ViewMode::Month => app.move_cursor_down_month(),
                },
                KeyCode::PageUp => {
                    if let ViewMode::Month = app.view_mode {
                        app.prev_month()
                    }
                }
                KeyCode::PageDown => {
                    if let ViewMode::Month = app.view_mode {
                        app.next_month()
                    }
                }
                KeyCode::Char('y') => app.set_view_mode(ViewMode::Year),
                KeyCode::Char('m') => app.set_view_mode(ViewMode::Month),
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

    let title = match app.view_mode {
        ViewMode::Year => format!("Year {}", app.cursor.year()),
        ViewMode::Month => format!("{} {}", app.cursor.format("%B"), app.cursor.year()),
    };
    let title_block = Block::default().borders(Borders::ALL).title(title);
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

    match app.view_mode {
        ViewMode::Year => {
            let habit_graph = HabitGraph {
                data: &app.alcohol_log,
                cursor: app.cursor,
            };
            f.render_widget(habit_graph, graph_area);
        }
        ViewMode::Month => {
            let month_view = MonthView {
                data: &app.alcohol_log,
                cursor: app.cursor,
            };
            f.render_widget(month_view, graph_area);
        }
    }

    let cursor_date = app.cursor.format("%A %d.%m.%Y").to_string();
    let cursor_date_paragraph = Paragraph::new(cursor_date).alignment(Alignment::Center);
    f.render_widget(cursor_date_paragraph, chunks[2]);

    let instructions_block = Block::default()
        .borders(Borders::ALL)
        .title("Use arrow keys to move. Press <space> to toggle a day. Press <y> for year view, <m> for month view. Use <PageUp> and <PageDown> to switch months. Press <q> to quit.");
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

        self.render_graph(area, buf);
    }
}

impl HabitGraph<'_> {

    fn render_graph(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let today = Local::now().date_naive();
        let year = self.cursor.year();

        let months_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Ratio(1, 12),
                    Constraint::Ratio(1, 12),
                    Constraint::Ratio(1, 12),
                    Constraint::Ratio(1, 12),
                    Constraint::Ratio(1, 12),
                    Constraint::Ratio(1, 12),
                    Constraint::Ratio(1, 12),
                    Constraint::Ratio(1, 12),
                    Constraint::Ratio(1, 12),
                    Constraint::Ratio(1, 12),
                    Constraint::Ratio(1, 12),
                    Constraint::Ratio(1, 12),
                ]
                .as_ref(),
            )
            .split(area);

        for month in 1..=12 {
            let month_area = months_layout[month as usize - 1];
            let month_block = Block::default()
                .borders(Borders::ALL)
                .title(format!("{}", month));
            let inner_area = month_block.inner(month_area);
            month_block.render(month_area, buf);

            let first_day_of_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
            let (next_month_year, next_month) = if month == 12 {
                (year + 1, 1)
            } else {
                (year, month + 1)
            };
            let last_day_of_month =
                NaiveDate::from_ymd_opt(next_month_year, next_month, 1)
                    .unwrap()
                    .checked_sub_days(Days::new(1))
                    .unwrap();

            let mut current_date = first_day_of_month;
            while current_date <= last_day_of_month {
                let day_of_week = current_date.weekday().num_days_from_sunday() as u16;
                let week_number = (current_date.day0() + first_day_of_month.weekday().num_days_from_sunday()) / 7;

                let (symbol, mut color) = match self.data.get(&current_date) {
                    Some(true) => ("■", Color::Red),
                    Some(false) => ("■", Color::Green),
                    None => ("□", Color::Rgb(50, 50, 50)), // No data
                };

                if current_date.day() == 1 {
                    color = Color::Yellow;
                }

                if current_date == today {
                    color = Color::Cyan;
                }

                if current_date == self.cursor {
                    color = Color::White;
                }

                buf.set_string(
                    inner_area.x + day_of_week * 2,
                    inner_area.y + week_number as u16,
                    symbol,
                    Style::default().fg(color),
                );

                current_date = current_date.succ_opt().unwrap();
            }
        }
    }
}

struct MonthView<'a> {
    data: &'a HashMap<NaiveDate, bool>,
    cursor: NaiveDate,
}

impl Widget for MonthView<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::default().borders(Borders::ALL);
        let inner_area = block.inner(area);
        block.render(area, buf);

        let today = Local::now().date_naive();
        let year = self.cursor.year();
        let month = self.cursor.month();

        let first_day_of_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let (next_month_year, next_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
        let last_day_of_month =
            NaiveDate::from_ymd_opt(next_month_year, next_month, 1)
                .unwrap()
                .checked_sub_days(Days::new(1))
                .unwrap();

        let month_name = self.cursor.format("%B").to_string();
        let title = format!("{} {}", month_name, year);
        Paragraph::new(title)
            .alignment(Alignment::Center)
            .render(inner_area, buf);

        let calendar_area = Rect {
            x: inner_area.x + (inner_area.width - 28) / 2,
            y: inner_area.y + 2,
            width: 28,
            height: 7,
        };

        let day_labels = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        for (i, label) in day_labels.iter().enumerate() {
            buf.set_string(
                calendar_area.x + i as u16 * 4,
                calendar_area.y,
                *label,
                Style::default(),
            );
        }

        let mut current_date = first_day_of_month;
        while current_date <= last_day_of_month {
            let week_day = current_date.weekday().num_days_from_sunday() as u16;
            let week_number = (current_date.day0() + first_day_of_month.weekday().num_days_from_sunday()) / 7;

            let (symbol, mut color) = match self.data.get(&current_date) {
                Some(true) => ("■", Color::Red),
                Some(false) => ("■", Color::Green),
                None => (" ", Color::Rgb(50, 50, 50)),
            };

            if current_date == today {
                color = Color::Cyan;
            }

            // draw the square
            buf.set_string(
                calendar_area.x + week_day * 4,
                calendar_area.y + 2 + week_number as u16,
                symbol,
                Style::default().fg(color),
            );

            // draw the day number
            buf.set_string(
                calendar_area.x + week_day * 4,
                calendar_area.y + 2 + week_number as u16,
                format!("{:2}", current_date.day()),
                if current_date == self.cursor {
                    Style::default().fg(Color::Black).bg(Color::White)
                } else {
                    Style::default().fg(Color::White)
                },
            );
            current_date = current_date.succ_opt().unwrap();
        }
    }
}
