use std::cell::RefCell;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap};

use code_core::auth;
use code_core::{
    built_in_model_providers,
    MINIMAX_PROVIDER_ID,
    OPENCODE_GO_PROVIDER_ID,
};

use crate::app_event_sender::AppEventSender;
use crate::chatwidget::BackgroundOrderTicket;

use super::bottom_pane_view::{BottomPaneView, ConditionalUpdate};
use super::BottomPane;

/// Interactive view shown for `/provider` to manage stored provider keys.
pub(crate) struct ProviderCredentialsView {
    state: Rc<RefCell<ProviderCredentialsState>>,
}

impl ProviderCredentialsView {
    pub fn new(
        code_home: PathBuf,
        _app_event_tx: AppEventSender,
        _tail_ticket: BackgroundOrderTicket,
    ) -> (Self, Rc<RefCell<ProviderCredentialsState>>) {
        let state = Rc::new(RefCell::new(ProviderCredentialsState::new(code_home)));
        (Self { state: state.clone() }, state)
    }
}

impl<'a> BottomPaneView<'a> for ProviderCredentialsView {
    fn handle_key_event(&mut self, pane: &mut BottomPane<'a>, key_event: KeyEvent) {
        let mut state = self.state.borrow_mut();
        state.handle_key_event(key_event);
        if state.should_close() {
            state.set_complete();
        }
        pane.request_redraw();
    }

    fn is_complete(&self) -> bool {
        self.state.borrow().is_complete
    }

    fn desired_height(&self, width: u16) -> u16 {
        let state = self.state.borrow();
        state.desired_height(width) as u16
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let state = self.state.borrow();
        state.render(area, buf);
    }

    fn handle_paste(&mut self, _text: String) -> ConditionalUpdate {
        ConditionalUpdate::NoRedraw
    }
}

#[derive(Clone, Debug)]
struct ProviderRow {
    provider_ref: String,
    label: String,
    detail: Option<String>,
    is_configured: bool,
}

#[derive(Clone, Debug)]
struct Feedback {
    message: String,
    is_error: bool,
}

pub(crate) struct ProviderCredentialsState {
    code_home: PathBuf,
    providers: Vec<ProviderRow>,
    selected: usize,
    feedback: Option<Feedback>,
    is_complete: bool,
}

impl ProviderCredentialsState {
    fn new(code_home: PathBuf) -> Self {
        let mut state = Self {
            code_home,
            providers: Vec::new(),
            selected: 0,
            feedback: None,
            is_complete: false,
        };
        state.reload_providers();
        state
    }

    fn reload_providers(&mut self) {
        self.feedback = None;
        let previously_selected_ref = self
            .providers
            .get(self.selected)
            .map(|row| row.provider_ref.clone());

        let provider_defs = built_in_model_providers(None);
        let auth_file = auth::get_auth_file(&self.code_home);
        let auth_json = match auth::try_read_auth_json(&auth_file) {
            Ok(auth_json) => Some(auth_json),
            Err(err) if err.kind() == io::ErrorKind::NotFound => None,
            Err(err) => {
                self.feedback = Some(Feedback {
                    message: format!("Failed to read current provider keys: {err}"),
                    is_error: true,
                });
                None
            }
        };

        const PROVIDER_ORDER: [&str; 3] = [
            OPENCODE_GO_PROVIDER_ID,
            MINIMAX_PROVIDER_ID,
            "openai",
        ];

        self.providers = PROVIDER_ORDER
            .into_iter()
            .filter_map(|provider_ref| {
                provider_defs.get(provider_ref).map(|provider| {
                    let is_configured = auth_json
                        .as_ref()
                        .and_then(|auth| auth.provider_api_key(provider_ref))
                        .is_some();
                    ProviderRow {
                        provider_ref: provider_ref.to_string(),
                        label: provider.name.clone(),
                        detail: provider
                            .env_key_instructions
                            .clone()
                            .or_else(|| provider.env_key.as_ref().map(|env| env.to_string())),
                        is_configured,
                    }
                })
            })
            .collect();

        if self.providers.is_empty() {
            self.selected = 0;
        } else if let Some(previously_selected_ref) = previously_selected_ref {
            self.selected = self
                .providers
                .iter()
                .position(|row| row.provider_ref == previously_selected_ref)
                .unwrap_or(0);
        } else {
            self.selected = self.selected.min(self.providers.len() - 1);
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        let provider_count = self.providers.len();

        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.is_complete = true;
            }
            KeyCode::Up => {
                if provider_count == 0 {
                    self.selected = 0;
                } else if self.selected == 0 {
                    self.selected = provider_count - 1;
                } else {
                    self.selected -= 1;
                }
            }
            KeyCode::Down => {
                if provider_count == 0 {
                    self.selected = 0;
                } else {
                    self.selected = (self.selected + 1) % provider_count;
                }
            }
            KeyCode::Char('r') => {
                self.reload_providers();
            }
            _ => {}
        }
    }

    fn should_close(&self) -> bool {
        self.is_complete
    }

    fn set_complete(&mut self) {
        self.is_complete = true;
    }

    fn desired_height(&self, _width: u16) -> usize {
        const MIN_HEIGHT: usize = 9;
        let mut lines = 4; // title, spacer, footer spacer, footer
        if self.feedback.is_some() {
            lines += 2;
        }
        lines += self.providers.len().max(1);
        (lines + 2).max(MIN_HEIGHT)
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(crate::colors::border()))
            .style(Style::default().bg(crate::colors::background()).fg(crate::colors::text()))
            .title(" Manage Providers ")
            .title_alignment(Alignment::Center);
        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines = Vec::new();
        if let Some(feedback) = &self.feedback {
            let style = if feedback.is_error {
                Style::default().fg(crate::colors::error()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(crate::colors::success()).add_modifier(Modifier::BOLD)
            };
            lines.push(Line::from(vec![Span::styled(feedback.message.clone(), style)]));
            lines.push(Line::from(""));
        }

        lines.push(Line::from(vec![Span::styled(
            "Supported Providers",
            Style::default().add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        if self.providers.is_empty() {
            lines.push(Line::from(Span::styled(
                "No supported providers found.",
                Style::default().fg(crate::colors::text_dim()),
            )));
        } else {
            for (idx, provider) in self.providers.iter().enumerate() {
                let selected = idx == self.selected;
                let arrow_style = if selected {
                    Style::default().fg(crate::colors::primary())
                } else {
                    Style::default().fg(crate::colors::text_dim())
                };
                let label_style = if selected {
                    Style::default()
                        .fg(crate::colors::primary())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let status_style = if provider.is_configured {
                    Style::default()
                        .fg(crate::colors::success())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(crate::colors::warning())
                        .add_modifier(Modifier::BOLD)
                };

                let mut spans = vec![
                    Span::styled(if selected { "› " } else { "  " }, arrow_style),
                    Span::styled(provider.label.clone(), label_style),
                    Span::raw(" "),
                    Span::styled(
                        if provider.is_configured {
                            "(configured)"
                        } else {
                            "(missing)"
                        },
                        status_style,
                    ),
                ];

                if let Some(detail) = &provider.detail {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(
                        detail.clone(),
                        Style::default().fg(crate::colors::text_dim()),
                    ));
                }

                lines.push(Line::from(spans));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("↑↓", Style::default().fg(crate::colors::function())),
            Span::styled(" Navigate  ", Style::default().fg(crate::colors::text_dim())),
            Span::styled("r", Style::default().fg(crate::colors::warning()).add_modifier(Modifier::BOLD)),
            Span::styled(" Reload  ", Style::default().fg(crate::colors::text_dim())),
            Span::styled("Esc", Style::default().fg(crate::colors::error()).add_modifier(Modifier::BOLD)),
            Span::styled(" Close", Style::default().fg(crate::colors::text_dim())),
        ]));

        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .style(Style::default().bg(crate::colors::background()).fg(crate::colors::text()))
            .render(
                Rect {
                    x: inner.x.saturating_add(1),
                    y: inner.y,
                    width: inner.width.saturating_sub(2),
                    height: inner.height,
                },
                buf,
            );
    }
}

