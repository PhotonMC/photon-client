use super::*;

/// The patch of a written book of these pages (plain text): one component, `written_book_content`.
pub(super) fn written_book_patch(pages: &[Value]) -> Vec<u8> {
    use voxel_core::wire::write_varint;
    let string = |out: &mut Vec<u8>, text: &str| {
        write_varint(text.len() as i32, out);
        out.extend_from_slice(text.as_bytes());
    };
    // One added component, none removed.
    let mut out = vec![1, 0];
    write_varint(
        voxel_data::item::component_types::WRITTEN_BOOK_CONTENT,
        &mut out,
    );
    string(&mut out, "Title");
    out.push(0);
    string(&mut out, "Author");
    write_varint(0, &mut out);
    write_varint(pages.len() as i32, &mut out);
    for page in pages {
        // A plain text component is a string tag.
        let text = page.as_str().unwrap_or_default();
        out.push(8);
        out.extend_from_slice(&(text.len() as u16).to_be_bytes());
        out.extend_from_slice(text.as_bytes());
        out.push(0);
    }
    out.push(1);
    out
}

pub(super) fn menu_key(step: &Value) -> Result<MenuKey, String> {
    use voxel_render::gui::widgets::slider::SliderKey;
    Ok(match step["key"].as_str().unwrap_or_default() {
        "tab" => MenuKey::Tab(true),
        "shift+tab" => MenuKey::Tab(false),
        "left" => MenuKey::Key(SliderKey::Left),
        "right" => MenuKey::Key(SliderKey::Right),
        "up" => MenuKey::Key(SliderKey::Up),
        "down" => MenuKey::Key(SliderKey::Down),
        "enter" | "space" => MenuKey::Key(SliderKey::Select),
        other => return Err(format!("unknown key {other}")),
    })
}

pub(super) fn ignore<T>(_: T) {}
