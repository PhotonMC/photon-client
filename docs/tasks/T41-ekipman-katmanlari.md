# T41 — Ekipman katmanları: `SetEquipment`, elde eşya, zırh, kafa eşyası, pelerin/elytra

**Amaç:** Insansı canlıların (oyuncu, zombi, iskelet, armor stand) giydiği/tuttuğu şeyleri çiz: `SetEquipment` paketi → `ClientEntity` ekipmanı, `ItemInHandLayer`, `HumanoidArmorLayer` (zırh dokuları, süsler), `CustomHeadLayer` (kafa/balkabağı), `CapeLayer`, `ElytraLayer`.
**Bağımlılık:** T28, T22. **Boy:** L. **Türler:** oyuncu ve insansı mob ailesi (T28/T29/T40), at eyeri/zırhı T31 sonrası.

## Oku
- Java: `client/renderer/entity/layers/HumanoidArmorLayer.java`, `client/renderer/entity/layers/ItemInHandLayer.java`, `client/renderer/entity/layers/CustomHeadLayer.java`, `client/renderer/entity/layers/CapeLayer.java`, `client/renderer/entity/layers/EquipmentLayerRenderer.java`, `client/renderer/entity/layers/PlayerItemInHandLayer.java`.
- Rust: `crates/photon-render/src/entity/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/photon/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/photon-render/src/entity/textures.rs` (`rect`), `crates/photon-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. `SetEquipment` ayrıştırma ve `ClientEntity.equipment` (slot → yığın; yığın okuyucusu `item_components` artık photon-world'de).
2. Elde eşya: `ItemModelResolver` ile mevcut eşya çizimi (T25 `item_entity`, birinci şahıs `hands`) yeniden kullanılır.
3. Zırh katmanı: `EquipmentClientInfo` (asset kimliği, boyama) verisi oracle'dan tablo; kafa eşyası ve elytra ayrı dallar.

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Eşya/zırh sunucu mantığı, `PlayerInfo` derisi/pelerin indirme (PLAN-M9).

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Zırh dalı + elde eşya dalı + elytra/pelerin dalı. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
- Yapıldı (bölündü): `SetEquipment` (102) → `ClientEntity.equipment` (8 yuva, `EquipmentSlot`), `Avatar` ana kolu (veri 15) ve mobun solak bayrağı; `ArmPose` {Empty, Item, BowAndArrow} `humanoid_anim`'de (iki elli sırası dahil); `held_item::hand_pose` (`ItemInHandLayer`, `translateToHand`: ince kol 0,5, iskelet 1,0, bebek ofseti) ve `photon/held_items.rs` ile uzak oyuncu, zombi/iskelet/piglin ailesi ve yerel oyuncu (üçüncü şahıs) elindeki eşya çizilir; iskeletin yayı artık gerçek eşyadan. Kayıt: `docs/records/client-held-item-parity-conformance.md`.
- Oracle: `entity/held-item-pose` (gerçek `ItemInHandLayer` + gerçek modeller; poz matrisi ve dörtgenler bit bit); oracle iskelet elinin 1 birim kaydığını yakaladı. `entity/variant-fields` yeni satır `avatar-main-hand`.
- Sapma: kullanım pozları ve `SpearAnimations` yok (T41d); yerel oyuncunun ana kolu seçenekten okunmuyor.
- Açık: zırh katmanı (T41b), kafa eşyası/pelerin/elytra (T41c), kullanım ve silah pozları (T41d).
