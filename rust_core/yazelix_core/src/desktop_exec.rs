use crate::bridge::CoreError;

pub(crate) fn parse_env_assignment(token: &str) -> Option<(&str, &str)> {
    let (key, value) = token.split_once('=')?;
    let mut chars = key.chars();
    let first = chars.next()?;
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return None;
    }
    if !chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric()) {
        return None;
    }
    Some((key, value))
}

pub(crate) fn split_desktop_exec_tokens(exec: &str) -> Result<Vec<String>, CoreError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut token_started = false;
    let mut in_quotes = false;
    let mut chars = exec.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            match ch {
                '"' => in_quotes = false,
                '\\' => current.push(chars.next().unwrap_or('\\')),
                other => current.push(other),
            }
            token_started = true;
            continue;
        }

        match ch {
            '"' => {
                in_quotes = true;
                token_started = true;
            }
            '\\' => {
                current.push(chars.next().unwrap_or('\\'));
                token_started = true;
            }
            other if other.is_whitespace() => {
                if token_started {
                    tokens.push(std::mem::take(&mut current));
                    token_started = false;
                }
            }
            other => {
                current.push(other);
                token_started = true;
            }
        }
    }

    if in_quotes {
        return Err(CoreError::usage(format!(
            "Unterminated quoted string in desktop Exec= command: {exec}"
        )));
    }
    if token_started {
        tokens.push(current);
    }
    Ok(tokens)
}
