# T28 — Zombi ailesi (`HumanoidModel` + zombi kolları)

**Amaç:** Zombi, husk, boğulmuş, köylü zombisi, dev (`giant`) ve bebek biçimlerini çiz: `HumanoidModel.setupAnim` (baş/kol/bacak, yürüme) ve zombinin kol kaldırması (`AnimationUtils.animateZombieArms`). Oyuncu modeli (`player_model.rs`) ile paylaşılan insansı iskeleti ortak kodda topla.
**Bağımlılık:** T24, T27. **Boy:** L. **Türler:** zombie, husk, drowned, zombie_villager, giant (yetişkin ve bebek katmanları `zombie`, `zombie_baby`, `drowned`, …).

## Oku
- Java: `client/model/HumanoidModel.java`, `client/model/monster/zombie/ZombieModel.java`, `client/renderer/entity/AbstractZombieRenderer.java`, `client/renderer/entity/ZombieRenderer.java`, `client/renderer/entity/DrownedRenderer.java`, `client/renderer/entity/ZombieVillagerRenderer.java`, `client/model/AnimationUtils.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/voxel/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/voxel-render/src/entity_textures.rs` (`rect`), `crates/voxel-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. Insansı model durumu (`HumanoidRenderState` alanları: baş yaw/pitch, yürüme, saldırı süresi, bebek) ve `setup_humanoid`; oyuncu modeli aynı çekirdeği kullansın (kopya yok).
2. Zombi kol pozu (`aggressive` bayrağı `MobFlags` bit 2 ile, `AnimationUtils.animateZombieArms`).
3. `mobs.rs`: tür → katman/doku tablosu; boğulmuş dış katmanı ve köylü zombisinin meslek/seviye katmanı kapsam dışı (T41/T34).

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Zırh/eşya/giysi katmanları (T41), köylü zombisi meslek katmanı (T34), `die()` yan etkileri.

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
İlk tür (zombi) dalı açar; husk/boğulmuş/dev yalnız doku+ölçek satırı. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
(boş)
