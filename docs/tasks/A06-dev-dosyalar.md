# A06 — Dev dosyaları ve oracle adaptörünü bölme

**Amaç:** Gezinilemeyen dosyaları sorumluluğa göre bölmek; test harness'ını ayrı crate'e almak.
**Bağımlılık:** –. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §2, §7
- `crates/voxel/examples/behavior_adapter/main.rs`, `crates/voxel-world/src/{tests,lib}.rs`, `crates/voxel-net/src/connection/protocol.rs`, `crates/voxel-render/src/mob_model.rs`

## Yapılacaklar
- `crates/voxel-oracle` (publish=false) crate'i; adaptör aileye göre modüllere (GUI örneği zaten var).
- `tests.rs` konu modüllerine; `protocol.rs` paket ailesine; `mob_model.rs` aileye göre.
- Hedef: hiçbir kaynak dosyası 500–600 satırı aşmaz ([kodlama standartları](../coding-standards.md#dosya-boyutu); üretilmiş tablolar hariç, D görevleri onları kaldırır). İlk turda 1500 satırın üstü, sonra kademeli olarak 600'e inilir.
- Birden fazla oturum gerekirse A06b, A06c.

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Mantık değişikliği.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
- Kural: bir kod dosyası 500–600 satırı geçmez (AGENTS.md, `docs/coding-standards.md` "Dosya boyutu", CONTRIBUTING). Hedef 1500'den 600'e çekildi.
- **Bölünen (Rust, 55 dosya → ≈250 modül):** `voxel-world` ve `voxel-net` test dosyaları konu modüllerine; `ClientEntity`, `LocalPlayer`, `ClientWorld`, `PlaySession`, `Controls`, `VulkanBackend`, `MobModel`, `BlockRenderer`, `ContainerMenu`, `BlockPos`… impl blokları, serbest işlevler (protokol ayrıştırıcıları, matematik, ses) ve tipler sorumluluğa göre alt modüllere. Görünürlük korundu (`pub(super)`/`pub(in …)`), harici yollar `pub use` ile aynı; mantık satırı değişmedi. Doğrulama: 539 test, 1109 parity, clippy temiz. Üretici betikler (ayrı `split_*.py`) depoya girmedi.
- Oracle adaptörü (`examples/behavior_adapter/main.rs` 27k satır) 58 dosyaya bölündü; ayrı `voxel-oracle` crate'ine taşıma yapılmadı (A06b).
- **Kalan (600'ü aşan, hepsi tek dev işlev ya da dosya):** Rust: `voxel-net/src/connection/probe/mode.rs` 2238 (tek işlev `connect_and_probe_with_mode`, test izleme), `play/incoming/play_session_items.rs` 836 (paket işleyici `match`), `voxel-world/.../shape_rules/singles/outline.rs` 670, adaptör `math_cases.rs` 3406, `shape.rs` 2170, `ticks_2.rs` 1700, `gui/result.rs` 1343, `entity.rs` 647 (hepsi `match operation` kolları). Python: `tools/behavior/cases.py` 19.4k (tek üretici `math_cases` 10.2k satır), `cases_gui.py` 2.3k. Java oracle'ları: `MathOracle` 13k, `EntityOracle` 6k, `MovementOracle` 3k, `GuiOracle` 2.8k, `BlockOracle` 1.7k, `WireOracle` 1.5k (tek sınıf, statik yöntemler). Bunlar mekanik bölünemez: kol/yield başına işleve ayırma ya da alt sınıf zinciri gerekir → A06b.
