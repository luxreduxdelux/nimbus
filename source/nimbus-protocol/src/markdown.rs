use std::{iter::Enumerate, str::Chars};

//================================================================

#[derive(Debug, Clone)]
pub enum Token {
    /// Plain text. (hello, world)
    Text(String),
    /// Italic text. (*hello, world*)
    Italic(String),
    /// Bold text. (**hello, world**)
    Bold(String),
    /// Bold and italic text. (***hello, world***)
    BoldItalic(String),
    /// Account text. (@luxreduxdelux)
    Account(String),
    /// Channel text. (#general)
    Channel(String),
    /// Emote text. (:smiley:)
    Emote(String),
    /// Hidden text. (||it's a secret||)
    Hidden(String),
    /// Header (type A) text. (# hello, world)
    HeaderA(String),
    /// Header (type B) text. (## hello, world)
    HeaderB(String),
    /// Header (type C) text. (### hello, world)
    HeaderC(String),
    /// Sub-text. (-# hello, world)
    SubText(String),
    /// Link text. ([a link to somewhere](www.google.com))
    Link(String, String),
    /// Code (type A) text. (`fn foo() {}`)
    CodeA(String),
    /// Code (type B) text. (```fn foo() {}```)
    CodeB(String),
    /// Quote text. (> hello, world)
    Quote(String),
}

impl Token {
    pub fn kind(&self) -> TokenKind {
        match self {
            Token::Text(_) => TokenKind::Text,
            Token::Italic(_) => TokenKind::Italic,
            Token::Bold(_) => TokenKind::Bold,
            Token::BoldItalic(_) => TokenKind::BoldItalic,
            Token::Account(_) => TokenKind::Account,
            Token::Channel(_) => TokenKind::Channel,
            Token::Emote(_) => TokenKind::Emote,
            Token::Hidden(_) => TokenKind::Hidden,
            Token::HeaderA(_) => TokenKind::HeaderA,
            Token::HeaderB(_) => TokenKind::HeaderB,
            Token::HeaderC(_) => TokenKind::HeaderC,
            Token::SubText(_) => TokenKind::SubText,
            Token::Link(_, _) => TokenKind::Link,
            Token::CodeA(_) => TokenKind::CodeA,
            Token::CodeB(_) => TokenKind::CodeB,
            Token::Quote(_) => TokenKind::Quote,
        }
    }
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
    Hidden,
    HeaderA,
    HeaderB,
    HeaderC,
    SubText,
    Link,
    CodeA,
    CodeB,
    Quote,
}

impl Token {
    pub fn parse(text: &str) -> (Vec<Token>, TokenKind, usize) {
        let mut which = TokenKind::Text;
        let mut index = 0;
        let mut result = Vec::new();
        let mut buffer = Buffer::default();
        let mut string = text.chars().enumerate();

        while let Some((i, character)) = string.next() {
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
                        index = i;

                        buffer.push(character);
                        match which_local {
                            TokenKind::Bold => {
                                buffer.push(string.next().unwrap().1);
                            }
                            TokenKind::BoldItalic => {
                                buffer.push(string.next().unwrap().1);
                                buffer.push(string.next().unwrap().1);
                            }
                            _ => {}
                        }

                        result.push(Self::from_string(buffer.clear()))
                    } else {
                        which = which_local;
                        index = i;

                        if !buffer.is_empty() {
                            result.push(Self::from_string(buffer.clear()));
                        }

                        buffer.push(character);
                        match which_local {
                            TokenKind::Bold => {
                                buffer.push(string.next().unwrap().1);
                            }
                            TokenKind::BoldItalic => {
                                buffer.push(string.next().unwrap().1);
                                buffer.push(string.next().unwrap().1);
                            }
                            _ => {}
                        }
                    }
                }
                '@' => {
                    if !buffer.is_empty() {
                        result.push(Self::from_string(buffer.clear()));
                    }

                    which = TokenKind::Account;
                    index = i;
                    buffer.push(character);
                }
                '#' => {
                    if !buffer.is_empty() {
                        result.push(Self::from_string(buffer.clear()));
                    }

                    which = TokenKind::Channel;
                    index = i;
                    buffer.push(character);
                }
                ':' => {
                    if which == TokenKind::Emote {
                        which = TokenKind::Text;
                        index = i;

                        buffer.push(character);
                        result.push(Self::from_string(buffer.clear()))
                    } else {
                        which = TokenKind::Emote;
                        index = i;

                        if !buffer.is_empty() {
                            result.push(Self::from_string(buffer.clear()));
                        }

                        buffer.push(character);
                    }
                }
                '|' => {
                    let next = Self::peek(string.clone(), 0, '|');

                    if next {
                        if which == TokenKind::Hidden {
                            which = TokenKind::Text;
                            index = i;

                            buffer.push(character);
                            buffer.push(string.next().unwrap().1);

                            result.push(Self::from_string(buffer.clear()))
                        } else {
                            which = TokenKind::Hidden;
                            index = i;

                            if !buffer.is_empty() {
                                result.push(Self::from_string(buffer.clear()));
                            }

                            buffer.push(character);
                            buffer.push(string.next().unwrap().1);
                        }
                    } else {
                        buffer.push(character);
                    }
                }
                _ => {
                    //
                    match which {
                        TokenKind::Account | TokenKind::Channel => {
                            if !character.is_alphabetic() {
                                result.push(Self::from_string(buffer.clear()));
                                which = TokenKind::Text;
                                index = i;
                                buffer.push(character);
                            } else {
                                buffer.push(character);
                            }
                        }
                        TokenKind::Emote => {
                            if !character.is_alphabetic() {
                                which = TokenKind::Text;
                                index = i;
                            }

                            buffer.push(character);
                        }
                        _ => {
                            buffer.push(character);
                        }
                    }
                }
            }
        }

        if !buffer.is_empty() {
            result.push(Self::from_string(buffer.clear()));
        }

        (result, which, index)
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
            } else if Self::start_end(&text, "||") {
                Self::Hidden(text[2..text.len() - 2].to_string())
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

    fn peek(mut string: Enumerate<Chars<'_>>, index: usize, character: char) -> bool {
        if let Some((_, c)) = string.nth(index)
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
