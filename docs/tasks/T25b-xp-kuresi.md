# T25b — XP küresi

**Amaç:** `ExperienceOrb` entity'sini çiz: küre sprite'ı (`textures/entity/experience_orb.png` 4×4 ızgara), renk salınımı, boyut (`value` → ikon indeksi).
**Bağımlılık:** T25. **Boy:** S.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/renderer/entity/ExperienceOrbRenderer.java`, `.../state/ExperienceOrbRenderState.java`, `.../world/entity/ExperienceOrb.java` (`getIcon`), `AddExperienceOrb` paketi (`ClientboundAddExperienceOrbPacket`).
- Rust: `crates/voxel/src/dropped_items.rs`, `crates/voxel-render/src/item_entity.rs` (oracle ve çizim kalıbı), `entity_textures.rs` (doku paketleme).

## Yapılacaklar
1. Orb'un `value`'su (paket) → `ClientEntity`; ikon indeksi, renk salınımı (`age` + `partial_tick`), her zaman kameraya dönük dörtgen.
2. Oracle: `experience_orb` operation'ı (entity ailesi), sahte render durumuyla köşe/renk sayıları.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `shot --rcon "summon experience_orb ~ ~ ~3 {Value:20}"`.

## Kapsam dışı
Toplama animasyonu, birleşme.

## Belgeler
`docs/remaining-work.md` M4 madde 3.

## Devir notu
- Yapıldı: `voxel-render/xp_orb.rs` (`ExperienceOrbRenderer.submit` dörtgeni: ikon eşikleri, yaşa bağlı renk, kameraya dönük pozlama, dönmüş normal ve CPU'da yüz ışığı), `PipelineKind::TextTranslucent` (dünya yazısı gölgelendiricileri entity dokusu üstünde, yarı saydam, derinlik yazmadan), `ClientEntity.orb_value` (`DATA_VALUE`, veri indeksi 8), `overlays::experience_orbs`. Kayıt: `docs/client-xp-orb-parity-conformance.md`.
- Oracle: `entity/xp-orb` 25 satır (gerçek çizici + sahte toplayıcı; konum/renk/uv/ışık/normal bit bit), `entity/variant-fields`'e orb alanı. Canlı: izleyici kipinde beş değerli küre görünür (oyuncu yakınındaki küreleri toplar, bu yüzden izleyici).
- Sapma: alfa kesimi çarpımdan sonra; ışık tek örnek; yarı saydam karışım ekranla doğrulandı.
- Açık: yok.
