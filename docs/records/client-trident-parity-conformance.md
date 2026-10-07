# T39b — Trident — [client-parity]

Bu kayıt, atılmış trident'in modelini ve yerleşimini Java 26.2 ile aynı sonucu vermesi için kapsar. Kanıt sınıfı
`client-parity`'dir: model gerçek `TridentModel`'e, yerleşim gerçek `ThrownTridentRenderer.submit`'e karşı karşılaştırılır. Büyü parıltısı
ve ışık oracle ile karşılaştırılmaz. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `TridentModel` (`Model<Unit>`): animasyonu yoktur; parçalar mesh'in koyduğu yerdedir.
- `ThrownTridentRenderer.submit`: `Y(yaw − 90)`, `Z(eğim + 90)` (model dik durur; okunki yatar), model `entityCutout` (arka yüz
  ayıklanmaz) ve `isFoil` ise bir de `entityGlint` katmanı. `yaw` ve `eğim` `getYRot(kısmiTik)`/`getXRot(kısmiTik)`.
- Hareket (sunucu) ve geri dönüş (`loyalty`) sunucunun gönderdiği konumlardır; titreme ve yerde olma verisi okla ortaktır (`AbstractArrow`).

## Kapsam

`entity/trident-model` 1 satır, `entity/trident-pose` 7 satır (okla aynı dönüş kümesi). Java'nın `order(n)` zinciri oracle'da
kendi toplayıcısına döner.

## Sapmalar (açık)

- Büyü parıltısı (`entityGlint`) çizilmez ([T41e](../tasks/T41e-zirh-susu-parilti.md)'nin parıltı hattı gelince); `ID_LOYALTY` ve `ID_FOIL` verisi
  okunmaz. Canlı: iki trident `summon` ile görüldü (yatay ve eğik).
