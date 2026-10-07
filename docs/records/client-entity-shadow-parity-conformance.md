# T42 — Entity gölgesi — [client-parity]

Bu kayıt, bir entity'nin altına düşen gölge parçalarının (`EntityRenderer.extractShadow`)
ve her parça için yazılan dörtgenin (`ShadowFeatureRenderer`) Java 26.2 ile Rust'ta
aynı olmasını kapsar. Kanıt sınıfı `client-parity`'dir: aynı sahne Java'nın gerçek
`ClientLevel`'ında ve gerçek ışık motoruyla (istemcinin yüklenen chunk'ları
aydınlatışı) kurulur, gerçek `extractShadow` ve `prepare` yöntemleri çağrılır;
parçalar (göreli konum, altındaki şeklin sınırları, alfa) ve köşeler (konum,
renk, doku koordinatı) bit bit karşılaştırılır. Canlı kabul (inek, domuz ve
eşyaların altında gölge) `server-conformance` gözlemidir; yarı saydam karışımın
kendisi oracle'da yok.

## Java kaynak incelemesi

- `client/renderer/entity/EntityRenderer.java:274-325`: seçenek açık ve entity
  görünürse yarıçap `min(getShadowRadius, 32)`; güç `(1 - mesafe²/256) *
  getShadowStrength` ve pozitif olmalı; x/z aralığı yarıçapın iki yanı, y aralığı
  `min(güç/0.5 - 1, yarıçap)` aşağıdan ayak seviyesine. Her hücre için: altındaki
  blok `RenderShape.INVISIBLE` değilse, hücrenin `getMaxLocalRawBrightness`'ı 3'ten
  büyükse ve altındaki bloğun çarpışma şekli tam bloksa (`isCollisionShapeFullBlock`)
  ve şekli boş değilse parça eklenir; alfa `(güç - derinlik*0.5) * 0.5 *
  Lightmap.getBrightness` (0..1).
- `client/renderer/feature/ShadowFeatureRenderer.java:29-60`: her parça, hücrenin
  yerinden şeklin sınırlarıyla bir dörtgen; doku koordinatı `-x / 2 / yarıçap + 0.5`;
  renk beyaz, alfa `floor(alfa*255)`. `ENTITY_SHADOW` ardışık ışıksız, yarı saydam,
  derinliğe yazmayan boru hattıdır; doku kenarda kıskaç (clamp) ile örneklenir.
- Yarıçaplar: `MobRenderer` kendi yarıçapını `ageScale` ile çarpar (bebek yarısı);
  domuz ve inek 0.7, oyuncu 0.5, yerdeki eşya ve deneyim küresi 0.15 (güç 0.75).
- `RenderShape.INVISIBLE` olan bloklar: hava (üç tür), bariyer, kabarcık sütunu, son
  geçidi/portalı, ışık bloğu, su, lav, yapı boşluğu, hareketli piston.

## Kapsam ve dal kararı

`movement/entity-shadows` 51 satır: düz zemin; üç yarıçap; blok köşesi ve sınırı;
yükseklik (yarım, bir, iki, çok yüksek, zayıf güç); kameraya mesafe (64, 200, 255,
256, 300: güç sıfıra iner); görünmez ve seçenek kapalı; çatı altı (ışık azalır) ve
yüksek çatı; meşale ve fener yanı; zeminsiz ve çok derin; ve altındaki blok olarak 27
farklı şekil (yarım/üst/çift basamak, merdiven, halı, çit, cam, buz, ruh kumu, patika,
tarla, bal, bariyer, su, lav, kar, kazan, kaktüs, sandık, yaprak, ışık, kapak, kapı,
piston, demir parmaklık, bedrock). Şekil tablosu aynı koddan geçtiği için satırdır.
`entity/` dışı: `movement` ailesinde çünkü gerçek `ClientLevel` ister.

## Sapmalar (açık)

- `getSkyDarken` (`gameplay/sky_light_level` öznitelik değeri) taşınmadı: gece
  gölgeler gündüz kadar koyu çizilir (Java'da gökyüzü ışığı 15'ten düşer).
- Yarıçaplar yalnız çizilen türler için (domuz, inek, oyuncu, eşya, küre);
  `getShadowRadius` geçersiz kılan türler (strider, kaplumbağa, köylü, balık,
  slime, display) kendi görevlerinde.
- Gölge dörtgenleri doku sınırına kırpılır (kenar texelleri saydam olduğundan
  Java'daki kıskaçlı örnekleme ile aynı sonuç); atlas içi örnekleme için gerekli.
- Yerel oyuncunun kendi gölgesi yalnız üçüncü şahıs kamerada çizilir ve oracle'da
  yok (aynı kod).
