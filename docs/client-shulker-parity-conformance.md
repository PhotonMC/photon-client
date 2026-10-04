# T37c — Shulker — [client-parity]

Bu kayıt, shulker'ın modelinin (kapak kalkışı, bükülme, sallanma, baş dönüşü), çizim dönüşlerinin
(gövdenin yarım dönüşü ve tutunduğu yüze göre döndürme), kapak tikinin ve veri alanlarının
(tutunma yüzü, kapak açıklığı, renk) Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt sınıfı
`client-parity`'dir: model gerçek `ShulkerModel`'e, dönüşler gerçek `ShulkerRenderer.setupRotations`'a,
tik gerçek `Shulker` varlığına karşı karşılaştırılır. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `ShulkerModel.setupAnim`: `bs = (0,5 + açıklık)·π`; kapak `y = 16 + sin(bs)·8 + ekstra`, ekstra yalnız
  `bs > π` iken `sin(yaş·0,1)·0,7`; açıklık `0,3`'ün üstündeyse kapak `y` dönüşü
  `(sin(bs) - 1)⁴·π·0,125`; kafa `xRot` pitch, `yRot` `(başDönüşü - 180 - gövdeDönüşü)` radyan.
  Baş ve gövde dönüşü ara değerlenmeden varlığın kendi alanlarıdır.
- `ShulkerRenderer.setupRotations`: üst sınıfı `gövde + 180` ile çağırır, sonra tutunduğu yüzün
  tersinin döndürmesini `(0, 0,5, 0)` etrafında uygular (`Direction.getRotation`).
- `Shulker.tick`: kapak her tik `0,01·ham` hedefine yirmide bir yaklaşır (`updatePeekAmount`).
- `ShulkerBodyRotationControl.clientTick` boştur: gövde başı izlemez; paketten yaratılınca
  `yBodyRot = 0` (güney) olur. Dönüş değerleri yine de yarım tur içinde tutulur.
- Veri 16 (yön, serileştirici 12), 17 (bayt, ham açıklık), 18 (bayt, renk; 16 renksiz).
- Doku `entity/shulker/shulker[_<boya>]`, renk `DyeColor` kimliğiyle.

## Kapsam

`entity/shulker-model` 21 satır (açıklık × baş dönüşü); `entity/shulker-rotations` 13 satır (altı yüz ×
iki gövde dönüşü; ölen shulker); `living-entity-animations`'a 6 satır (açılma, kısmi açılma, açılıp
kapanma, duvarda renkli, renksiz, etrafa bakma). Oracle'a `shulker_data` olayı ve `shulker` ayrıntısı
eklendi. `Pose::rotate_around` JOML kaynağındaki sıraya (kaynaşık çarpma-toplama yok) çekildi.

## Sapmalar (açık)

- Konum yakalama (`setPos`'ın blok merkezine oturtması), blok değişince açıklığın sıfırlanması ve
  6 tiklik ışınlanma kaydırması (`getRenderPosition`) ile açıklığa bağlı vuruş kutusu yok: T37c2.
- Render türü `entityCutoutZOffset`'in derinlik kayması yok (kapak ile taban aynı düzlemde değil,
  fakat ofsetli çizim yapılmıyor).
- Ses, parçacık, mermi (`ShulkerBullet`) yok.
