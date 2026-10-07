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

- Taşıdığı blok T37b2'de eklendi (aşağıda); arketipler hâlâ yok.
- Görünmezlikte bir piksellik yukarı kaydırma sıfırlanmaz (görünmezlik hiçbir mob'ta gözetilmiyor).
- Fitil beyaz bloğu (`TntMinecartRenderer.submitWhiteSolidBlock`) bu mob'da yoktur; yalnız iç gövde
  beyazlar. Ses, parçacık, patlama yok.

## Taşıdığı blok (T37b2) — [client-parity]

- `SetEquipment` BODY yuvası (sıra 6) zaten saklanır; çizici bu yuvadaki eşyadan
  `Block.byItem(item).defaultBlockState()` kurar ve eşyanın `block_state` bileşenini uygular (bakır
  golemin anteniyle ortak `block_item_state`). Blok eşyası olmayan eşya hava verir, yani boş sayılır.
- Blok doluysa `applySizeAndSquish` squish'i sıfırlar (`ss = 0`); boyut ölçeği kalır.
- `SulfurCubeInnerLayer` blok dalı: `XP` 180°, küçükte `0,5` ölçek, sonra `(-0,5, -0,518, -0,5)`;
  blok varken iç gövde çizilmez. Poz, gerçek katmanın `submitBlockModel`'e verdiği matrisle ve
  gerçek `scale` kancasının bıraktığı pozdan karşılaştırılır (`cube-mob-rotations`, 8 satır:
  yetişkin/küçük × fitil 0/6,25).
- Kaplama (T37b3): `SulfurCubeInnerLayer` bloğu iç gövdenin kaplamasıyla verir (fitil yanıkken
  beyaz, aksi hâlde varlığın hasar kırmızısı); `submitBlockModel`'e giden `overlayCoords`
  oracle'da yakalanır, `overlay_coords` ve `sulfur_cube_overlay` ile 4 satırda karşılaştırılır.
  Aynı kaplamayı kar golemi balkabağı ve mooshroom mantarı da alır; enderman, demir golem çiçeği ve
  bakır golem anteni `NO_OVERLAY` verir. Taşıyıcı: blok köşelerinin kaplama kodu shade'in üstünde
  (`shade + 2·kod`); yer/blok hattının shader'ı kaplamayı ışık haritasından önce uygular. Opak
  yer/blok hattı entity'lerden önce çizilir (yarı saydam kabuk bloğun üstüne karışır).
