# T27 — Canlı entity vuruş ve ölüm geri bildirimi (`hurtTime`, `deathTime`, kırmızı örtü, yatış) — [client-parity]

Bu kayıt, sunucunun bir `LivingEntity`'yi yaralayıp öldürdüğünü bildiren
paketlerin (`DamageEvent`, `HurtAnimation`, `EntityEvent` 3) istemcide aynı
sayaçları, aynı yürüme animasyonunu ve aynı çizim pozunu üretmesini kapsar.
Kanıt sınıfı `client-parity`'dir: tik ilerlemesi Java'nın gerçek `ClientLevel`,
gerçek `Pig`/`Cow`/`Zombie`/`Spider` entity'si ve gerçek paket işleyicileriyle,
yatış pozu gerçek `LivingEntityRenderer.submit` ile (bkz. oracle) karşılaştırılır.
Kırmızı örtünün rengi için oracle yok (aşağıda).

## Java kaynak incelemesi

- `world/entity/LivingEntity.java:466-476`: her tikte `hurtTime` bire iner;
  sağlığı bitmiş (`isDeadOrDying`) entity'de ve `level.shouldTickDeath` doğruysa
  `tickDeath` `deathTime`'ı artırır. `client/multiplayer/ClientLevel.java:463`:
  `shouldTickDeath`, entity'nin chunk'ının oyuncunun chunk'ına satranç
  uzaklığının sunucu simülasyon mesafesini (`Login` ve `SetSimulationDistance`)
  aşmamasıdır; uzakta ölen mob yatmaz.
- `:1877`, `:2054`: `animateHurt` ve `handleDamageEvent` `hurtTime`'ı 10 yapar;
  hasar olayı ayrıca yürüme hızını anında 1.5'e çeker (`walkAnimation.setSpeed`).
  `:2065-2085` olay 3 (oyuncu dışı): sağlığı sıfırlar ve `die` çağırır. Hasar
  paketi `Entity` kimliğiyle başlar, kaynağın geri kalanı çizime etki etmez.
- `client/renderer/entity/LivingEntityRenderer.java:164-171`:
  `deathTime > 0` iken `sqrt((deathTime - 1) / 20 * 1.6)` en çok 1, `getFlipDegrees`
  (varsayılan 90, örümcek/gümüş balığı/endermite 180) ile z ekseninde döner;
  dönüş yaw'dan sonra, `scale(-1,-1,1)`'den önce uygulanır. `:281`, `:297`:
  `hasRedOverlay = hurtTime > 0 || deathTime > 0`, `deathTime` kesirle birlikte
  (`+ partialTicks`) çizime gider.
- `client/renderer/texture/OverlayTexture.java`, `shaders/core/entity.fsh`:
  kırmızı örtü, dokunun yüzey ışığından sonra, ışık haritasından önce
  `mix(kırmızı, renk, 0xB2/0xFF)` ile karışmasıdır.

## Kapsam ve dal kararı

- `movement/living-entity-ticks` (12 satır): sessiz, hasar, vuruş animasyonu,
  iki vuruş (sayaç yeniden 10), ölüm, hasar+ölüm, ölüm+hasar, yürürken hasar,
  örümcek ölümü, oyuncu uzakta / simülasyon sınırında / hemen dışında ölüm.
  Her tik `hurtTime`, `deathTime`, ölü mü, yürüme hızı ve konumu karşılaştırılır.
- `entity/living-pose` (23 satır): `deathTime` (kesirli), yön ve ölçek için
  gerçek `submit`'in ürettiği 4×4 matris.
- Kırmızı örtünün rengi ve ölüm sırasında örtünün sürmesi (`hasRedOverlay`)
  kuraldır; shader'daki karışım sabitleri Java kaynağından okunmuştur ve canlı
  ekran görüntüsüyle (vurulan domuz kırmızı, ölen domuz yatık ve kırmızı)
  doğrulandı; piksel karşılaştırmalı çizim oracle'ı yok.

## Sapmalar (açık)

- `getFlipDegrees` yalnız 90 (domuz, inek); örümcek, gümüş balığı ve endermite
  (180) çizimi gelince mob başına tablo gerekir.
- Oyuncu için yatış aynı işlevi kullanır; oyuncu pozu için ayrı oracle yok
  (oyuncu `Pig` gibi kurulamaz). Yerel oyuncunun kendi ölümü ekran olarak var.
- Ölümde `die()`'ın istemci yan etkileri (duruş `DYING`, ses, parçacık) yok.
- `shouldTickDeath` oyuncunun chunk'ına bakar; canlı oturumda dünyadaki
  `local_player` kaydının konumu güncellenmez (yürütmeyi `PlaySession.player`
  yapar), bu yüzden oturum her tikte `set_local_player_chunk` ile gerçek
  konumu verir. Başlangıçta bu bilinmez ve ölüm tikler. Aynı bayat kayıt başka
  yerlerde okunuyorsa (`entity_position` yardımcıları) aynı sorunu taşır.
- Ölü entity'nin `alive` bayrağı sağlıktan türetilir (`isAlive`); sağlık
  `SetEntityData`'dan gelir. Sağlığı hiç gönderilmemiş entity canlı sayılır.
