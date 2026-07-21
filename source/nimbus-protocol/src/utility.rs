use regex::Regex;
use serde::{Deserialize, Serialize};

//================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

//================================================================

pub fn serialize<T: Serialize>(value: &T) -> anyhow::Result<Vec<u8>> {
    let mut data = Vec::new();
    ciborium::into_writer(value, &mut data)?;
    Ok(data)
}

pub fn deserialize<T: for<'a> Deserialize<'a>>(value: &[u8]) -> anyhow::Result<T> {
    Ok(ciborium::from_reader(value)?)
}

pub fn set_panic_hook(name: &'static str) {
    let default_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        let mut log = String::new();

        if let Some(s) = info.payload().downcast_ref::<&str>() {
            log.push_str(&format!("Message: {}\n", s));
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            log.push_str(&format!("Message: {}\n", s));
        } else {
            log.push_str("Message: <non-string payload>\n");
        }

        if let Some(location) = info.location() {
            log.push_str(&format!(
                "Location: {}:{}:{}\n",
                location.file(),
                location.line(),
                location.column()
            ));
        }

        let backtrace = std::backtrace::Backtrace::force_capture();
        log.push_str("\nBacktrace:\n");
        log.push_str(&backtrace.to_string());

        let date = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let _ = std::fs::write(format!("panic_{name}_{date}.log"), log);

        default_hook(info);
    }));
}

pub fn get_link_list(text: &str) -> anyhow::Result<Vec<String>> {
    let mut list = Vec::default();
    let pattern = Regex::new(
        r"\b((?:https?:\/\/)?(?:www\.)?[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)+(?:\/[^\s]*)?)\b",
    )?;

    for (_, [link]) in pattern.captures_iter(text).map(|capture| capture.extract()) {
        list.push(link.to_string());
    }

    Ok(list)
}
