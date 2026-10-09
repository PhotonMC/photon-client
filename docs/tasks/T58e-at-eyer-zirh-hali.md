# T58e — At ailesi eyeri, zırhı ve lama halısı

**Amaç:** `SimpleEquipmentLayer` (at, eşek, katır, iskelet/zombi atı eyeri; at zırhı) ve `LlamaDecorLayer` (halı, lama zırhı) dünya çiziminde ve binek önizlemesinde (`farm_animals::gui_preview`).
**Bağımlılık:** T58d. **Boy:** M.

## Oku
Java: `client/renderer/entity/{HorseRenderer,LlamaRenderer}.java`, `layers/{SimpleEquipmentLayer,LlamaDecorLayer}`. Rust: `crates/photon/src/mobs/aquatic/nautilus.rs` (`worn_layers` örnek), `equipment/armor.rs` (`simple_layer`), `mobs/animals/farm_animals.rs`.

## Yapılacaklar
Dünya çizimine katmanları ekle; `gui_preview` aynı yardımcıyı kullansın (kopyalama). Oracle/kabul/belgeler T58d gibi.

## Devir notu
(boş)
