use std::str::Chars;

//================================================================

#[derive(Debug, Clone)]
pub enum Token {
    Text(String),
    Italic(String),
    Bold(String),
    BoldItalic(String),
    Account(String),
    Channel(String),
    Emote(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Text,
    Italic,
    Bold,
    BoldItalic,
    Account,
    Channel,
    Emote,
}

impl Token {
    pub fn parse(text: &str) -> Vec<Token> {
        let mut which = TokenKind::Text;
        let mut result = Vec::new();
        let mut buffer = Buffer::default();
        let mut string = text.chars();

        while let Some(character) = string.next() {
            match character {
                '*' => {
                    let bold = Self::peek(string.clone(), 0, '*');
                    let bold_italic = Self::peek(string.clone(), 1, '*');
                    let which_local = if bold_italic && bold {
                        TokenKind::BoldItalic
                    } else if bold {
                        TokenKind::Bold
                    } else {
                        TokenKind::Italic
                    };

                    if which_local == which {
                        which = TokenKind::Text;

                        buffer.push(character);
                        match which_local {
                            TokenKind::Bold => {
                                buffer.push(string.next().unwrap());
                            }
                            TokenKind::BoldItalic => {
                                buffer.push(string.next().unwrap());
                                buffer.push(string.next().unwrap());
                            }
                            _ => {}
                        }

                        result.push(Self::from_string(buffer.clear()))
                    } else {
                        which = which_local;

                        if !buffer.is_empty() {
                            result.push(Self::from_string(buffer.clear()));
                        }

                        buffer.push(character);
                        match which_local {
                            TokenKind::Bold => {
                                buffer.push(string.next().unwrap());
                            }
                            TokenKind::BoldItalic => {
                                buffer.push(string.next().unwrap());
                                buffer.push(string.next().unwrap());
                            }
                            _ => {}
                        }
                    }
                }
                _ => buffer.push(character),
            }
        }

        if !buffer.is_empty() {
            result.push(Self::from_string(buffer.clear()));
        }

        result
    }

    pub fn from_string(text: String) -> Token {
        if text.is_empty() || text.len() == 1 {
            Self::Text(text)
        } else {
            if Self::start_end(&text, "***") {
                Self::BoldItalic(text[3..text.len() - 3].to_string())
            } else if Self::start_end(&text, "**") {
                Self::Bold(text[2..text.len() - 2].to_string())
            } else if Self::start_end(&text, "*") {
                Self::Italic(text[1..text.len() - 1].to_string())
            } else if text.starts_with("@") {
                Self::Account(text)
            } else if text.starts_with("#") {
                Self::Channel(text)
            } else if Self::start_end(&text, ":") {
                Self::Emote(text[1..text.len() - 1].to_string())
            } else {
                Self::Text(text)
            }
        }
    }

    fn start_end(text: &str, character: &str) -> bool {
        if text.len() > character.len() * 2 {
            text.starts_with(character) && text.ends_with(character)
        } else {
            false
        }
    }

    fn peek(mut string: Chars<'_>, index: usize, character: char) -> bool {
        if let Some(c) = string.nth(index)
            && c == character
        {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Buffer {
    data: String,
}

impl Buffer {
    fn push(&mut self, character: char) {
        self.data.push(character);
    }

    fn clear(&mut self) -> String {
        let data = self.data.clone();
        self.data.clear();

        data
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
