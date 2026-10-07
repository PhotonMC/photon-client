# A01 — `JavaF64`/`DVec3` ve kimlik newtype'ları

**Amaç:** Bit olarak saklanan float'ları ve ham `i32` kimlikleri tiplere almak.
**Bağımlılık:** –. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §3.2, §3.3
- `crates/voxel-world/src/entity.rs:661`, `grep -rn '_bits: u' crates/*/src`

## Yapılacaklar
- `voxel-core`: `JavaF64`, `JavaF32` (Eq/Hash = bit eşitliği, aritmetik Java kurallarıyla), `DVec3`, `Vec2f`.
- `EntityId`, `BlockStateId`, `EntityTypeId` newtype'ları; `OpenContainerMenu` tuple'ı adlandırılmış yapı.
- Önce `ClientEntity` konum/dönüş/hareket alanları; diğer `*_bits` alanları aynı commit'e sığmazsa A01b.

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Davranış değişikliği.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
