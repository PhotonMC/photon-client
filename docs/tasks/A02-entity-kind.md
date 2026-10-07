# A02 — `ClientEntity` → `EntityBase` + `EntityKind` enum

**Amaç:** Türe özgü ~30 `Option` alanını toplam tipe çevirmek; geçersiz durumları temsil edilemez yapmak.
**Bağımlılık:** A01. **Boy:** L.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §3.1
- `crates/voxel-world/src/entity.rs:661-818`, tür modülleri (`rabbit.rs`, `warden.rs`…)

## Yapılacaklar
- `EntityBase` (ortak), `LivingState` (canlılar), `EntityKind` enum'u (büyük durumlar `Box`).
- Erişimciler `match`/`if let EntityKind::Rabbit(..)` ile; türü değişmeyen entity için kurulumda bir kez seçilir.
- Sığmazsa aile aile (canlılar, araçlar, mermiler) bölünür.

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Render tarafı (A03).

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
- `voxel-world/src/entity_kind.rs`: `EntityKind` enum'u (`Plain` + 27 tür durumu, hepsi `Box`ta; tavşan, kalamar, binek, ender ejderhası, display, minecart…) bir makroyla; her tür için `x()`, `x_mut()`, `x_or_insert_with(..)` erişimcileri. `ClientEntity`'deki 28 `Option<..>` alanı (`rabbit`, `warden`, `horse`, `minecart`, `minecart_track`, `display`…) tek `kind` alanına indi. Bir entity ömrü boyunca tek türdür: başka türün durumu istenirse `panic` (hata); `display` ve yeni davranışlı minecart türü kurulumda seçilir, geri kalanı durum ilk geldiğinde (`Plain` → tür) oluşur. Minecart'ın iki durumu (`MinecartState`, `TrackLerp`) birbirinden bağımsız oluşabildiği için tek `MinecartSlot` varyantında.
- Erişim yeri ≈130 (çoğu `entity.rs`): yaygın biçimler betikle, kalanı elle dönüştürüldü. Davranış değişmedi: 539 test ve 1109 parity kaydı aynı, clippy temiz.
- Sapma: `ClientEntity` adı korundu ve `EntityBase`/`LivingState` ayrımı yapılmadı (ad değişikliği ve ortak alan bölmesi ayrı, mekanik ve büyük bir tarama; tür enum'u asıl güvenlik kazancını sağladı). Canlı varlığa özgü ortak alanlar (`living`, `equipment`, `attributes`…) hâlâ `ClientEntity`'de.
- Kalan: `living_kind()` ile tür ailelerini ayıran `Option`/bayrak desenleri ve `is_*` yüklemleri (tür kimliğine göre dizeyle aramalar) → A01b'nin kimlik tipleriyle birlikte.
