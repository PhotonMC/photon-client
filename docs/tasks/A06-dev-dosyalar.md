# A06 — Dev dosyaları ve oracle adaptörünü bölme

**Amaç:** Gezinilemeyen dosyaları sorumluluğa göre bölmek; test harness'ını ayrı crate'e almak.
**Bağımlılık:** –. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §2, §7
- `crates/voxel/examples/behavior_adapter/main.rs`, `crates/voxel-world/src/{tests,lib}.rs`, `crates/voxel-net/src/connection/protocol.rs`, `crates/voxel-render/src/mob_model.rs`

## Yapılacaklar
- `crates/voxel-oracle` (publish=false) crate'i; adaptör aileye göre modüllere (GUI örneği zaten var).
- `tests.rs` konu modüllerine; `protocol.rs` paket ailesine; `mob_model.rs` aileye göre.
- Hedef: hiçbir kaynak dosyası 1500 satırı aşmaz (üretilmiş tablolar hariç; D görevleri onları kaldırır).
- Birden fazla oturum gerekirse A06b, A06c.

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Mantık değişikliği.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
