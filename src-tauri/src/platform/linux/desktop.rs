use std::{collections::HashMap, path::Path};

use crate::domain::ErrorCategory;

type Result<T> = std::result::Result<T, ErrorCategory>;

pub(super) struct DesktopEntry {
    pub name: String,
    pub icon: Option<String>,
    pub exec: Option<String>,
    pub try_exec: Option<String>,
    pub dbus: bool,
    pub working_directory: Option<String>,
}

impl DesktopEntry {
    pub fn parse(text: &str, locale: &str) -> Result<Self> {
        let mut values = HashMap::new();
        let mut active = false;
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') {
                active = line == "[Desktop Entry]";
                continue;
            }
            if active {
                let (key, value) = line.split_once('=').ok_or(ErrorCategory::Unsupported)?;
                if values
                    .insert(key.trim().to_owned(), value.trim().to_owned())
                    .is_some()
                {
                    return Err(ErrorCategory::Unsupported);
                }
            }
        }
        let boolean = |key| -> Result<bool> {
            match values.get(key).map(String::as_str) {
                None | Some("false") => Ok(false),
                Some("true") => Ok(true),
                _ => Err(ErrorCategory::Unsupported),
            }
        };
        if values.get("Type").map(String::as_str) != Some("Application")
            || boolean("Hidden")?
            || boolean("NoDisplay")?
            || boolean("Terminal")?
        {
            return Err(ErrorCategory::Unsupported);
        }
        let base_name = values
            .get("Name")
            .filter(|v| !v.is_empty())
            .ok_or(ErrorCategory::Unsupported)?;
        let localized = |key: &str| {
            locale_keys(locale)
                .iter()
                .find_map(|locale| values.get(&format!("{key}[{locale}]")))
                .or_else(|| values.get(key))
                .cloned()
        };
        let name = unescape(&localized("Name").unwrap_or_else(|| base_name.clone()))?;
        if name.is_empty() || name.chars().any(char::is_control) {
            return Err(ErrorCategory::Unsupported);
        }
        Ok(Self {
            name,
            icon: localized("Icon").map(|v| unescape(&v)).transpose()?,
            exec: values.get("Exec").cloned(),
            try_exec: values.get("TryExec").map(|v| unescape(v)).transpose()?,
            dbus: boolean("DBusActivatable")?,
            working_directory: values
                .get("Path")
                .filter(|v| !v.is_empty())
                .map(|v| unescape(v))
                .transpose()?,
        })
    }

    pub fn command(&self, path: &Path) -> Result<Option<Vec<String>>> {
        let Some(exec) = &self.exec else {
            return Ok(None);
        };
        let tokens = tokenize(&unescape(exec)?)?;
        let mut arguments = Vec::new();
        let mut file_codes = 0;
        for (token, quoted) in tokens {
            if quoted && token.contains('%') {
                return Err(ErrorCategory::Unsupported);
            }
            if token == "%i" {
                if let Some(icon) = self.icon.as_ref().filter(|v| !v.is_empty()) {
                    arguments.extend(["--icon".into(), icon.clone()]);
                }
                continue;
            }
            let mut output = String::new();
            let mut chars = token.chars();
            while let Some(ch) = chars.next() {
                if ch != '%' {
                    output.push(ch);
                    continue;
                }
                match chars.next().ok_or(ErrorCategory::Unsupported)? {
                    '%' => output.push('%'),
                    code @ ('f' | 'u' | 'F' | 'U') => {
                        file_codes += 1;
                        if matches!(code, 'F' | 'U') && token.len() != 2 {
                            return Err(ErrorCategory::Unsupported);
                        }
                    }
                    'c' => output.push_str(&self.name),
                    'k' => output.push_str(path.to_str().ok_or(ErrorCategory::Unsupported)?),
                    'd' | 'D' | 'n' | 'N' | 'v' | 'm' => {}
                    _ => return Err(ErrorCategory::Unsupported),
                }
            }
            if !output.is_empty() || token.is_empty() {
                arguments.push(output);
            }
        }
        if file_codes > 1
            || arguments
                .first()
                .is_none_or(|arg| arg.is_empty() || arg.contains('='))
        {
            return Err(ErrorCategory::Unsupported);
        }
        Ok(Some(arguments))
    }
}

fn locale_keys(locale: &str) -> Vec<String> {
    let (base, modifier) = locale
        .split_once('@')
        .map_or((locale, None), |(a, b)| (a, Some(b)));
    let base = base.split('.').next().unwrap_or(base);
    let language = base.split('_').next().unwrap_or(base);
    let mut keys = Vec::new();
    if let Some(modifier) = modifier {
        keys.push(format!("{base}@{modifier}"));
    }
    keys.push(base.into());
    if let Some(modifier) = modifier {
        keys.push(format!("{language}@{modifier}"));
    }
    keys.push(language.into());
    keys
}

fn unescape(value: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }
        result.push(match chars.next().ok_or(ErrorCategory::Unsupported)? {
            's' => ' ',
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '\\' => '\\',
            _ => return Err(ErrorCategory::Unsupported),
        });
    }
    if result.contains('\0') {
        return Err(ErrorCategory::Unsupported);
    }
    Ok(result)
}

fn tokenize(value: &str) -> Result<Vec<(String, bool)>> {
    let mut chars = value.chars().peekable();
    let mut tokens = Vec::new();
    while chars.peek().is_some() {
        while chars.peek().is_some_and(|ch| ch.is_ascii_whitespace()) {
            chars.next();
        }
        if chars.peek().is_none() {
            break;
        }
        let quoted = chars.peek() == Some(&'"');
        let mut token = String::new();
        if quoted {
            chars.next();
        }
        let mut closed = !quoted;
        while let Some(ch) = chars.next() {
            if quoted && ch == '"' {
                closed = true;
                break;
            }
            if !quoted && ch.is_ascii_whitespace() {
                break;
            }
            if quoted && ch == '\\' {
                let escaped = chars.next().ok_or(ErrorCategory::Unsupported)?;
                if !matches!(escaped, '"' | '`' | '$' | '\\') {
                    return Err(ErrorCategory::Unsupported);
                }
                token.push(escaped);
            } else {
                if (!quoted && "\"'\\><~|&;$*?#()`".contains(ch))
                    || (quoted && matches!(ch, '$' | '`'))
                {
                    return Err(ErrorCategory::Unsupported);
                }
                token.push(ch);
            }
        }
        if !closed || (quoted && chars.peek().is_some_and(|ch| !ch.is_ascii_whitespace())) {
            return Err(ErrorCategory::Unsupported);
        }
        tokens.push((token, quoted));
    }
    Ok(tokens)
}
