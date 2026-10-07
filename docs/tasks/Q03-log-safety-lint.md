# Q03 — Log altyapısı, SAFETY yorumları, lint sıkılaştırma

**Amaç:** `println!` tabanlı çıktıyı seviyeli log'a çevirmek; `unsafe` sözleşmelerini yazılı yapmak.
**Bağımlılık:** Q01. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §4.4, §5 (5.11)
- `grep -rn 'println!\|eprintln!' crates/*/src`

## Yapılacaklar
- `log` + küçük bir arka uç; betik/probe raporları ayrı tutulur (makine okunur satırlar değişmez).
- Workspace lint'leri: `clippy::undocumented_unsafe_blocks`, `clippy::unwrap_used` (yalnız üretim kodu; testlerde `#[allow]` değil modül düzeyi `expect`).
- Her `unsafe` bloğuna `// SAFETY:` gerekçesi; `unsafe fn`'lere `# Safety` belgesi.
- Sessizce yutulan ayrıştırma hataları (`if let Ok(packet)`) `debug!` ile kaydedilir.

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Davranış değişikliği.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
- Lint: `undocumented_unsafe_blocks` ve `unwrap_used` workspace'te `warn` (CI `-D warnings`); `clippy.toml` `allow-unwrap-in-tests`. Üretim kodunda `unwrap` zaten 0 çıktı (141'i testlerde).
- `unsafe`: 50 bloğa `// SAFETY:` (48 `vulkan.rs`, `play.rs`, `window_probe.rs`), `unsafe fn`'lere `# Safety`; ortak değişmezler `vulkan.rs` başında. 24 ham bayt görünümü tek `as_bytes` + `Plain` özelliğine indi (dolgusuz `repr(C)` tipler, boyut `const` iddiasıyla); push sabiti dilimleri artık sınır denetimli. Bayt dizisi aynı; çizim canlı koşulmadı (yalnız derleme, test, clippy, parity).
- Log: `log` (std özelliği) + `voxel/src/logger.rs` (stderr, `VOXEL_LOG`, varsayılan info), ek bağımlılık yok (`log` zaten kilitteydi). `eprintln!`/tanılama `println!` → `warn!/error!/info!`; probe ve profil `anahtar=değer` satırları `println!` kaldı (stdout'u okuyan betik yok, yine de değişmedi).
- Yutulan ayrıştırma hataları: `connection/play.rs`'te 19 `if let Ok` → `parsed(..)` ile `debug!` kaydı.
- Silindi: `block_entities.rs`'te önceki commit'ten kalan her karede basılan `BANNERDEBUG` `eprintln!`'i.
- **Açık sorun (görev dışı, çözülmedi, yeniden üretilmedi):** `render_frame` başında emekli bölümler serbest bırakılıyor, bekleyen `staged` kopyaları ise sonra kaydediliyor. Bir bölüm aynı karede iki kez yüklenirse ilkinin kopyası serbest kalmış (belki `trim_arena` ile yok edilmiş ya da başka bloğa yeniden atanmış) aralığı hedefler. V03 (kare uçuşta) veya V06 çözerken ele alınmalı.
