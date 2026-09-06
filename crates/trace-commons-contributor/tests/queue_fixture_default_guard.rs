//! QueueEntry's public Default exists for external test fixtures, not live offers.
//! This lexical guard covers explicit constructors, typed defaults and struct updates,
//! including simple aliases. It is not Rust type/data-flow resolution: macro-generated
//! or indirectly inferred defaults still require review. Production updates from an
//! existing entry (`..old`) remain valid.
use std::path::Path;

fn tokens(source: &str) -> Vec<String> {
    let chars: Vec<char> = source.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        if chars[i..].starts_with(&['/', '/']) {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if chars[i..].starts_with(&['/', '*']) {
            i += 2;
            let mut depth = 1;
            while depth > 0 {
                assert!(i < chars.len(), "unterminated block comment");
                if chars[i..].starts_with(&['/', '*']) {
                    depth += 1;
                    i += 2;
                } else if chars[i..].starts_with(&['*', '/']) {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        // Raw strings can contain arbitrary braces, quotes and apparent attributes.
        let raw = if chars[i..].starts_with(&['b', 'r']) {
            i + 1
        } else {
            i
        };
        if chars[raw] == 'r' {
            let mut quote = raw + 1;
            while chars.get(quote) == Some(&'#') {
                quote += 1;
            }
            if chars.get(quote) == Some(&'"') {
                let hashes = quote - raw - 1;
                i = quote + 1;
                loop {
                    assert!(i < chars.len(), "unterminated raw string");
                    if chars[i] == '"' && (0..hashes).all(|n| chars.get(i + 1 + n) == Some(&'#')) {
                        i += 1 + hashes;
                        break;
                    }
                    i += 1;
                }
                out.push("literal".into());
                continue;
            }
        }
        // Preserve lifetimes as tokens, while dropping quoted character literals.
        let character = chars[i] == '\''
            && (chars.get(i + 2) == Some(&'\'') || chars.get(i + 1) == Some(&'\\'));
        if chars[i] == '"' || character {
            let quote = chars[i];
            i += 1;
            loop {
                assert!(i < chars.len(), "unterminated literal");
                if chars[i] == '\\' {
                    i += 2;
                } else if chars[i] == quote {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            out.push("literal".into());
            continue;
        }
        if chars[i].is_alphanumeric() || chars[i] == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            out.push(chars[start..i].iter().collect());
        } else {
            out.push(chars[i].to_string());
            i += 1;
        }
    }
    out
}

fn group_end(tokens: &[String], start: usize) -> usize {
    let close = match tokens[start].as_str() {
        "{" => "}",
        "[" => "]",
        "(" => ")",
        _ => panic!("group expected"),
    };
    let mut i = start + 1;
    while i < tokens.len() {
        if tokens[i] == close {
            return i + 1;
        }
        if ["{", "[", "("].contains(&tokens[i].as_str()) {
            i = group_end(tokens, i);
        } else {
            i += 1;
        }
    }
    panic!("unbalanced source group")
}

fn production_tokens(source: &str) -> Vec<String> {
    let tokens = tokens(source);
    let mut out = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        // Only an unconditional cfg(test) exemption. cfg(any(test, feature=...))
        // may compile in production and must not hide a constructor from this guard.
        if tokens[i..].starts_with(&["#", "[", "cfg", "(", "test", ")", "]"].map(String::from)) {
            i += 7;
            while i < tokens.len() {
                match tokens[i].as_str() {
                    "{" => {
                        i = group_end(&tokens, i);
                        break;
                    }
                    ";" => {
                        i += 1;
                        break;
                    }
                    "[" | "(" => i = group_end(&tokens, i),
                    _ => i += 1,
                }
            }
        } else {
            out.push(tokens[i].clone());
            i += 1;
        }
    }
    out
}

fn uses_fixture_default_in_production(source: &str) -> bool {
    let t = production_tokens(source);
    let mut names = std::collections::BTreeSet::from(["QueueEntry".to_string()]);
    // Resolve straightforward `use ...QueueEntry as Entry` / `type Entry = QueueEntry`.
    loop {
        let before = names.len();
        for i in 0..t.len() {
            if names.contains(&t[i]) {
                if t.get(i + 1).is_some_and(|s| s == "as") {
                    if let Some(alias) = t.get(i + 2).filter(|s| s.as_str() != "Default") {
                        names.insert(alias.clone());
                    }
                }
                if i >= 3 && t[i - 1] == "=" && t[i - 3] == "type" {
                    names.insert(t[i - 2].clone());
                }
            }
        }
        if names.len() == before {
            break;
        }
    }
    for i in 0..t.len() {
        if !names.contains(&t[i]) {
            continue;
        }
        let rest = &t[i + 1..];
        if rest.starts_with(&[":", ":", "default", "("].map(String::from)) {
            return true;
        }
        if rest.starts_with(&["as", "Default", ">", ":", ":", "default"].map(String::from)) {
            return true;
        }
        if rest.first().is_some_and(|s| s == "=") {
            let end = rest.iter().position(|s| s == ";").unwrap_or(rest.len());
            if rest[..end]
                .windows(4)
                .any(|w| w == ["Default", ":", ":", "default"])
            {
                return true;
            }
        }
        if rest.first().is_some_and(|s| s == "{") {
            let end = group_end(&t, i + 1);
            let mut j = i + 2;
            while j + 1 < end {
                if t[j] == "." && t[j + 1] == "." {
                    if t[j + 2..end].iter().any(|s| s == "default") {
                        return true;
                    }
                }
                if ["{", "[", "("].contains(&t[j].as_str()) {
                    j = group_end(&t, j);
                } else {
                    j += 1;
                }
            }
        }
    }
    false
}

#[test]
fn the_guard_rejects_production_defaults_without_confusing_test_scopes_or_literals() {
    for source in [
        "fn live() { let e = QueueEntry { state: pending(), ..Default::default() }; }",
        "fn live() { let e = QueueEntry::default(); }",
        "fn live() { let e: QueueEntry = Default::default(); }",
        "fn live() { let e = <QueueEntry as Default>::default(); }",
        "use crate::queue::QueueEntry as Entry; fn live() { let e = Entry::default(); }",
        "type Entry = QueueEntry; fn live() { let e = Entry { ..Default::default() }; }",
        "#[cfg(test)] mod tests { fn fixture() {} } fn live() { let e = QueueEntry::default(); }",
        "#[cfg(any(test, feature = \"live\"))] fn live() { let e = QueueEntry::default(); }",
    ] {
        assert!(
            uses_fixture_default_in_production(source),
            "guard missed: {source}"
        );
    }
    for source in [
        "#[cfg(test)] mod tests { fn fixture() { let e = QueueEntry { ..Default::default() }; } }",
        "#[cfg(test)] #[tokio::test] async fn fixture() { let e = QueueEntry::default(); }",
        "fn live() { let e = QueueEntry { state: pending(), ..old }; }",
        "// QueueEntry::default()\nfn live() {}",
        "/* outer /* QueueEntry::default() */ comment */ fn live() {}",
        "fn live() { let s = r###\"#[cfg(test)] { QueueEntry::default() }\"###; let c = '}'; }",
    ] {
        assert!(
            !uses_fixture_default_in_production(source),
            "guard rejected: {source}"
        );
    }
}

#[test]
fn production_queue_entries_do_not_use_fixture_defaults() {
    fn visit(path: &Path) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let source = std::fs::read_to_string(&path).unwrap();
                assert!(
                    !uses_fixture_default_in_production(&source),
                    "QueueEntry fixture Default used outside cfg(test): {}",
                    path.display()
                );
            }
        }
    }
    visit(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"));
}
