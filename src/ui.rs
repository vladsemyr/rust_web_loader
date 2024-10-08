use crate::Statistic;
use chrono::Local;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::crossterm::{event, ExecutableCommand};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use std::io;
use std::io::stdout;
use std::sync::{Arc, RwLock};

pub fn ui_main(draw_ui: bool, statistic: Arc<RwLock<Statistic>>) -> io::Result<()> {
    enable_raw_mode()?;

    if draw_ui {
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let mut should_quit = false;
        while !should_quit {
            terminal.draw(|frame| ui(frame, &statistic))?;
            should_quit = handle_events()?;
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
    } else {
        let mut should_quit = false;
        while !should_quit {
            should_quit = handle_events()?;
        }
    }

    Ok(())
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(200))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn ui(frame: &mut Frame, statistic: &Arc<RwLock<Statistic>>) {
    let resp_code;
    let cps;
    let other_err;
    let error_log;

    {
        let r = statistic.read().unwrap();
        resp_code = r.resp_code.clone();
        cps = r.cps;
        other_err = r.other_err;
        error_log = r.error_log.clone();
    };

    let [left_area, right_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(frame.area());

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(format!("Время {}", Local::now().format("%H:%M:%S"))),
            Line::from(format!("HTTP code {:?}", resp_code)),
            Line::from(format!("Кол-во остальных ошибок {}", other_err)),
            Line::from(format!("CPS {}", cps)),
            Line::from(format!("")),
            Line::from(format!("Нажмите 'q' для выхода")),
        ])
        .block(Block::bordered().title("Статистика").border_style(Style::new().light_blue().bold())),
        left_area,
    );

    frame.render_widget(
        Paragraph::new(
            error_log
            .into_iter()
            .rev()
            .collect::<Vec<_>>().join("\n")
            //.collect::<List>()
            //.,
        )
        .style(Style::new().red())
        .wrap(Wrap {trim: false})
        .block(Block::bordered().title("Журнал").border_style(Style::new().light_blue().bold())),
        right_area,
    );
}
