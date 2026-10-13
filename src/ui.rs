//! Interface terminal (TUI) : liste des reglages, selection par cases a cocher,
//! volet de detail, application et restauration.

use crate::backup;
use crate::model::{Tweak, TweakState};
use crate::tweaks;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Terminal;
use std::io::Stdout;

struct Row {
    tweak: Tweak,
    state: TweakState,
    selected: bool,
}

struct App {
    rows: Vec<Row>,
    cursor: usize,
    status: String,
}

impl App {
    fn new() -> Self {
        let rows = tweaks::catalog()
            .into_iter()
            .map(|t| {
                let state = tweaks::detect(&t);
                Row {
                    tweak: t,
                    state,
                    selected: false,
                }
            })
            .collect();
        App {
            rows,
            cursor: 0,
            status: "Espace: cocher  |  a: appliquer  |  r: tout restaurer  |  q: quitter".into(),
        }
    }

    fn apply_selected(&mut self) {
        let mut backups = Vec::new();
        let mut count = 0;
        let mut reboot = false;
        for row in self.rows.iter_mut().filter(|r| r.selected) {
            match backup::capture(&row.tweak) {
                Ok(b) => backups.push(b),
                Err(e) => {
                    self.status = format!("Sauvegarde impossible pour {}: {e}", row.tweak.id);
                    return;
                }
            }
            if let Err(e) = tweaks::apply(&row.tweak) {
                self.status = format!("Echec sur {} (droits admin ?): {e}", row.tweak.id);
                return;
            }
            row.state = tweaks::detect(&row.tweak);
            row.selected = false;
            reboot |= row.tweak.requires_reboot;
            count += 1;
        }
        if count == 0 {
            self.status = "Rien de selectionne. Utilise Espace pour cocher.".into();
            return;
        }
        match backup::save_session(backups) {
            Ok(path) => {
                self.status = format!(
                    "{count} reglage(s) applique(s). Sauvegarde: {}{}",
                    path.display(),
                    if reboot { "  — redemarrage conseille." } else { "" }
                );
            }
            Err(e) => self.status = format!("Applique, mais sauvegarde non ecrite: {e}"),
        }
    }

    fn restore_latest(&mut self) {
        match backup::load_latest() {
            Ok(Some(session)) => {
                let mut ok = 0;
                for tb in &session.tweaks {
                    if backup::restore(tb).is_ok() {
                        ok += 1;
                    }
                }
                for row in &mut self.rows {
                    row.state = tweaks::detect(&row.tweak);
                }
                self.status = format!("Restauration : {ok} reglage(s) remis a leur etat anterieur.");
            }
            Ok(None) => self.status = "Aucune sauvegarde a restaurer.".into(),
            Err(e) => self.status = format!("Lecture de sauvegarde impossible: {e}"),
        }
    }
}

fn state_span(state: TweakState) -> Span<'static> {
    match state {
        TweakState::Disabled => Span::styled("[desactive]", Style::default().fg(Color::Green)),
        TweakState::Enabled => Span::styled("[actif]", Style::default().fg(Color::Yellow)),
        TweakState::Unknown => Span::styled("[inconnu]", Style::default().fg(Color::DarkGray)),
    }
}

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal: Terminal<CrosstermBackend<Stdout>> = Terminal::new(backend)?;

    let mut app = App::new();
    let res = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.cursor + 1 < app.rows.len() {
                        app.cursor += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.cursor = app.cursor.saturating_sub(1);
                }
                KeyCode::Char(' ') => {
                    if let Some(r) = app.rows.get_mut(app.cursor) {
                        r.selected = !r.selected;
                    }
                }
                KeyCode::Char('a') => app.apply_selected(),
                KeyCode::Char('r') => app.restore_latest(),
                _ => {}
            }
        }
    }
    Ok(())
}

fn draw(f: &mut ratatui::Frame, app: &App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(f.area());

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(outer[0]);

    // Liste des reglages
    let items: Vec<ListItem> = app
        .rows
        .iter()
        .map(|r| {
            let check = if r.selected { "[x] " } else { "[ ] " };
            ListItem::new(Line::from(vec![
                Span::raw(check),
                Span::raw(r.tweak.name),
                Span::raw("  "),
                state_span(r.state),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.cursor));
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Reglages de confidentialite Windows 11 "))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    f.render_stateful_widget(list, cols[0], &mut list_state);

    // Volet de detail
    let detail = if let Some(r) = app.rows.get(app.cursor) {
        vec![
            Line::from(vec![Span::styled(r.tweak.name, Style::default().add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from(vec![Span::styled("Categorie : ", Style::default().fg(Color::Cyan)), Span::raw(r.tweak.category.label())]),
            Line::from(vec![Span::styled("Risque    : ", Style::default().fg(Color::Cyan)), Span::raw(r.tweak.risk.label())]),
            Line::from(vec![Span::styled("Etat      : ", Style::default().fg(Color::Cyan)), state_span(r.state)]),
            Line::from(""),
            Line::from(vec![Span::styled("Ce que ca fait :", Style::default().add_modifier(Modifier::BOLD))]),
            Line::from(r.tweak.description),
            Line::from(""),
            Line::from(vec![Span::styled("Si desactive :", Style::default().add_modifier(Modifier::BOLD))]),
            Line::from(r.tweak.impact),
        ]
    } else {
        vec![Line::from("")]
    };
    let detail = Paragraph::new(detail)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title(" Detail "));
    f.render_widget(detail, cols[1]);

    // Barre de statut
    let status = Paragraph::new(app.status.as_str())
        .block(Block::default().borders(Borders::ALL).title(" Aide / statut "));
    f.render_widget(status, outer[1]);
}
