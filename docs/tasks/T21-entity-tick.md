# T21 — Entity tick ve interpolasyon canlı döngüde

**Amaç:** T20'den gelen entity'ler canlı oyun döngüsünde her istemci tikinde ilerlesin (hareket, interpolasyon) ve çizim için `partial_tick` ile konum/dönüş verilsin.
**Bağımlılık:** T20. **Boy:** M.

## Oku
- Rust: `crates/photon-world/src/lib.rs` (entity tik/interpolasyon API'si: `grep -n "pub fn .*tick\|pub fn .*interpol\|pub fn lerp"`), `crates/photon-world/src/entity.rs` (`ClientEntity`, konum/dönüş geçmişi), docs `client-world-entity-transform-parity-conformance.md` (yalnız "kapsam" bölümü), `crates/photon/src/play.rs` (oyun döngüsü: yerel oyuncu tik çağrısı nerede, `world.tick` var mı: `grep -n "tick"`), `crates/photon-net/src/connection/play.rs` (`PlaySession::tick`).
- Java: `.data/minecraft/source/net/minecraft/client/multiplayer/ClientLevel.java` (`tickEntities`, `tickNonPassenger`), `.data/minecraft/source/net/minecraft/world/entity/Entity.java` (`lerpTo`, `getLerpedPosition`, `InterpolationHandler` kullanımı), `.data/minecraft/source/net/minecraft/world/entity/InterpolationHandler.java`.

## Yapılacaklar
1. Canlı tik: her istemci tikinde `ClientWorld`'ün entity tik fonksiyonları (yalnız mevcut ve `client-parity` ile doğrulanmış olanlar: ticking eligibility, transform, passengers) çağrılır; yerel oyuncu hariç entity'ler.
2. Çizim sorgusu: `entity_snapshot(id, partial_tick) -> {pos, yaw, pitch, head_yaw}` (Java `getLerpedPosition`/`Mth.rotLerp`).
3. Yer çekimi/çarpışma ile hareket **yok** (uzak entity'ler sunucu konumunu izler; Java da öyle, `lerpTo`). Passenger konumu mevcut `positionRider` kodu.

## Oracle
Mevcut `entity` ailesi (client-parity). Yeni satır gerekmiyorsa Java'da zaten karşılığı var; eksik kol varsa o dalı kapsayan satırı ekle.

## Kabul
`tools/photon.py check` → `CHECK OK`. Canlı: `shot` ile `summon pig` + `data merge entity @e[type=pig,limit=1] {NoAI:1}` gerekmez; `summon minecart ~ ~ ~ {Motion:[0.0,0.0,0.3]}` sonrası iki kare arasında konum değişimi log satırında.

## Kapsam dışı
Yerçekimi/AI/çarpışma (uzak entity'lerde Java'da da yok), çizim.

## Belgeler
`docs/remaining-work.md` M4.

## Devir notu
- Yapıldı: `ClientWorld::tick_entities` (`tick`'ten ayrıldı, yerel oyuncu hariç), `PlaySession::tick` her tikte çağırır, `EntitySnapshot`/`entity_snapshot(id, partial)` (konum, yaw, pitch, kafa yaw'ı: `getPosition(partial)` ve `rotLerp`), `ticking_entity_count`, oturum sonu `ticking=` satırı, iki birim test (tik sayısı ve iki tik arası ara konum).
- Canlı: `Fresh8` ile `entities=125 ticking=125` (yüklü yığındakiler tikler; yüklü olmayan yığındaki entity tiklemez, Java'daki gibi).
- Sapma: yok; tik kolları yalnız parity'li olanlar (mermiler, tekne, oyuncu, xp küresi…); oyuncu dışındaki mob'ların konum interpolasyonu yok (konum paketle anında değişir): T21b.
- Açık: uzak oyuncu çizimi (T22).
