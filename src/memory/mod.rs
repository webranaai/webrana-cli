use crate::llm::Message;

#[derive(Debug, Default)]
pub struct Context {
    messages: Vec<Message>,
    max_messages: usize,
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            messages: self.messages.clone(),
            max_messages: self.max_messages,
        }
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_messages: 50,
        }
    }

    pub fn with_max_messages(max: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_messages: max,
        }
    }

    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push(Message::user(content));
        self.trim();
    }

    pub fn add_assistant_message(&mut self, content: &str) {
        self.messages.push(Message::assistant(content));
        self.trim();
    }

    pub fn get_messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    fn trim(&mut self) {
        while self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}
