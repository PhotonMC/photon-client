# T27 — Mob hasar kırmızısı ve ölüm yatışı

**Amaç:** Vurulan canlı kırmızı örtüyle çizilsin (`hurtTime`), ölen canlı yan yatsın (`deathTime`). Kullanıcı gözlemi (T26 sonrası canlı oyun): vurulan mob kırmızı olmuyor, ölen mob yatmıyor. Kırmızı örtü parçacıkla ilgili değil: çizicinin overlay dokusudur (`LivingEntityRenderer.getOverlayCoords`, `hasRedOverlay = hurtTime > 0 || deathTime > 0`); vuruş parçacığı (hasar kıvılcımı) ayrıdır ve PLAN-M8'e (parçacık motoru) aittir.
**Bağımlılık:** T26 (vurma), T24 (entity çizim iskeleti). **Boy:** M.

## Oku
- Java: `world/entity/LivingEntity.java` (`hurtTime`/`hurtDuration` 230–232, `tick` 466–475, `tickDeath` 571, `handleEntityEvent` 2065–2085: olay 2 vuruş, 3 ölüm; `animateHurt` ~1877, `handleDamageEvent` ~2054), `client/renderer/entity/LivingEntityRenderer.java` (`getOverlayCoords` 133, `setupRotations` 160–175 ölüm yatışı `getFlipDegrees`, `extractRenderState` 281/297), `client/renderer/texture/OverlayTexture.java`, `network/protocol/game/ClientboundHurtAnimationPacket.java`, `ClientboundDamageEventPacket.java`, `ClientboundEntityEventPacket.java`; istemci işleyicileri `ClientPacketListener.handleHurtAnimation/handleDamageEvent/handleEntityEvent`.
- Rust: `crates/voxel-net/src/connection/entities.rs` (paket uygulama), `crates/voxel-world/src/living.rs` ve `entity.rs` (`tick_entities`, ölüm/sağlık durumu), `crates/voxel-render/src/mob_model.rs`, `player_renderer.rs`, `crates/voxel/src/mobs.rs`, `overlays.rs` (çizim; overlay dokusu ve yatış matrisi), `vulkan.rs` entity shader'ı (overlay rengi).

## Yapılacaklar
1. Paketler: `EntityEvent` (2, 3 ve oyuncu dışı `setHealth(0)`), `HurtAnimation`, `DamageEvent`; `SetEntityData` sağlığı ile ölü-mü durumu. `hurtTime`/`deathTime` istemci tikinde Java gibi azalır/artar (`LivingEntity.tick`/`tickDeath`); ölüm 20 tikte biter.
2. Çizim: kırmızı örtü (overlay dokusunun v koordinatı, Java'daki `OverlayTexture` renk karışımı: vurulunca/ölürken kırmızı), `setupRotations`'taki ölüm yatışı (`sqrt((deathTime-1)/20*1.6)` sınırlı 1, `getFlipDegrees` türe göre 90 veya örümcekte 180).
3. Oracle: `deathTime`/`hurtTime` tik ilerlemesi gerçek `LivingEntity`'ye karşı (T21 mob oracle kalıbı); yatış matrisi gerçek `setupRotations` ile (T22/T24 çizim oracle kalıbı).

## Oracle
Tik ilerlemesi `client-parity` (paket → `hurtTime`/`deathTime` dizisi); yatış ve örtü rengi çizim oracle'ı. Oracle kurulamayan kısım varsa nedeni devir notuna.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): `summon pig`, sol tık: domuz kırmızı; `kill`/öldürücü vuruşta domuz yatar (ekran görüntüsü bir kez okunur).

## Kapsam dışı
Vuruş/ölüm parçacıkları (PLAN-M8), sesler (PLAN-M7), tepme hareketi (sunucu konumu zaten gönderir), oyuncu ölüm ekranı (var).

## Gözlem: kaçış süresi (kullanıcı, doğrulanmadı)
Kullanıcı, vurulan mobların eskisinden kısa süre kaçtığını fark etti ("vanillada güncellemeyle değişmiş olabilir ama sanmıyorum"). Kaçış (`PanicGoal`) sunucu yapay zekâsıdır; istemci yalnız konum paketlerini gösterir. Olası istemci kaynakları, ölçülmeden hükme varılmaz: (a) bizim `Attack` yoğunluğu/zamanlaması sunucunun saldırı gücü ve tepmesini değiştirir (kılıçsız, sprint paketi yok), (b) `EntityPositionSync`/göreli hareket uygulamasında interpolasyon bozuklukları mobu yanlış yerde gösterebilir, (c) vanilla istemciyle aynı sunucuda yan yana karşılaştırma yapılmadı. Bu görevde yalnız ölçüm: aynı sunucuda Java istemcisiyle ve bizimkiyle aynı vuruş, mobun konum izlerini (sunucu `data get entity ... Pos`) karşılaştır; fark sunucu tarafında çıkarsa görev kapatılır.

## Belgeler
`docs/remaining-work.md` M4 (paketler satırı: `EntityEvent`, `HurtAnimation`, `DamageEvent` kalanından düş).

## Devir notu
- Yapıldı: `ClientEntity.hurt_time/death_time` ve tik (`tick_hurt_and_death`, `shouldTickDeath` = oyuncu chunk'ına simülasyon mesafesi; `Login`/`SetSimulationDistance` artık dünyada), `DamageEvent`/`HurtAnimation`/`EntityEvent 3` (kimlik 25/42/111 simülasyon), `WalkAnimation::set_speed`, entity shader'ında kırmızı örtü (`Vertex.shade` entity boru hattında örtü bayrağı), ölüm yatışı (`mob_model::death_roll`, mob ve oyuncu). Kayıt: `docs/client-entity-hurt-death-parity-conformance.md`.
- Oracle: `movement/living-entity-ticks` 12 satır (gerçek `ClientLevel` + paket işleyicileri), `entity/living-pose` 23 satır (gerçek `LivingEntityRenderer.submit`). Canlı: vurulan domuz kırmızı, 1 canlı domuz öldürücü vuruşta yatık ve kırmızı (ekran görüntüleri okundu).
- Sapma: `getFlipDegrees` yalnız 90 (örümcek vb. 180, çizilince tablo gerekir); örtü rengi piksel oracle'sız (kural + canlı ekran); `die()` yan etkileri, ses ve parçacık yok.
- Açık: kaçış süresi gözlemi ölçülmedi (sunucu AI'sı; yukarıdaki plan). Bu görev kapanırken ayrı kayıt gerekir: aynı sunucuda Java istemcisiyle karşılaştırma.
