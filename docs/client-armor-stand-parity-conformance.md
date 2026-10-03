# T40 — Zırh askısı — [client-parity]

Bu kayıt, zırh askısının (büyük ve küçük) çizimini kapsar: modelin sunucunun
yolladığı yedi veriyle (bayraklar ve altı dönüş) pozlanması, taban plakası, çubuklar,
kollar, giydiği zırh ve tuttuğu eşya. Kanıt sınıfı `client-parity`'dir: gerçek
`ArmorStandModel`/`ArmorStandArmorModel` katman tablosundan pişirilip gerçek `setupAnim`
ile pozlanır, her dörtgen bit bit karşılaştırılır; veri indeksleri Java erişimcilerinden
her koşuda karşılaştırılır. Zırh ve elde eşya kayıtları (`client-armor`, `client-held-item`)
ile aynı kanıtlara dayanır. Canlı gözlem (kollu, küçük, zırhlı, kılıçlı) `server-conformance`.

## Java kaynak incelemesi

- `ArmorStandArmorModel.setupAnim`: insansı poz, sonra baş, gövde, kollar ve bacaklar
  `Rotations` derecesinin radyana çevrilmişiyle (x, y, z) doğrudan yazılır.
  `ArmorStandModel.setupAnim`: ek olarak taban plakası `−yRot` kadar döner, kollar
  `showArms` ile, plaka `showBasePlate` ile görünür; üç çubuk gövdenin dönüşünü alır;
  şapka gizlidir.
- `ArmorStandRenderer`: `isBaby` = küçük (zırh modeli seti küçük), ama zırh dokusu katman
  türü `humanoid` (bebek değil) ve eşya elde bebek ofseti yok; `setupRotations` yalnız
  `180 − bodyRot` ile döner (`yBodyRot` hep `yRot`); gölge yarıçapı 0.
- `ArmorStand`: bayrak baytı 1 küçük, 4 kollar, 8 plakasız, 16 işaretçi; varsayılan
  dönüşler (kollar ve bacaklar hafif açılı).

## Veri

`ArmorStand.DATA_CLIENT_FLAGS` (15, bayt) ve altı `DATA_*_POSE` (16–21, `ROTATIONS`
serileştiricisi 9) `entity/variant-fields` satırlarıyla her koşuda karşılaştırılır.
Dönüşler her pakette yalnız değişenleri taşır: dünya durumu birleştirir
(`voxel-world/armor_stand.rs`).

## Kapsam

`entity/armor-stand-model` 40 satır (büyük ve küçük × ana model, dört zırh modeli × dört poz:
varsayılan, kollu, özel dönüşler ve dönmüş, plakasız).

## Sapmalar (açık)

- Vurulunca sallanma (`wiggle`, `EntityEvent` 32) çizilmiyor; işaretçi (`marker`) standın
  özel render türü (gölgesiz/saydam) yok, yalnız normal çizilir.
- Kafadaki eşya (kafatası, balkabağı) ve elytra T41c'de; eşya çerçevesi, tablo, bağ
  düğümü, kristal, TNT, düşen blok, yıldırım, display entity'leri T40b'de.
