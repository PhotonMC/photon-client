# T21b — Mob'ların konum/dönüş interpolasyonu

**Amaç:** `ClientWorld::tick_entity_once` konum, hareket ve kafa interpolasyonunu yalnız oyuncu entity'si için çalıştırıyor (`VANILLA_PLAYER_ENTITY_TYPE_ID`). Java'da bütün `LivingEntity`'ler (mob'lar, zırh askısı dahil değil) aynı `InterpolationHandler` yolunu izler; uzaktaki mob'lar sunucu paketleriyle adım adım değil, üç tikte yumuşak ilerlemeli.
**Bağımlılık:** T21. **Boy:** M.

## Oku
- Rust: `crates/voxel-world/src/lib.rs` (`tick_entity_once` ~1488–1560: `tick_position_interpolation`, `tick_motion_interpolation`, `tick_head_rotation`; `apply_entity_move`/`apply_entity_teleport`'in `lerpTo` kolu), `crates/voxel-world/src/entity.rs` (`interpolation_steps`, `interpolation_target_position`), `docs/records/client-world-entity-transform-parity-conformance.md` (yalnız "kapsam").
- Java: `.data/minecraft/source/net/minecraft/world/entity/LivingEntity.java` (`lerpTo`, `aiStep`, `lerpSteps`/`InterpolationHandler`), `Entity.java` (`moveOrInterpolateTo`), `InterpolationHandler.java`, `ClientLevel.java` (`tickNonPassenger`).

## Yapılacaklar
1. Hangi entity türlerinin `LivingEntity` olduğunu **Java'dan** al (oracle'dan tablo; tahmin yok): tür kimliği → `living: bool`. Tabloyu `voxel-data/src/generated` altına üretilmiş dosya olarak yaz, her parity koşusunda karşılaştır (AGENTS: tabloyu veri olarak al).
2. `tick_entity_once` içinde interpolasyon çağrılarını oyuncu yerine `living` tablosuna bağla; `apply_entity_move/teleport/position_sync` kollarında interpolasyon hedefini aynı koşula bağla.
3. Mevcut `entity` ailesine bir mob türü (domuz, zombi) için `MoveEntity` + birkaç tik satırı ekle; Java tarafı gerçek `LivingEntity` sahtesiyle kurulur.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`; canlı: `summon pig` + `shot`, `entity_snapshot` konumu iki paket arasında her tikte değişir (log).

## Kapsam dışı
AI, yerçekimi, çarpışma, çizim.

## Belgeler
`docs/remaining-work.md` M4 satırı.

## Devir notu
- Yapıldı: `ClientEntity::living_kind` (oyuncu, mob, diğer: yaşayan tür listesi `VANILLA_LIVING_ENTITY_TYPE_IDS` + zırh askısı ayrımı), konum/dönüş/kafa interpolasyonu ve `tick_living_pose` artık bütün yaşayan entity'lerde; mob'larda `BodyRotationControl.clientTick` (`living::tick_mob_body`, `Mth.rotateIfNecessary`), yürüme animasyonu hepsinde; hareket interpolasyonu (`lerpMotion`) yalnız oyuncuda kaldı (Java'da `RemotePlayer` geçersiz kılar). Yüklü olmayan yığına süzülen yaşayan entity tiklemeyi bırakır. Entity tür tablosu (`entity_types` oracle'ı, `voxel-data::entity_type`) bu görevle geldi.
- Tablo: yaşayan türler elle yazılmış listeden (önceki oturumlardan), oracle'dan yeniden üretilmedi; yeni bir tür çıkarsa liste güncellenir.
- Sapma: `getMaxHeadYRot` her mob için 75 (geçersiz kılan mob'lar — örn. bazı su canlıları — işlenmedi); `omnidirectionalAirMover` (uçan mob'larda yürüme y'yi sayar) yok.
- Açık: türe özgü mob istisnaları (T24+).
