//! What running a command the player did not type needs first (`ClientPacketListener.verifyCommand`).

use super::{CommandTree, NodeKind, NodeStub, ParseResults, has_signable_arguments};

/// `ClientPacketListener.CommandCheckResult`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCheck {
    /// The command parses for the player and needs nothing more.
    NoIssues,
    /// The text is not a whole command.
    ParseErrors,
    /// A message in it would have to be signed, which only the chat box does.
    SignatureRequired,
    /// It parses only with permissions the player has: a source with none could not run it.
    PermissionsRequired,
}

/// `ClientPacketListener.isValidCommand`.
fn is_valid(parse: &ParseResults) -> bool {
    !parse.can_read() && parse.exceptions.is_empty() && parse.context.last_child().executable
}

impl CommandTree {
    /// A dispatcher with only its root, which the client has before the server sends commands.
    pub fn empty() -> Self {
        let root = NodeStub {
            kind: NodeKind::Root,
            children: Vec::new(),
            redirect: None,
            executable: false,
            custom_suggestions: false,
            restricted: false,
            type_id: 0,
        };
        Self::build(vec![root], 0).expect("a lone root links")
    }

    /// `verifyCommand`: `command` has no slash. `selectors` is whether the player may use entity
    /// selectors; a source without permissions may not.
    pub fn check(&self, command: &str, selectors: bool) -> CommandCheck {
        let with_permissions = self.parse_as(command, 0, (selectors, true));
        if !is_valid(&with_permissions) {
            return CommandCheck::ParseErrors;
        }
        if has_signable_arguments(self, &with_permissions) {
            return CommandCheck::SignatureRequired;
        }
        if is_valid(&self.parse_as(command, 0, (false, false))) {
            CommandCheck::NoIssues
        } else {
            CommandCheck::PermissionsRequired
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tree_without_commands_knows_none() {
        let tree = CommandTree::empty();
        assert_eq!(tree.check("say hi", true), CommandCheck::ParseErrors);
    }
}
