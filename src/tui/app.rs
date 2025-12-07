// ============================================
// TUI App State - FORGE (Team Alpha)
// ============================================

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Normal,
    Input,
    Processing,
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FocusedPanel {
    Chat,
    Files,
    Output,
}

pub struct App {
    /// Current app state
    pub state: AppState,
    
    /// Input buffer for user text
    pub input: String,
    
    /// Cursor position in input
    pub cursor_position: usize,
    
    /// Chat history
    pub messages: Vec<ChatMessage>,
    
    /// Current output/response
    pub output: String,
    
    /// File tree (simplified)
    pub files: Vec<String>,
    
    /// Selected file index
    pub selected_file: usize,
    
    /// Currently focused panel
    pub focused_panel: FocusedPanel,
    
    /// Scroll offset for chat
    pub chat_scroll: u16,
    
    /// Scroll offset for output
    pub output_scroll: u16,
    
    /// Status message
    pub status: String,
    
    /// Is the app running
    pub running: bool,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Input,
            input: String::new(),
            cursor_position: 0,
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: "Welcome to Webrana AI! Type your message and press Enter.".to_string(),
                    timestamp: chrono_lite(),
                },
            ],
            output: String::new(),
            files: vec![
                "src/".to_string(),
                "├── main.rs".to_string(),
                "├── cli/".to_string(),
                "├── core/".to_string(),
                "├── llm/".to_string(),
                "├── skills/".to_string(),
                "├── plugins/".to_string(),
                "└── tui/".to_string(),
                "Cargo.toml".to_string(),
                "Dockerfile".to_string(),
            ],
            selected_file: 0,
            focused_panel: FocusedPanel::Chat,
            chat_scroll: 0,
            output_scroll: 0,
            status: "Ready".to_string(),
            running: true,
        }
    }

    pub fn tick(&mut self) {
        // Called on each tick, can be used for animations
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Return true to quit
        match self.state {
            AppState::Normal => self.handle_normal_mode(key),
            AppState::Input => self.handle_input_mode(key),
            AppState::Processing => false,
            AppState::Help => self.handle_help_mode(key),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('i') => {
                self.state = AppState::Input;
                self.status = "-- INSERT --".to_string();
            }
            KeyCode::Char('?') => {
                self.state = AppState::Help;
                self.status = "Help - press q to close".to_string();
            }
            KeyCode::Tab => {
                self.focused_panel = match self.focused_panel {
                    FocusedPanel::Chat => FocusedPanel::Files,
                    FocusedPanel::Files => FocusedPanel::Output,
                    FocusedPanel::Output => FocusedPanel::Chat,
                };
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match self.focused_panel {
                    FocusedPanel::Files => {
                        if self.selected_file > 0 {
                            self.selected_file -= 1;
                        }
                    }
                    FocusedPanel::Chat => {
                        if self.chat_scroll > 0 {
                            self.chat_scroll -= 1;
                        }
                    }
                    FocusedPanel::Output => {
                        if self.output_scroll > 0 {
                            self.output_scroll -= 1;
                        }
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match self.focused_panel {
                    FocusedPanel::Files => {
                        if self.selected_file < self.files.len().saturating_sub(1) {
                            self.selected_file += 1;
                        }
                    }
                    FocusedPanel::Chat => {
                        self.chat_scroll += 1;
                    }
                    FocusedPanel::Output => {
                        self.output_scroll += 1;
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn handle_input_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::Normal;
                self.status = "Normal mode".to_string();
            }
            KeyCode::Enter => {
                if !self.input.is_empty() {
                    // Add user message
                    self.messages.push(ChatMessage {
                        role: MessageRole::User,
                        content: self.input.clone(),
                        timestamp: chrono_lite(),
                    });
                    
                    // Simulate response (in real app, this would call LLM)
                    self.messages.push(ChatMessage {
                        role: MessageRole::Assistant,
                        content: format!("Processing: {}", self.input),
                        timestamp: chrono_lite(),
                    });
                    
                    self.input.clear();
                    self.cursor_position = 0;
                    self.status = "Message sent".to_string();
                }
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.input.remove(self.cursor_position);
                }
            }
            KeyCode::Delete => {
                if self.cursor_position < self.input.len() {
                    self.input.remove(self.cursor_position);
                }
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.input.len() {
                    self.cursor_position += 1;
                }
            }
            KeyCode::Home => {
                self.cursor_position = 0;
            }
            KeyCode::End => {
                self.cursor_position = self.input.len();
            }
            // Ctrl+C to quit from input mode
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return true;
            }
            _ => {}
        }
        false
    }

    fn handle_help_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state = AppState::Normal;
                self.status = "Normal mode".to_string();
            }
            _ => {}
        }
        false
    }

    pub fn add_assistant_message(&mut self, content: &str) {
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: content.to_string(),
            timestamp: chrono_lite(),
        });
    }

    pub fn set_output(&mut self, output: &str) {
        self.output = output.to_string();
    }

    pub fn set_status(&mut self, status: &str) {
        self.status = status.to_string();
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

// Simple timestamp without chrono dependency
fn chrono_lite() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
