//! What the player may do in the chat (`ChatAbilities`): the restrictions that apply, and the
//! permissions that are left once each has taken its share.

use crate::options::ChatVisibility;

/// `ChatRestriction`: a reason the chat is limited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChatRestriction {
    ChatAndCommandsDisabledByOptions,
    ChatDisabledByOptions,
    DisabledByLauncher,
    DisabledByProfile,
}

/// What a restriction offers to do about itself (`ChatRestriction.Action`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestrictionAction {
    GoToAccount,
    GoToChatSettings,
}

impl RestrictionAction {
    /// The button's text.
    pub const fn title_key(self) -> &'static str {
        match self {
            Self::GoToAccount => "chat_restriction.disabled_by_profile.action",
            Self::GoToChatSettings => "chat_restriction.chat_disabled_by_options.action",
        }
    }
}

impl ChatRestriction {
    /// The text that says why.
    pub const fn display_key(self) -> &'static str {
        match self {
            Self::ChatAndCommandsDisabledByOptions => {
                "chat_restriction.chat_and_commands_disabled_by_options"
            }
            Self::ChatDisabledByOptions => "chat_restriction.chat_disabled_by_options",
            Self::DisabledByLauncher => "chat_restriction.disabled_by_launcher",
            Self::DisabledByProfile => "chat_restriction.disabled_by_profile",
        }
    }

    pub const fn action(self) -> Option<RestrictionAction> {
        match self {
            Self::ChatAndCommandsDisabledByOptions | Self::ChatDisabledByOptions => {
                Some(RestrictionAction::GoToChatSettings)
            }
            Self::DisabledByProfile => Some(RestrictionAction::GoToAccount),
            Self::DisabledByLauncher => None,
        }
    }
}

/// A chat permission (`Permissions.CHAT_PERMISSIONS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatPermission {
    SendMessages,
    SendCommands,
    ReceiveSystemMessages,
    ReceivePlayerMessages,
}

impl ChatPermission {
    /// The order the restrictions screen lists them in.
    pub const ALL: [Self; 4] = [
        Self::SendMessages,
        Self::SendCommands,
        Self::ReceiveSystemMessages,
        Self::ReceivePlayerMessages,
    ];

    /// The permission's id in the screen's texts.
    pub const fn id(self) -> &'static str {
        match self {
            Self::SendMessages => "send_messages",
            Self::SendCommands => "send_commands",
            Self::ReceiveSystemMessages => "receive_system_messages",
            Self::ReceivePlayerMessages => "receive_player_messages",
        }
    }
}

/// Where a chat message came from (`GuiMessageSource`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageSource {
    SystemClient,
    SystemServer,
    Player,
}

/// `ChatAbilities`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChatAbilities {
    /// In the order they were given: the reference keeps them in a set whose order is salted anew
    /// in every run, so none is the right one.
    restrictions: Vec<ChatRestriction>,
}

impl ChatAbilities {
    /// `Minecraft.computeChatAbilities`: the chat option, and on a multiplayer server what the
    /// launcher and the account allow.
    pub fn compute(visibility: ChatVisibility, (launcher, profile): (bool, bool)) -> Self {
        let mut restrictions = Vec::new();
        match visibility {
            ChatVisibility::Hidden => {
                restrictions.push(ChatRestriction::ChatAndCommandsDisabledByOptions);
            }
            ChatVisibility::System => restrictions.push(ChatRestriction::ChatDisabledByOptions),
            ChatVisibility::Full => {}
        }
        if !launcher {
            restrictions.push(ChatRestriction::DisabledByLauncher);
        }
        if !profile {
            restrictions.push(ChatRestriction::DisabledByProfile);
        }
        Self { restrictions }
    }

    /// The abilities these restrictions leave (`ChatAbilities.Builder`).
    pub fn of(restrictions: impl IntoIterator<Item = ChatRestriction>) -> Self {
        let mut all = Vec::new();
        for restriction in restrictions {
            if !all.contains(&restriction) {
                all.push(restriction);
            }
        }
        Self { restrictions: all }
    }

    pub fn has_any_restrictions(&self) -> bool {
        !self.restrictions.is_empty()
    }

    pub fn restrictions(&self) -> &[ChatRestriction] {
        &self.restrictions
    }

    /// Whether a permission is left: each restriction takes its share away
    /// (`ChatRestriction.modifyPermissions`).
    pub fn has(&self, permission: ChatPermission) -> bool {
        use ChatPermission::{ReceivePlayerMessages, ReceiveSystemMessages, SendCommands};
        use ChatRestriction::ChatAndCommandsDisabledByOptions as NoCommands;
        !self.restrictions.iter().any(|restriction| {
            let player_messages = matches!(
                permission,
                ChatPermission::SendMessages | ReceivePlayerMessages
            );
            let commands = matches!(permission, SendCommands | ReceiveSystemMessages);
            player_messages || (commands && *restriction == NoCommands)
        })
    }

    pub fn can_send_messages(&self) -> bool {
        self.has(ChatPermission::SendMessages)
    }

    pub fn can_send_commands(&self) -> bool {
        self.has(ChatPermission::SendCommands)
    }

    pub fn can_receive_player_messages(&self) -> bool {
        self.has(ChatPermission::ReceivePlayerMessages)
    }

    pub fn can_receive_system_messages(&self) -> bool {
        self.has(ChatPermission::ReceiveSystemMessages)
    }

    /// `visibleMessagesFilter`: client messages always show.
    pub fn shows(&self, source: MessageSource) -> bool {
        match source {
            MessageSource::SystemClient => true,
            MessageSource::Player => self.can_receive_player_messages(),
            MessageSource::SystemServer => self.can_receive_system_messages(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_taken_from_an_open_chat() {
        let abilities = ChatAbilities::compute(ChatVisibility::Full, (true, true));
        assert!(!abilities.has_any_restrictions());
        assert!(ChatPermission::ALL.iter().all(|each| abilities.has(*each)));
    }

    #[test]
    fn commands_only_keeps_commands_and_system_messages() {
        let abilities = ChatAbilities::compute(ChatVisibility::System, (true, true));
        assert!(abilities.can_send_commands() && abilities.can_receive_system_messages());
        assert!(!abilities.can_send_messages() && !abilities.can_receive_player_messages());
    }

    #[test]
    fn a_hidden_chat_keeps_nothing() {
        let abilities = ChatAbilities::compute(ChatVisibility::Hidden, (true, true));
        assert!(ChatPermission::ALL.iter().all(|each| !abilities.has(*each)));
        assert!(abilities.shows(MessageSource::SystemClient));
        assert!(!abilities.shows(MessageSource::SystemServer));
    }

    #[test]
    fn the_account_takes_player_messages_only() {
        let abilities = ChatAbilities::compute(ChatVisibility::Full, (true, false));
        assert_eq!(
            abilities.restrictions(),
            [ChatRestriction::DisabledByProfile]
        );
        assert!(abilities.can_send_commands() && !abilities.can_send_messages());
    }
}
