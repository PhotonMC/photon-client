# A01b — Kalan `*_bits` alanları ve kimlik newtype'ları

**Amaç:** A01'in bölünen kalanı: bit olarak saklanan son 47 float alanı ve ham kimlikler.
**Bağımlılık:** A01. **Boy:** M.

## Oku
- [A01 devir notu](A01-newtype-degerler.md), inceleme raporu §3.2–3.3
- `grep -rn '_bits: u' crates/*/src`

## Yapılacaklar
- Kalan `*_bits` alanları (`ClientPlayerState` hız/giriş/sağlık, `border`, `animal`, `horse`, `living`, `dimension`, öznitelikler) `JavaF32`/`JavaF64`/`DVec3`.
- `EntityId(i32)`, `BlockStateId`, `EntityTypeId` newtype'ları, önce photon-world içinde, sonra sınır tiplerinde; entity türü dizeyle aranmaz (kimlik tablosundan sabit).

## Kabul
`python3 tools/photon.py check` → `CHECK OK`; davranış değişmez.

## Kapsam dışı
Davranış değişikliği; `ClientEntity`'nin tür enum'una bölünmesi (A02).

## Belgeler
Bu görev dosyasının devir notu.

## Devir notu
- 43 `*_bits: u32/u64` alanı `JavaF32`/`JavaF64` oldu (`ClientPlayerState` hız/giriş/sağlık, `border` (`from`/`to`/`size`/`previous_size`/merkez), `animal`, `horse`, `living`, `dimension`, öznitelik `amount`/`base_value`, hava durumu, `minecart_track`, oyun modu hızları). Betikle dönüştürüldü (`X_bits: e.to_bits()` → `X: JavaF::new(e)`, `f32::from_bits(a.X_bits)` → `a.X.get()`), kalanı derleyici yönlendirmesiyle elle (`border.rs`, `horse::lerp`, `player/travel.rs`). Davranış değişmedi: 542 test, 1109 parity kaydı.
- Kalan `*_bits` yalnız GPU `valid_bits` (bit sayısı, float değil).
- **Kimlik newtype'ları → [A01c](A01c-kimlik-newtype.md):** `EntityId`/`BlockStateId`/`EntityTypeId` 1700+ kullanım yerinde ve tipler tutarsız (`state_id` `i32`/`u32`/`i64`); ayrı görev.
