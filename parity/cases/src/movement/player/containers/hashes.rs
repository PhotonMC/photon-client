//! The hashes of item components, as the reference's own codecs compute them.

use serde_json::{Value, json};

use super::*;
use crate::components::{
    CUSTOM_DATA, DYED_COLOR, GLINT, ITEM_NAME, LORE, PROTECTION, RARITY, Tag, UNBREAKABLE, compound, nbt,
};

fn hashed(label: &str, item: &str, hex_patch: String) -> Value {
    json!({"label": label, "item": format!("minecraft:{item}"), "count": 1, "patch": hex_patch})
}

fn flag(on: bool) -> Tag {
    Tag::Byte(i64::from(on))
}

fn styled_name() -> Tag {
    compound(vec![("text", "Bold".into()), ("bold", flag(true)), ("italic", flag(false)), ("color", "red".into())])
}

fn siblings() -> Tag {
    let tail = compound(vec![("text", "b".into()), ("underlined", flag(true))]);
    compound(vec![("text", "".into()), ("extra", Tag::List(vec!["a".into(), tail]))])
}

fn translated() -> Tag {
    compound(vec![("translate", "item.minecraft.stone".into()), ("with", Tag::List(vec!["x".into()]))])
}

fn custom_data() -> Tag {
    compound(vec![
        ("flag", flag(true)),
        ("short", Tag::Short(-2)),
        ("int", Tag::Int(70000)),
        ("long", Tag::Long(-5)),
        ("float", Tag::Float(0.5)),
        ("double", Tag::Double(-1.25)),
        ("text", "\u{15f}".into()),
        ("bytes", Tag::Bytes(vec![1, -1])),
        ("ints", Tag::Ints(vec![3, 4])),
        ("longs", Tag::Longs(vec![5])),
        ("list", Tag::List(vec![Tag::Int(1), Tag::Int(2)])),
        ("nested", compound(vec![("empty", Tag::List(vec![]))])),
    ])
}

fn name_rows() -> Vec<Value> {
    let named = |component: i64, tag: Tag| patch(&[(component, nbt(&tag))]);
    vec![
        hashed("plain custom name", "stone", named(CUSTOM_NAME, "Hello, D\u{fc}nya".into())),
        hashed("styled item name", "stone", named(ITEM_NAME, styled_name())),
        hashed("name with siblings", "stone", named(CUSTOM_NAME, siblings())),
        hashed("translated name", "stone", named(CUSTOM_NAME, translated())),
        hashed(
            "lore lines",
            "paper",
            patch(&[(
                LORE,
                format!(
                    "{}{}{}",
                    vi(2),
                    nbt_text("first"),
                    nbt(&compound(vec![("text", "second".into()), ("italic", flag(false))]))
                ),
            )]),
        ),
    ]
}

fn plain_rows() -> Vec<Value> {
    vec![
        hashed("damage and repair cost", "iron_sword", patch(&[(DAMAGE, vi(12)), (REPAIR_COST, vi(3))])),
        hashed("max stack size and max damage", "stick", patch(&[(MAX_STACK_SIZE, vi(16)), (MAX_DAMAGE, vi(250))])),
        hashed("unbreakable", "iron_pickaxe", patch(&[(UNBREAKABLE, String::new())])),
    ]
}

fn enchantment_rows() -> Vec<Value> {
    vec![
        hashed("rarity", "stick", patch(&[(RARITY, vi(3))])),
        hashed(
            "enchantments",
            "diamond_sword",
            patch(&[(ENCHANTMENTS, enchantments(&[(SHARPNESS, 5), (UNBREAKING, 3), (MENDING, 1)]))]),
        ),
        hashed("no enchantments on a book", "book", patch(&[(STORED_ENCHANTMENTS, enchantments(&[]))])),
        hashed(
            "stored enchantments",
            "enchanted_book",
            patch(&[(STORED_ENCHANTMENTS, enchantments(&[(PROTECTION, 4)]))]),
        ),
        hashed("glint override", "stick", patch(&[(GLINT, "00".to_owned())])),
        hashed("dyed colour", "leather_helmet", patch(&[(DYED_COLOR, "00a06540".to_owned())])),
    ]
}

fn nested_rows() -> Vec<Value> {
    let inner = patch_of(&[(CUSTOM_NAME, nbt_text("x"))], &[MAX_STACK_SIZE]);
    let contents = bundle(&[stack("stone", 1), ("dirt", 5, inner), ("bundle", 1, bundle(&[stack("ender_pearl", 3)]))]);
    vec![
        hashed("custom data", "stick", patch(&[(CUSTOM_DATA, nbt(&custom_data()))])),
        hashed("bundle contents", "bundle", contents),
        hashed(
            "added and removed together",
            "diamond_sword",
            patch_of(&[(CUSTOM_NAME, nbt_text("Mixed")), (DAMAGE, vi(1))], &[MAX_STACK_SIZE]),
        ),
    ]
}

pub fn component_hashes() -> Value {
    let rows = [&plain_rows()[..2], &plain_rows()[2..], &name_rows()[..], &enchantment_rows()[..], &nested_rows()[..]]
        .concat();
    case("movement/component-hashes", "movement", "component_hashes", json!({"rows": rows}))
}
