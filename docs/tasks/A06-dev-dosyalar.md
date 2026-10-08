# A06 — Dev dosyaları ve oracle adaptörünü bölme

**Amaç:** Gezinilemeyen dosyaları sorumluluğa göre bölmek; test harness'ını ayrı crate'e almak.
**Bağımlılık:** –. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §2, §7
- `crates/photon/examples/behavior_adapter/main.rs`, `crates/photon-world/src/{tests,lib}.rs`, `crates/photon-net/src/connection/protocol.rs`, `crates/photon-render/src/entity/mob_model.rs`

## Yapılacaklar
- `crates/photon-oracle` (publish=false) crate'i; adaptör aileye göre modüllere (GUI örneği zaten var).
- `tests.rs` konu modüllerine; `protocol.rs` paket ailesine; `mob_model.rs` aileye göre.
- Hedef: hiçbir kaynak dosyası 500–600 satırı aşmaz ([kodlama standartları](../coding-standards.md#dosya-boyutu); üretilmiş tablolar hariç, D görevleri onları kaldırır). İlk turda 1500 satırın üstü, sonra kademeli olarak 600'e inilir.
- Birden fazla oturum gerekirse A06b, A06c.

## Kabul
Davranış değişmez: `python3 tools/photon.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Mantık değişikliği.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
- Kural: bir kod dosyası 500–600 satırı geçmez (AGENTS.md, `docs/coding-standards.md` "Dosya boyutu", CONTRIBUTING). Hedef 1500'den 600'e çekildi.
- **Bölünen (Rust, 55 dosya → ≈250 modül):** `photon-world` ve `photon-net` test dosyaları konu modüllerine; `ClientEntity`, `LocalPlayer`, `ClientWorld`, `PlaySession`, `Controls`, `VulkanBackend`, `MobModel`, `BlockRenderer`, `ContainerMenu`, `BlockPos`… impl blokları, serbest işlevler (protokol ayrıştırıcıları, matematik, ses) ve tipler sorumluluğa göre alt modüllere. Görünürlük korundu (`pub(super)`/`pub(in …)`), harici yollar `pub use` ile aynı; mantık satırı değişmedi. Doğrulama: 539 test, 1109 parity, clippy temiz. Üretici betikler (ayrı `split_*.py`) depoya girmedi.
- Oracle adaptörü (`examples/behavior_adapter/main.rs` 27k satır) 58 dosyaya bölündü; ayrı `photon-oracle` crate'ine taşıma yapılmadı (A06b).
- **Kalan (600'ü aşan, güncel liste [A06b devir notunda](A06b-dev-islevler.md)):** Rust yalnız `probe/mode.rs` (A04 ile); Python yalnız `tools/photon.py` 740 (`cases` paketine bölündü); Java oracle'ları (`MathOracle` 13k, `EntityOracle` 6.2k, `MovementOracle` 3.1k, `GuiOracle` 2.8k, `BlockOracle` 1.7k, `WireOracle` 1.5k, `SoundOracle` 706, `ModelOracle` 675). Adaptörün dev işlevleri, `handle` ve `outline` A06b'de bölündü.
