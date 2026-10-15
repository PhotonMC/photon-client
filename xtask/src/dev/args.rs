//! A small option parser for the developer commands.

use std::collections::BTreeMap;
use std::str::FromStr;

/// The options a command accepts: flags, and options with a fixed number of values.
pub struct Spec<'a> {
    pub flags: &'a [&'a str],
    pub options: &'a [(&'a str, usize)],
}

#[derive(Default)]
pub struct Parsed {
    flags: Vec<String>,
    options: BTreeMap<String, Vec<Vec<String>>>,
    pub positional: Vec<String>,
}

/// Splits `--name=value` into two arguments.
fn expand(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for arg in args {
        match arg.split_once('=') {
            Some((name, value)) if name.starts_with("--") => {
                out.push(name.to_owned());
                out.push(value.to_owned());
            }
            _ => out.push(arg.clone()),
        }
    }
    out
}

pub fn parse(args: &[String], spec: &Spec) -> Result<Parsed, String> {
    let mut parsed = Parsed::default();
    let expanded = expand(args);
    let mut rest = expanded.iter();
    while let Some(arg) = rest.next() {
        if spec.flags.contains(&arg.as_str()) {
            parsed.flags.push(arg.clone());
        } else if let Some((_, arity)) = spec.options.iter().find(|(name, _)| name == arg) {
            let values: Vec<String> = rest.by_ref().take(*arity).cloned().collect();
            if values.len() != *arity {
                return Err(format!("{arg} needs {arity} value(s)"));
            }
            parsed.options.entry(arg.clone()).or_default().push(values);
        } else if arg.starts_with("--") {
            return Err(format!("unknown option `{arg}`"));
        } else {
            parsed.positional.push(arg.clone());
        }
    }
    Ok(parsed)
}

impl Parsed {
    pub fn flag(&self, name: &str) -> bool {
        self.flags.iter().any(|flag| flag == name)
    }

    /// The first value of the last use of `name`.
    pub fn value(&self, name: &str) -> Option<&str> {
        let last = self.options.get(name)?.last()?;
        last.first().map(String::as_str)
    }

    /// The first value of every use of `name`.
    pub fn values(&self, name: &str) -> Vec<&str> {
        let uses = self.options.get(name).map(Vec::as_slice).unwrap_or(&[]);
        uses.iter()
            .filter_map(|values| values.first())
            .map(String::as_str)
            .collect()
    }

    /// All values of the last use of `name`.
    pub fn group(&self, name: &str) -> Option<&[String]> {
        self.options.get(name)?.last().map(Vec::as_slice)
    }

    /// The value of `name` parsed as a number, or `default`.
    pub fn number<T: FromStr>(&self, name: &str, default: T) -> Result<T, String> {
        match self.value(name) {
            None => Ok(default),
            Some(text) => text
                .parse()
                .map_err(|_| format!("{name}: `{text}` is not a number")),
        }
    }

    /// The first positional argument, or `default`.
    pub fn address(&self, default: &str) -> String {
        self.positional
            .first()
            .cloned()
            .unwrap_or_else(|| default.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: Spec = Spec {
        flags: &["--background"],
        options: &[("--size", 2), ("--rcon", 1), ("--wait", 1)],
    };

    fn parsed(args: &[&str]) -> Result<Parsed, String> {
        parse(
            &args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>(),
            &SPEC,
        )
    }

    #[test]
    fn flags_options_and_positionals_are_separated() {
        let args = parsed(&[
            "name",
            "--background",
            "--size",
            "800",
            "600",
            "--rcon",
            "a",
            "--rcon=b",
            "--wait=2.5",
        ])
        .expect("parses");
        assert!(args.flag("--background"));
        assert_eq!(args.positional, ["name"]);
        assert_eq!(
            args.group("--size"),
            Some(["800".to_owned(), "600".to_owned()].as_slice())
        );
        assert_eq!(args.values("--rcon"), ["a", "b"]);
        assert_eq!(args.number("--wait", 9.0), Ok(2.5));
        assert_eq!(args.number("--missing", 9.0), Ok(9.0));
    }

    #[test]
    fn unknown_options_and_short_groups_are_errors() {
        assert!(parsed(&["--nope"]).is_err());
        assert!(parsed(&["--size", "800"]).is_err());
        assert!(
            parsed(&["--wait", "soon"])
                .expect("parses")
                .number("--wait", 1.0)
                .is_err()
        );
    }
}
