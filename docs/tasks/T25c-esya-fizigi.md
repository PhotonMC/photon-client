# T25c — Yerdeki eşyanın istemci fiziği (düşme, sekme, kayma)

**Amaç:** Blok kırılınca eşya, Java'daki gibi kırılan yerden küçük bir hızla fırlayıp düşer, yere vurunca hafif sekip kayarak durur. Şimdi eşya yalnız sunucu paketindeki konumda belirir: `ItemEntity` için istemci tiki yok, bu yüzden düşüş görünmez ve eşya yere inmiş hâliyle çıkar. T25'te çizim yapıldı, fizik yapılmadı.
**Bağımlılık:** T25. **Boy:** M (sığmazsa böl: önce hava + zemin, sonra su/lav ve sürtünme blokları).

## Neden istemcide
Sunucu `ItemEntity` konumunu seyrek yollar (varlık türünün güncelleme aralığı uzun; hız değişince `needsSync`). Aradaki tikleri istemci kendi `ItemEntity.tick`'iyle simüle eder; `AddEntity` paketindeki hız bu simülasyonun başlangıcıdır.

## Oku
- Java: `.data/minecraft/source/net/minecraft/world/entity/item/ItemEntity.java` (`tick` 118–192: yerçekimi 0,04, `onGround` iken her 4 tikte bir hareket, sürtünme + zemin sürtünmesi, sekme `y * -0.5`, su/lav kolları, `getBlockPosBelowThatAffectsMyMovement`), `world/entity/Entity.java` (`move` 712–768, `collide`, `getAirDrag`, `applyGravity`), `Block.getFriction`.
- Rust: `crates/voxel-world/src/lib.rs` `tick_entity_once` (kolları `projectile_kind`/boat; `ItemEntity` kolu yok), `entity.rs` `tick_falling_block`/`move_by_motion` (çarpışmasız), `collision.rs` `collide_entity_movement[_state]` (ortak, çarpışma bayrakları dilimi: `docs/records/client-world-movement-flags-conformance.md`), `local_player.rs` `collide` (dünya şekillerini toplayan kısım; entity'ler için ortaklaştırılabilir).

## Yapılacaklar
1. `ItemEntity.tick` istemci kolu: `age`, `pickupDelay`, yerçekimi, `Entity.move` (dünya çarpışması + `onGround`), sürtünme, sekme. Çarpışma şekillerini toplama kodunu `local_player`'dan entity'lerle paylaş (DRY).
2. Su/lav kolları ve zemin sürtünmesi (`Block.getFriction`) ayrı adım olabilir; bölünürse yeni görev aç.
3. `entity_snapshot` zaten `capture_old_transform` ile ara konum veriyor; çizim değişmez.

## Oracle
`item_entity_tick`: Java `ItemEntity`'yi sahte dünyada (zemin bloğu, hız, tik sayısı) N tik koştur, her tikte konum/hız/`onGround` bit bit karşılaştır. Hava, zemine düşüş, sekme, kayma, blok yanı çarpışması, buz gibi farklı sürtünmeli zemin satırları.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: ağaç/blok kırınca eşya fırlayıp düşer (`shot` ile ardışık iki kare: düşerken ve yere inmiş).

## Kapsam dışı
Toplama animasyonu (`TakeItemEntity`), birleşme (sunucu tarafı), XP küresi (T25b).

## Belgeler
Kendi `client-parity` kaydı (entity hareket belgeleri gibi); `docs/index.md` girdisi.

## Devir notu
- Yapıldı: `voxel-world/src/item_physics.rs` (`ItemEntity.tick` istemci kolu: yerçekimi, çarpışma, blok hız çarpanı, `stepOn` slime, sürtünme, sekme, destek bloğu), `Level::{check_supporting_block,on_pos,block_speed_factor}` `local_player.rs`'den paylaşıma çıkarıldı (oyuncu aynı kodu çağırır). Oracle: `MovementOracle.item_entity_ticks` gerçek `ClientLevel`+`ItemEntity`, 28 satır, bit bit geçti. Kayıt: `docs/records/client-world-entity-item-tick-parity-conformance.md`.
- Sonradan (canlı geri bildirim): göreli `MoveEntity` paketleri entity'nin kendi konumuna uygulanıyordu; Java `VecDeltaCodec` tabanına uygular (`ClientEntity.tracking.packet_base`, AddEntity/Move/PositionSync'te güncellenir). İstemci eşyayı ilerletince eşya sunucu konumuna ışınlanıyordu; oracle'a paket olayları eklendi (32 satır, canlı iz: iniş anındaki sync farkı 3e-4).
- Sapma: akışkana değen eşya simüle edilmez (sunucu konumunda kalır), blok içi etkiler, dünya sınırı ve entity çarpışması yok. Canlı kare düşüşü yakalayamadı (`shot` zamanlaması); düşüşün kanıtı parity.
- Açık: T25d (akışkan: kaldırma, akıntı; `Entity.updateFluidInteraction` ortak hâle getirilip oyuncuyla paylaşılabilir).
