# T35b — Kalamar ve ışıklı kalamar — [client-parity]

Bu kayıt, kalamarın ve ışıklı kalamarın (yetişkin ve bebek) modelinin, çizim dönüşlerinin, istemci
tikinin (dokunaç vuruşu, gövde dönüşleri, suda olma durumu) ve ışıklı kalamarın kendi ışığının
Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt sınıfı `client-parity`'dir: model gerçek
`SquidModel` ve `BabySquidModel`'e, dönüşler gerçek `SquidRenderer.setupRotations`'a, tik gerçek
`Squid` ve `GlowSquid` varlığına karşı karşılaştırılır; ışık düzeyi gerçek `Mth.clampedLerp`'e
karşı. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `SquidModel.setupAnim`: sekiz dokunacın hepsi `tentacleAngle` kadar eğilir. Bebek modeli aynı
  sınıftır, yalnız katmanı küçüktür.
- `SquidRenderer.setupRotations` (üst sınıfınkini çağırmaz: ölüm düşüşü ve titreme yok): `0,5`
  (bebekte `0,25`) yukarı, `180° - gövde dönüşü` y, `xBodyRot` x, `zBodyRot` y, sonra `-1,2`
  (bebekte `-0,6`) aşağı. Üç değer kısmi tikle ara değer alır.
- `Squid.aiStep` (istemci): dokunaç hareketi her tik `tentacleSpeed` ilerler ve `2π`'de kalır
  (sunucu döngüye sokar, istemci sıfırlamaz; olay 19 sıfırlar). Suda: yarısına dek (`< π`)
  `sin(ölçek²·π)·π/4` açısı, ölçek `0,75`'in üstündeyse dönme hızı 1, değilse `·0,8`; sonrasında
  açı sıfır, hız `·0,99`; gövde yönü hareketin `atan2(x, z)`'ine onda bir yaklaşır ve varlığın
  dönüşü olur, `zBodyRot` `π·hız·1,5` artar, `xBodyRot` `atan2(yatay, y)`'ye onda bir yaklaşır.
  Karada: açı `|sin|·π/4`, `xBodyRot` `-90°`'a yüzde iki yaklaşır. Hareket verisi (sunucudan) kullanılır.
- `LivingEntity.aiStep` (her yaşayan varlık, istemcinin kendisinin oynatmadığı): konum
  ara değerlemesi sürmüyorsa hareket `0,98` ile azalır, ve her ekseni üç binde birin altındaysa sıfırlanır.
  Bu, kalamarın dönüşünü sürer; ilk kez bütün mob'lar için yazıldı.
- `Entity.baseTick`: suda olma durumu (`wasTouchingWater`) kutunun suya değmesiyle hesaplanır
  (kutu: `EntityPick` boyutu, bebek ve ölçekle); kalamar suyun akıntısıyla itilmez
  (`AgeableWaterCreature.isPushedByFluid` yanlış).
- `GlowSquid`: karanlık tik verisi (18, int) her tik bir azalır; renderer ışığı
  `clampedLerp(1 - tik/10, 0, 15)` ile verir, 15 ise 15, değilse çevre ışığıyla büyüğünü.

## Kapsam

`entity/squid-model` 16 satır (dört katman × dört açı); `entity/squid-rotations` 8 satır (yetişkin
ve bebek, dört dönüş); `entity/glow-squid-light` 24 satır; `living-entity-animations`'a 10 satır
(derin, sığ ve susuz; hareket ve konum paketleri; olay 19; ışıklı kalamar ve karanlık tikler).
Oracle'ın başlangıç yönü artık paketteki gibi sabittir (kurucunun rastgele baş dönüşü yerine).

## Sapmalar (açık)

- Suda olma yalnız kalamar için izlenir; öteki su canlıları T35c ve sonrasında ekler (`tracks_water`).
  Akıntıyla itilen mob'lar için akıntı henüz hareketine uygulanmaz.
- Kalamarın dokunaç hızı kurucuda tohumsuz çekilir (referans da öyle); karşılaştırmada sabitlenir
  (`fix_squid_tentacle_speed`), üretimde saat. Mürekkep parçacığı ve sesler yok.
- Işıklı kalamarın ışığı yalnız `GlowSquidRenderer` mantığıdır; `BlockPos` ışığı çizimde bakılan yerden
  alınır (diğer mob'larla aynı).
- Gölge çizilmez (yalnız domuz, inek, eşya ve orb gölgesi var).
