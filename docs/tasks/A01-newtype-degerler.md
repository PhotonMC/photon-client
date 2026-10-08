# A01 — `JavaF64`/`DVec3` ve kimlik newtype'ları

**Amaç:** Bit olarak saklanan float'ları ve ham `i32` kimlikleri tiplere almak.
**Bağımlılık:** –. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §3.2, §3.3
- `crates/photon-world/src/entity.rs:661`, `grep -rn '_bits: u' crates/*/src`

## Yapılacaklar
- `photon-core`: `JavaF64`, `JavaF32` (Eq/Hash = bit eşitliği, aritmetik Java kurallarıyla), `DVec3`, `Vec2f`.
- `EntityId`, `BlockStateId`, `EntityTypeId` newtype'ları; `OpenContainerMenu` tuple'ı adlandırılmış yapı.
- Önce `ClientEntity` konum/dönüş/hareket alanları; diğer `*_bits` alanları aynı commit'e sığmazsa A01b.

## Kabul
Davranış değişmez: `python3 tools/photon.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Davranış değişikliği.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
- `photon-core/src/java_float.rs`: `JavaF64`, `JavaF32` (Eq/Hash/PartialEq bit eşitliğiyle, `get`/`new`/`to_bits`/`from_bits`) ve `DVec3` (üç `JavaF64`); birim testli (±0, NaN, bitlerin aynen dönmesi, karma).
- `ClientEntity` ve `ClientPlayerState`: konum/eski konum/hareket (`position`, `old_position`, `motion`, `motion_lerp_target`) ve interpolasyon hedef/önceki konumu (`target`, `previous`) tek `DVec3` alanına indi; dönüş, yatış, kafa/gövde açısı, `swim_amount`, `bob_offset`, boyut (`width`, `height`, `eye_height`) vb. `JavaF32`. 31+ `*_bits` alanı ve `from_bits`/`to_bits` gürültüsü kalktı (≈500 erişim, betikle dönüştürüldü, derleyiciyle düzeltildi). Davranış değişmedi: 539 test ve 1109 parity kaydı aynı.
- `OpenContainerMenu` isimsiz 5'li tuple yerine adlandırılmış yapı (alanlar `container_id`, `menu_type_id`, `slots`, `carried`, `state_id`).
- **Kalan → A01b:** `*_bits: u32/u64` hâlâ 47 alanda (`ClientPlayerState`'in hız/giriş/sağlık alanları, `border.rs`, `animal.rs`, `horse.rs`, `living.rs`, öznitelik değeri…) ve kimlik newtype'ları (`EntityId`, `BlockStateId`, `EntityTypeId`): bunlar binlerce kullanım yeri olan ayrı bir tarama; entity türünün dizeyle aranması (`entity_type::id("minecraft:fox")`) kimlik tipleriyle birlikte ele alınmalı.
