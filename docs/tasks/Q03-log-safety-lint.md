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
(boş)
