//! `StringTemplate` and `ParsedTemplate`: a command with `$(name)` holes the dialog's inputs fill.

use std::collections::HashMap;

/// A template: the text between the holes, and the hole names.
#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    segments: Vec<String>,
    variables: Vec<String>,
}

/// `StringTemplate.isValidVariableName`.
pub fn is_valid_variable_name(name: &str) -> bool {
    name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

impl Template {
    /// `StringTemplate.fromString`: none when a hole is open, badly named, or there are none.
    pub fn parse(input: &str) -> Option<Self> {
        let mut segments = Vec::new();
        let mut variables = Vec::new();
        let mut start = 0;
        let mut index = input.find('$');
        while let Some(at) = index {
            if input[at + 1..].starts_with('(') {
                segments.push(input[start..at].to_owned());
                let end = at + 1 + input[at + 1..].find(')')?;
                let variable = &input[at + 2..end];
                if !is_valid_variable_name(variable) {
                    return None;
                }
                variables.push(variable.to_owned());
                start = end + 1;
                index = input[start..].find('$').map(|found| start + found);
            } else {
                index = input[at + 1..].find('$').map(|found| at + 1 + found);
            }
        }
        if start == 0 {
            return None;
        }
        if start != input.len() {
            segments.push(input[start..].to_owned());
        }
        Some(Self {
            segments,
            variables,
        })
    }

    /// `ParsedTemplate.instantiate`: each hole takes its input's text, or nothing when the dialog
    /// has no such input.
    pub fn instantiate(&self, arguments: &HashMap<&str, &str>) -> String {
        let mut out = String::new();
        for (segment, variable) in self.segments.iter().zip(&self.variables) {
            out.push_str(segment);
            out.push_str(arguments.get(variable.as_str()).copied().unwrap_or(""));
        }
        if self.segments.len() > self.variables.len() {
            out.push_str(&self.segments[self.segments.len() - 1]);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments<'a>(pairs: &[(&'a str, &'a str)]) -> HashMap<&'a str, &'a str> {
        pairs.iter().copied().collect()
    }

    #[test]
    fn a_template_needs_a_variable_and_a_closed_valid_one() {
        assert_eq!(Template::parse("say hello"), None);
        assert_eq!(Template::parse("say $(name"), None);
        assert_eq!(Template::parse("say $(bad-name)"), None);
        assert!(Template::parse("say $(name)").is_some());
        assert!(Template::parse("$(name)").is_some());
    }

    #[test]
    fn the_holes_take_their_values_and_the_text_around_them_stays() {
        let template = Template::parse("a $(x) b $(y)").unwrap();
        assert_eq!(
            template.instantiate(&arguments(&[("x", "1"), ("y", "2")])),
            "a 1 b 2"
        );
        let trailing = Template::parse("$(x) tail").unwrap();
        assert_eq!(trailing.instantiate(&arguments(&[("x", "v")])), "v tail");
    }

    #[test]
    fn a_dollar_without_a_bracket_is_text_and_a_missing_value_is_empty() {
        let template = Template::parse("cost $5 $(x)!").unwrap();
        assert_eq!(template.instantiate(&arguments(&[])), "cost $5 !");
    }
}
