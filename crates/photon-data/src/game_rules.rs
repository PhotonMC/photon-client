//! The game rules the in-world rule screen lists (`GameRules`, the built-in registry).

use crate::tables::game_rules::GAME_RULES;

/// One rule: an on/off switch, or an integer when `bounds` is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameRule {
    pub name: &'static str,
    /// The category's name (`GameRuleCategory`), without a namespace.
    pub category: &'static str,
    /// The default as the server writes it.
    pub default: &'static str,
    /// The lowest and highest an integer takes.
    pub bounds: Option<(i32, i32)>,
    /// Whether the vanilla feature set turns the rule on.
    pub vanilla: bool,
}

/// Every rule, in registry order.
pub fn all() -> &'static [GameRule] {
    &GAME_RULES
}

/// The rule with this name.
pub fn rule(name: &str) -> Option<&'static GameRule> {
    GAME_RULES.iter().find(|rule| rule.name == name)
}
