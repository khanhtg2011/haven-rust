use regex::Regex;
use std::collections::HashSet;

/// Parsed slash command
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlashCommand {
    pub cmd: String,
    pub arg: String,
}

lazy_static::lazy_static! {
    static ref SLASH_RE: Regex = Regex::new(r"^/([a-zA-Z]+)(?:\s+([\s\S]*))?$").unwrap();
}

/// "/name the rest of the message" as a command and its text, or None.
/// Preserves exact behavior of Haven Node.js slashParse.js.
pub fn parse_slash(text: &str) -> Option<SlashCommand> {
    let caps = SLASH_RE.captures(text)?;
    let cmd = caps.get(1)?.as_str().to_lowercase();
    let arg = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("").to_string();
    Some(SlashCommand { cmd, arg })
}

pub const REFUSAL: &str = "That name is not allowed on this server";
pub const MAX_WORDS: usize = 300;
pub const MAX_WORD_LEN: usize = 40;

/// Map look-alike characters and leetspeak numbers
fn map_lookalike(c: char, one: char) -> char {
    match c {
        '1' | '|' => one,
        '0' => 'o',
        '3' => 'e',
        '4' => 'a',
        '5' => 's',
        '7' => 't',
        '8' => 'b',
        '@' => 'a',
        '$' => 's',
        '!' => 'i',
        // Cyrillic homoglyphs
        'а' => 'a',
        'в' => 'b',
        'е' => 'e',
        'к' => 'k',
        'м' => 'm',
        'н' => 'h',
        'о' => 'o',
        'р' => 'p',
        'с' => 'c',
        'т' => 't',
        'у' => 'y',
        'х' => 'x',
        'і' => 'i',
        'ј' => 'j',
        'ѕ' => 's',
        _ => c,
    }
}

/// Normalizes string removing accents, non-alphanumeric, and mapping leetspeak/homoglyphs
pub fn reduce_name(text: &str, one: char) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| map_lookalike(c, one))
        .filter(|c| c.is_alphanumeric())
        .collect()
}

/// Parses the admin's blocked names raw text
pub fn parse_blocked_names(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    for part in raw.split(|c| c == '\n' || c == ',') {
        let trimmed: String = part.split_whitespace().collect::<Vec<_>>().join(" ");
        let word: String = trimmed.chars().take(MAX_WORD_LEN).collect();
        let key = reduce_name(&word, 'i');
        if word.is_empty() || key.len() < 2 || seen.contains(&key) {
            continue;
        }
        seen.insert(key);
        out.push(word);
        if out.len() >= MAX_WORDS {
            break;
        }
    }
    out
}

/// Returns the first blocked word `name` contains, or None.
pub fn blocked_word_in<'a>(name: &str, words: &'a [String]) -> Option<&'a str> {
    if words.is_empty() {
        return None;
    }
    let form_i = reduce_name(name, 'i');
    let form_l = reduce_name(name, 'l');

    for word in words {
        let key = reduce_name(word, 'i');
        if !key.is_empty() && (form_i.contains(&key) || form_l.contains(&key)) {
            return Some(word.as_str());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_slash() {
        assert_eq!(
            parse_slash("/ban @user spam"),
            Some(SlashCommand {
                cmd: "ban".into(),
                arg: "@user spam".into()
            })
        );
        assert_eq!(
            parse_slash("/HELP"),
            Some(SlashCommand {
                cmd: "help".into(),
                arg: "".into()
            })
        );
        assert_eq!(parse_slash("/usr/bin/python"), None);
        assert_eq!(parse_slash("hello /test"), None);
    }

    #[test]
    fn test_blocked_names() {
        let words = parse_blocked_names("badword\nanotherbad");
        assert_eq!(blocked_word_in("B4d_W0rd", &words), Some("badword"));
        assert_eq!(blocked_word_in("b!d word", &words), None);
        assert_eq!(blocked_word_in("good_user", &words), None);
    }
}
