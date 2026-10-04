# T37b — Sulfur cube — [client-parity]

Bu kayıt, sulfur cube'un (yetişkin ve küçük) ölçek zincirinin (boyut, squish, patlamak üzere olan
fitilin şişmesi, kabuğun oturması), fitil tikinin ve çizim katmanlarının Java 26.2 ile aynı
sonucu vermesini kapsar. Kanıt sınıfı `client-parity`'dir: ölçek gerçek
`SulfurCubeRenderer.scale`'e, tik gerçek `SulfurCube` varlığına karşı karşılaştırılır. Canlı gözlem
`server-conformance`. Slime ve magma cube'un ortak tabanı [T37a kaydındadır](client-cube-mob-parity-conformance.md).

## Java kaynak incelemesi

- `SulfurCubeRenderer.scale`: slime gibi `downscaleSlightly`, `applySizeAndSquish`, sonra fitil
  `0 < f < 10` ise `1 + swell(f)` ölçeği (`TntRenderer.getSwellAmount`: `((1 - f/10) kıstırılmış)⁴·0,3`),
  sonra yetişkinde `0,5` (küçükte `1`) ölçek ve `(0, ofset - 1/16, 0)` kaydırması (ofset yetişkinde
  `0,98`, küçükte `1,24`; `-0,0F` değerleriyle). Kaydırma `-1,501`'den önce, ölçeklenmiş uzayda.
- Çizim durumu: `fuseRemainingTicks = primed ? fuse - kısmi + 1 : 0`; küçük model bebek bayrağına bağlı.
- `SulfurCubeInnerLayer` (`order(-1)`, kabuktan önce): iç gövde, fitil yanıksa (`TntRenderer.isLit`:
  her beş tikte bir) beyaz kaplama 1,0, değilse olağan kaplama (kırmızı); kabuk `entityTranslucent`.
  Kabuk ve iç gövde ayrı ayrı uzaktan yakına sıralanır.
- `SulfurCube.tick`: fitil önce (`fuse > 0` ise bir azalır; istemcide patlama verisi olmadığından
  sıfırda kalır ve `isPrimed` doğru kalır), sonra `AbstractCubeMob.tick` (squish 0,6 sönmesi).
- `MAX_FUSE` verisi (18, int) `setFuse`'u çağırır. Boyut verisi (17) T37a'daki gibidir.

## Kapsam

`entity/cube-mob-rotations`'a 36 satır (yetişkin boyut 2 ve 3, küçük, iki squish, altı fitil
değeri; gerçek `scale` kancasıyla tam poz); `living-entity-animations`'a 3 satır (sıçrayış,
fitilin sıfıra inmesi, küçük). Katman tabloları (T23) her koşuda karşılaştırılır.

## Sapmalar (açık)

- Taşıdığı blok (`getBodyArmorItem`: gövde ekipmanı, blok modeli, squish'in sıfırlanması) ve
  arketipler T37b2'dedir; bu yüzden squish her zaman uygulanır.
- Görünmezlikte bir piksellik yukarı kaydırma sıfırlanmaz (görünmezlik hiçbir mob'ta gözetilmiyor).
- Fitil beyaz bloğu (`TntMinecartRenderer.submitWhiteSolidBlock`) bu mob'da yoktur; yalnız iç gövde
  beyazlar. Ses, parçacık, patlama yok.
