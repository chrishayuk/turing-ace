//! Just enough JSON for the language server: a value, a reader and a writer.

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> &Json {
        match self {
            Json::Obj(fields) => fields.iter().find(|f| f.0 == key).map(|f| &f.1).unwrap_or(&Json::Null),
            _ => &Json::Null,
        }
    }
    pub fn str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn num(&self) -> Option<usize> {
        match self {
            Json::Num(n) if *n >= 0.0 => Some(*n as usize),
            _ => None,
        }
    }
    pub fn items(&self) -> &[Json] {
        match self {
            Json::Arr(items) => items,
            _ => &[],
        }
    }

    pub fn write(&self, out: &mut String) {
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Num(n) if n.fract() == 0.0 && n.abs() < 9e15 => out.push_str(&(*n as i64).to_string()),
            Json::Num(n) => out.push_str(&n.to_string()),
            Json::Str(s) => {
                out.push('"');
                for c in s.chars() {
                    match c {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        '\t' => out.push_str("\\t"),
                        c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                        c => out.push(c),
                    }
                }
                out.push('"');
            }
            Json::Arr(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    item.write(out);
                }
                out.push(']');
            }
            Json::Obj(fields) => {
                out.push('{');
                for (i, (key, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    Json::Str(key.clone()).write(out);
                    out.push(':');
                    value.write(out);
                }
                out.push('}');
            }
        }
    }
}

/// `obj([("a", 1.into()), ...])`
pub fn obj<const N: usize>(fields: [(&str, Json); N]) -> Json {
    Json::Obj(fields.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}
impl From<&str> for Json {
    fn from(s: &str) -> Json {
        Json::Str(s.to_string())
    }
}
impl From<String> for Json {
    fn from(s: String) -> Json {
        Json::Str(s)
    }
}
impl From<usize> for Json {
    fn from(n: usize) -> Json {
        Json::Num(n as f64)
    }
}
impl From<bool> for Json {
    fn from(b: bool) -> Json {
        Json::Bool(b)
    }
}
impl From<Vec<Json>> for Json {
    fn from(items: Vec<Json>) -> Json {
        Json::Arr(items)
    }
}

struct Reader<'a> {
    cs: std::iter::Peekable<std::str::Chars<'a>>,
}

pub fn parse(text: &str) -> Result<Json, String> {
    let mut r = Reader { cs: text.chars().peekable() };
    let v = r.value()?;
    r.space();
    match r.cs.next() {
        None => Ok(v),
        Some(c) => Err(format!("unexpected `{}` after the value", c)),
    }
}

impl Reader<'_> {
    fn space(&mut self) {
        while self.cs.peek().is_some_and(|c| c.is_whitespace()) {
            self.cs.next();
        }
    }
    fn word(&mut self, word: &str, v: Json) -> Result<Json, String> {
        for want in word.chars() {
            if self.cs.next() != Some(want) {
                return Err(format!("expected `{}`", word));
            }
        }
        Ok(v)
    }
    fn hex4(&mut self) -> Result<u32, String> {
        let mut v = 0;
        for _ in 0..4 {
            let d = self.cs.next().and_then(|c| c.to_digit(16)).ok_or("bad \\u escape")?;
            v = v * 16 + d;
        }
        Ok(v)
    }
    fn string(&mut self) -> Result<String, String> {
        let mut s = String::new();
        loop {
            match self.cs.next().ok_or("unfinished string")? {
                '"' => return Ok(s),
                '\\' => match self.cs.next().ok_or("unfinished escape")? {
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    'b' => s.push('\u{8}'),
                    'f' => s.push('\u{c}'),
                    'u' => {
                        let mut v = self.hex4()?;
                        if (0xD800..0xDC00).contains(&v) {
                            if self.cs.next() != Some('\\') || self.cs.next() != Some('u') {
                                return Err("unpaired surrogate".into());
                            }
                            let low = self.hex4()?;
                            v = 0x10000 + ((v - 0xD800) << 10) + (low.wrapping_sub(0xDC00) & 0x3FF);
                        }
                        s.push(char::from_u32(v).unwrap_or('\u{FFFD}'));
                    }
                    c => s.push(c),
                },
                c => s.push(c),
            }
        }
    }
    fn value(&mut self) -> Result<Json, String> {
        self.space();
        match *self.cs.peek().ok_or("unexpected end")? {
            'n' => self.word("null", Json::Null),
            't' => self.word("true", Json::Bool(true)),
            'f' => self.word("false", Json::Bool(false)),
            '"' => {
                self.cs.next();
                Ok(Json::Str(self.string()?))
            }
            '[' => {
                self.cs.next();
                let mut items = Vec::new();
                loop {
                    self.space();
                    if self.cs.peek() == Some(&']') {
                        self.cs.next();
                        return Ok(Json::Arr(items));
                    }
                    if !items.is_empty() && self.cs.next() != Some(',') {
                        return Err("expected `,` or `]`".into());
                    }
                    items.push(self.value()?);
                }
            }
            '{' => {
                self.cs.next();
                let mut fields = Vec::new();
                loop {
                    self.space();
                    if self.cs.peek() == Some(&'}') {
                        self.cs.next();
                        return Ok(Json::Obj(fields));
                    }
                    if !fields.is_empty() {
                        if self.cs.next() != Some(',') {
                            return Err("expected `,` or `}`".into());
                        }
                        self.space();
                    }
                    if self.cs.next() != Some('"') {
                        return Err("expected a field name".into());
                    }
                    let key = self.string()?;
                    self.space();
                    if self.cs.next() != Some(':') {
                        return Err("expected `:`".into());
                    }
                    fields.push((key, self.value()?));
                }
            }
            _ => {
                let mut text = String::new();
                while self.cs.peek().is_some_and(|c| c.is_ascii_digit() || "+-.eE".contains(*c)) {
                    text.push(self.cs.next().unwrap());
                }
                text.parse::<f64>().map(Json::Num).map_err(|_| format!("bad number `{}`", text))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_what_it_writes() {
        let text = r#"{"a":[1,2.5,true,null],"b":{"c":"π → \"x\"\n"},"d":"😀→"}"#;
        let v = parse(text).unwrap();
        assert_eq!(v.get("a").items().len(), 4);
        assert_eq!(v.get("b").get("c").str(), Some("π → \"x\"\n"));
        assert_eq!(v.get("d").str(), Some("😀→"));
        let mut out = String::new();
        v.write(&mut out);
        assert_eq!(parse(&out).unwrap(), v);
    }
}
