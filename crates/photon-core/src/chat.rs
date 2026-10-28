//! How the client words a chat message: a chat type's decoration
//! (`ChatTypeDecoration`) puts the sender, the target and the content into a
//! translation, in the type's style.

use crate::text::{Component, Style};

/// What a decoration's translation takes as an argument
/// (`ChatTypeDecoration.Parameter`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatParameter {
    Sender,
    Target,
    Content,
}

impl ChatParameter {
    /// The parameter a data pack names, or the network's ordinal.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "sender" => Some(Self::Sender),
            "target" => Some(Self::Target),
            "content" => Some(Self::Content),
            _ => None,
        }
    }

    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::Sender),
            1 => Some(Self::Target),
            2 => Some(Self::Content),
            _ => None,
        }
    }
}

/// `ChatTypeDecoration`: a translation key, the parameters it takes in order,
/// and the style its text gets.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChatDecoration {
    pub translation_key: String,
    pub parameters: Vec<ChatParameter>,
    pub style: Style,
}

impl ChatDecoration {
    /// `ChatTypeDecoration.decorate`: the content in the decoration's words.
    /// The target is empty when the message has none.
    pub fn decorate(
        &self,
        content: Component,
        sender: &Component,
        target: Option<&Component>,
    ) -> Component {
        let args = self
            .parameters
            .iter()
            .map(|parameter| match parameter {
                ChatParameter::Sender => sender.clone(),
                ChatParameter::Target => target.cloned().unwrap_or_else(Component::empty),
                ChatParameter::Content => content.clone(),
            })
            .collect();
        Component::translatable_with(self.translation_key.clone(), args).with_style(&self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::Lang;

    #[test]
    fn plain_chat_reads_name_then_content() {
        let decoration = ChatDecoration {
            translation_key: "chat.type.text".to_owned(),
            parameters: vec![ChatParameter::Sender, ChatParameter::Content],
            style: Style::default(),
        };
        let lang = Lang::new([("chat.type.text".to_owned(), "<%s> %s".to_owned())]);
        let message =
            decoration.decorate(Component::literal("hi"), &Component::literal("Miner"), None);
        assert_eq!(lang.plain(&message), "<Miner> hi");
    }

    #[test]
    fn a_whisper_takes_its_style() {
        let decoration = ChatDecoration {
            translation_key: "commands.message.display.incoming".to_owned(),
            parameters: vec![ChatParameter::Sender, ChatParameter::Content],
            style: Style {
                colour: Some(0xAAAAAA),
                italic: Some(true),
                ..Style::default()
            },
        };
        let message = decoration.decorate(Component::literal("x"), &Component::literal("A"), None);
        assert_eq!(message.style.italic, Some(true));
    }
}

/// `ChatVisiblity`: which chat the client shows, and tells the server it
/// wants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

impl ChatVisibility {
    pub const fn id(self) -> i32 {
        match self {
            Self::Full => 0,
            Self::System => 1,
            Self::Hidden => 2,
        }
    }

    pub const fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::Full),
            1 => Some(Self::System),
            2 => Some(Self::Hidden),
            _ => None,
        }
    }
}
