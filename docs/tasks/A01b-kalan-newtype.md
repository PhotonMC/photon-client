# A01b — Kalan `*_bits` alanları ve kimlik newtype'ları

**Amaç:** A01'in bölünen kalanı: bit olarak saklanan son 47 float alanı ve ham kimlikler.
**Bağımlılık:** A01. **Boy:** M.

## Oku
- [A01 devir notu](A01-newtype-degerler.md), inceleme raporu §3.2–3.3
- `grep -rn '_bits: u' crates/*/src`

## Yapılacaklar
- Kalan `*_bits` alanları (`ClientPlayerState` hız/giriş/sağlık, `border`, `animal`, `horse`, `living`, `dimension`, öznitelikler) `JavaF32`/`JavaF64`/`DVec3`.
- `EntityId(i32)`, `BlockStateId`, `EntityTypeId` newtype'ları, önce voxel-world içinde, sonra sınır tiplerinde; entity türü dizeyle aranmaz (kimlik tablosundan sabit).

## Kabul
`python3 tools/voxel.py check` → `CHECK OK`; davranış değişmez.

## Kapsam dışı
Davranış değişikliği; `ClientEntity`'nin tür enum'una bölünmesi (A02).

## Belgeler
Bu görev dosyasının devir notu.

## Devir notu
(boş)
