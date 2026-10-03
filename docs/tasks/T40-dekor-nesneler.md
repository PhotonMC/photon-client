# T40 — Dekor ve nesneler: zırh askısı, çerçeve, tablo, bağ düğümü, kristal, TNT, düşen blok, yıldırım, display'ler

**Amaç:** Dünyaya yerleştirilen entity'ler: zırh askısı (pozlar, küçük, kollu), eşya çerçevesi (içerik, dönüş), tablo (varyant kaydı, yüz), bağ düğümü ve `SetEntityLink` bağı, ender kristali (ışın), TNT (yanıp sönme), düşen blok, yıldırım, `display` entity'leri (blok/eşya/yazı).
**Bağımlılık:** T24. **Boy:** L. **Türler:** armor_stand, item_frame, glow_item_frame, painting, leash_knot, end_crystal, tnt, falling_block, lightning_bolt, block_display, item_display, text_display, firework_rocket(hariç T39).

## Oku
- Java: `client/model/object/armorstand/ArmorStandModel.java`, `client/renderer/entity/ArmorStandRenderer.java`, `client/renderer/entity/ItemFrameRenderer.java`, `client/renderer/entity/PaintingRenderer.java`, `client/renderer/entity/LeashKnotRenderer.java`, `client/renderer/entity/EndCrystalRenderer.java`, `client/renderer/entity/TntRenderer.java`, `client/renderer/entity/FallingBlockRenderer.java`, `client/renderer/entity/LightningBoltRenderer.java`, `client/renderer/entity/DisplayRenderer.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/voxel/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/voxel-render/src/entity_textures.rs` (`rect`), `crates/voxel-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. Armor stand (`ArmorStandModel`, 7 poz verisi: `Rotations` serileştiricisi T24b'de yürünüyor), marker/küçük bayrakları (T26 notu: marker alanı).
2. Çerçeve ve tablo: duvara yapışık ikili geometri; tablo varyant kaydı (`PaintingVariant` holder, T24b'de doğrudan değer atlanıyordu).
3. `SetEntityLink` paketi ve bağ çizimi (`LeashKnot`, `Leashable`).
4. Display entity'leri: ayrı alt görev (yazı yerleşimi `Font` ile, enterpolasyon).

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Display entity'lerinin tüm enterpolasyon kolları ilk oturumda; blok entity çizicileri PLAN-M8.

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Çok bağımsız dal; ilk oturumda armor stand + çerçeve + tablo. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
- Yapıldı (bölündü): zırh askısı (büyük/küçük): `ArmorStandState` + `setup_armor_stand` (ana model ve zırh modelleri), `voxel-world/armor_stand.rs` (bayrak ve altı dönüş verisi birleştirilir), `voxel/armor_stands.rs` (zırh, elde eşya). Kayıt: `docs/client-armor-stand-parity-conformance.md`.
- Oracle: `entity/armor-stand-model` 40 satır (gerçek model sınıfları); `variant-fields`'e yedi veri indeksi.
- Sapma: `wiggle` ve işaretçi render türü yok.
- Açık: çerçeve, tablo, bağ düğümü, kristal, TNT, düşen blok, yıldırım, display'ler (T40b).
