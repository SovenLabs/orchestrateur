//! Rendu ratatui — widgets texte uniquement.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use super::state::{AppState, View};

/// Dessine l'interface complète.
pub fn draw(frame: &mut Frame<'_>, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_header(frame, chunks[0], state);
    draw_body(frame, chunks[1], state);
    draw_footer(frame, chunks[2], state);

    if state.current_view == View::Help {
        draw_help(frame);
    }
}

fn draw_header(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let version = state
        .version
        .as_deref()
        .map(|v| format!(" v{v}"))
        .unwrap_or_default();
    let degraded = !state.llm_available || !state.embedding_available;
    let title = if degraded {
        format!("Orchestrateur TUI{version} [dégradé]")
    } else {
        format!("Orchestrateur TUI{version}")
    };
    let color = if degraded { Color::Red } else { Color::Cyan };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(color));
    frame.render_widget(block, area);
}

fn draw_body(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    match state.current_view {
        View::Detail => draw_detail(frame, area, state),
        View::Assimilate => draw_assimilate(frame, area, state),
        View::List | View::Help => draw_list(frame, area, state),
    }
}

fn draw_list(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let mode = if state.showing_search_results {
        "Recherche"
    } else {
        "Liste"
    };
    let filter_hint = if state.search_mode {
        format!(" / {}", state.input_buffer)
    } else if state.input_buffer.is_empty() {
        String::new()
    } else {
        format!(" filtre: {}", state.input_buffer)
    };

    let items: Vec<ListItem<'_>> = state
        .items
        .iter()
        .enumerate()
        .map(|(idx, mem)| {
            let tags = if mem.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", mem.tags.join(", "))
            };
            let line = format!("{} | {}{}", mem.id, mem.title, tags);
            let style = if idx == state.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(format!(
        "{mode}{filter_hint} ({}/{})",
        state.items.len(),
        state.total
    )));
    frame.render_widget(list, area);
}

fn draw_detail(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let detail = state.detail.as_ref();
    let title = detail.map_or("Détail", |d| d.title.as_str());
    let body = detail.map_or("Chargement…", |d| d.content.as_str());
    let tags = detail
        .map(|d| d.tags.join(", "))
        .filter(|t| !t.is_empty())
        .map(|t| format!("\n\nTags: {t}"))
        .unwrap_or_default();

    let paragraph = Paragraph::new(format!("{body}{tags}"))
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_assimilate(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let text = if state.assimilate_text.is_empty() {
        "Saisissez le texte à assimiler…".to_string()
    } else {
        state.assimilate_text.clone()
    };
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Assimilation (Entrée envoie, Esc annule)"),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let help = match state.current_view {
        View::List => {
            "j/k:nav  Entrée:détail  /:recherche  a:assimiler  r:refresh  ?:aide  q:quitter"
        }
        View::Detail => "Esc:retour  a:assimiler  ?:aide",
        View::Assimilate => "Entrée:envoyer  Esc:annuler",
        View::Help => "Esc ou ?:fermer aide",
    };
    let status_color = if !state.llm_available || !state.embedding_available {
        Color::Red
    } else {
        Color::Green
    };
    let line = Line::from(vec![
        Span::styled(
            state.status_message.clone(),
            Style::default().fg(status_color),
        ),
        Span::raw(" — "),
        Span::styled(help, Style::default().fg(Color::DarkGray)),
    ]);
    let paragraph = Paragraph::new(line)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}

fn draw_help(frame: &mut Frame<'_>) {
    let area = centered_rect(60, 50, frame.area());
    frame.render_widget(Clear, area);
    let text = vec![
        Line::from("Orchestrateur — Aide TUI"),
        Line::from(""),
        Line::from("Liste: j/k ou flèches, Entrée = détail"),
        Line::from("Filtre local: tapez du texte, Esc efface"),
        Line::from("/ = recherche sémantique (vectorielle)"),
        Line::from("a = assimiler via LLM"),
        Line::from("r = rafraîchir depuis LanceDB"),
        Line::from("q = quitter  Ctrl+C = quitter"),
        Line::from(""),
        Line::from("Bridge: Command/Response — zéro logique métier ici"),
    ];
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Aide")
                .style(Style::default().fg(Color::White)),
        )
        .alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
