//! The dialog packets: `ClientboundShowDialogPacket`, `ClientboundClearDialogPacket` and the
//! client's `ServerboundCustomClickActionPacket`.

use photon_core::dialog::Dialog;
use photon_core::nbt::FieldValue;

use super::reader::read_varint;
use super::*;

/// What the server did to the dialog on screen.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogEvent {
    Show(Box<Dialog>),
    Clear,
}

/// `Holder<Dialog>` on the wire: a registry entry by number, or the dialog written out.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogHolder {
    Registry(usize),
    Direct(Box<Dialog>),
}

/// `Dialog.STREAM_CODEC` (the play phase): a holder.
pub fn parse_show_dialog(input: &[u8]) -> Result<DialogHolder, ConnectionError> {
    let mut input = input;
    match read_varint(&mut input, "dialog holder")? {
        0 => Ok(DialogHolder::Direct(Box::new(read_dialog(&mut input)?))),
        id => Ok(DialogHolder::Registry(
            usize::try_from(id - 1).map_err(|_| ConnectionError::Codec("dialog holder"))?,
        )),
    }
}

fn read_dialog(input: &mut &[u8]) -> Result<Dialog, ConnectionError> {
    photon_core::nbt::read_any_nbt(input)
        .ok()
        .flatten()
        .and_then(|tag| Dialog::from_nbt(&tag))
        .ok_or(ConnectionError::Codec("dialog"))
}

/// `ServerboundCustomClickActionPacket.STREAM_CODEC`: the identifier, and the payload tag behind
/// a length prefix (an end tag when there is none).
pub fn encode_custom_click_action(
    id: &str,
    payload: Option<&FieldValue>,
    output: &mut Vec<u8>,
) -> Result<(), ConnectionError> {
    let id = photon_core::identifier::Identifier::parse(id)
        .map_err(|_| ConnectionError::Codec("custom click action id"))?;
    photon_core::identifier::write_identifier(&id, output)
        .map_err(|_| ConnectionError::Codec("custom click action id"))?;
    let mut tag = Vec::new();
    photon_core::nbt::write_any_nbt(payload, &mut tag);
    wire::write_varint(
        i32::try_from(tag.len()).map_err(|_| ConnectionError::Codec("custom click payload"))?,
        output,
    );
    output.extend_from_slice(&tag);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_click_action_without_a_payload_ends_in_a_length_prefixed_end_tag() {
        let mut output = Vec::new();
        encode_custom_click_action("a:b", None, &mut output).expect("encode");
        assert_eq!(output, [3, b'a', b':', b'b', 1, 0]);
    }

    #[test]
    fn a_holder_names_an_entry_or_carries_the_dialog() {
        assert_eq!(
            parse_show_dialog(&[3]).expect("registry"),
            DialogHolder::Registry(2)
        );
        assert!(parse_show_dialog(&[0, 0]).is_err());
    }
}

wrapped_packet!(
    /// `ClientboundShowDialogPacket`.
    ShowDialog(DialogHolder) = parse_show_dialog
);

/// `ClientboundClearDialogPacket`.
pub struct ClearDialog;

impl Decode for ClearDialog {
    fn decode(_payload: &[u8]) -> Result<Self, ConnectionError> {
        Ok(Self)
    }
}
