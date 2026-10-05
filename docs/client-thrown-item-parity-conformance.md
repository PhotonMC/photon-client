# T39c — Fırlatılan eşyalar ve havai fişek — [client-parity]

Bu kayıt, kameraya dönük eşya olarak çizilen varlıkların (kartopu, yumurta, ender incisi, ender gözü, iksirler, büyü şişesi, ateş ve
küçük ateş topu, havai fişek roketi) yerleşimini Java 26.2 ile aynı sonucu vermesi için kapsar. Kanıt sınıfı `client-parity`'dir:
yerleşim gerçek `ThrownItemRenderer.submit` ve `FireworkEntityRenderer.submit`'e karşı karşılaştırılır; eşyanın kendi modeli ve
ışık oracle ile karşılaştırılmaz (eşya modelleri kendi kayıtlarında). Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `ThrownItemRenderer.submit`: `scale(ölçek)`, sonra `camera.orientation` (kameranın dönüşü: eşyanın düz yüzü göze bakar); ardından
  eşya durumu (`ItemStackRenderState`) her katmanı kendi görüntü dönüşümüyle (`ItemTransform`, `GROUND` bağlamı) çizer. Ölçek:
  ateş topu 3, küçük ateş topu 0.75, diğerleri 1; ateş topları ve ender gözü tam parlak (blok ışığı 15). Eşya, varlığın verisidir
  (`DATA_ITEM_STACK`, veri 8: `ThrowableItemProjectile`, `Fireball`, `EyeOfEnder`), türün varsayılanı sunucunun gönderdiğidir.
- `FireworkEntityRenderer.submit`: `camera.orientation`; çapraz yaydan atılmışsa (`DATA_SHOT_AT_ANGLE`, veri 10) `Z(180)`, `Y(180)`,
  `X(90)` ile yan yatırılır.
- `Camera.setRotation`: yön `rotationYXZ(π − yaw, −pitch, 0)`; oyun bunu isim etiketleri için de kullanır.

## Kapsam

`entity/thrown-item-pose` 12 satır (kameranın dört yönü, tepeden ve alttan, çapraz; ateş topu ve küçüğü; uzak konum; roket ve çapraz
yaydan atılan roket). Java'nın boş katmanı `ItemTransform.NO_TRANSFORM` ile `(−0.5, −0.5, −0.5)` kayması ekler; karşılaştırma bunu
Rust tarafına aynen ekler (eşya katmanlarının kendi dönüşümü başka kayıtlarda).

## Sapmalar (açık)

- Parıltı, ışık ve eşya modelinin kendisi oracle'sız; ender incisi ve potions için eşyanın bileşenleri (renkli iksir) eşya
  çiziminin kapsamıdır. Ejderha topu, rüzgâr yükleri ve diğerleri [T39d](tasks/T39d-atesler-ruzgar-yukleri.md)'de. Parçacıklar ve sesler yok.
  Canlı: kartopu, inci, ateş topu, küçük ateş topu ve iksir `summon` ile görüldü.
