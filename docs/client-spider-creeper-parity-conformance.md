# T33 — Örümcek, mağara örümceği ve creeper — [client-parity]

Bu kayıt, örümceğin, mağara örümceğinin (sekiz bacak yürüyüşü, ölürken yarım tur
yatma) ve creeper'ın (bacak sallanması, fitil, şişme ve beyaz yanıp sönme) Java 26.2
ile Rust'ta aynı köşe konum, doku koordinatı ve normali ve aynı poz/ölçek/üst katman
değerlerini üretmesini kapsar. Kanıt sınıfı `client-parity`'dir: gerçek model sınıfları
(`SpiderModel`, `CreeperModel`) katman tablosundan pişirilip gerçek `setupAnim` ile
pozlanır ve her dörtgen bit bit karşılaştırılır; ölçek ve beyaz yanıp sönme gerçek
`CreeperRenderer`'ın `scale`/`getWhiteOverlayProgress` yöntemlerinden, fitil gerçek
`Creeper.tick`'ten (gerçek `ClientPacketListener` ve entity verisi paketiyle) okunur.
Canlı gözlem (mağara örümceği, creeper, şişmiş creeper) `server-conformance`.

## Java kaynak incelemesi

- `SpiderModel.setupAnim`: baş bakışla; her bacak çifti (`phase` 0, π, π/2, 3π/2) yürüyüş
  fazına göre y ekseninde `−cos(2·faz)·0,4·hız` ve z ekseninde `|sin(faz)·0,4|·hız`
  kadar kayar; sağ ve sol bacaklar ters işaretli (`+=`/`−=`, ilk pozdan).
  `SpiderRenderer.getFlipDegrees` 180. Mağara örümceğinin 0,7 ölçeği model katmanına
  pişirilidir (ayrı ölçek kancası yok).
- `CreeperModel.setupAnim`: bacak adları ve alanları ters bağlıdır (`leftHindLeg` =
  `right_hind_leg` parçası); sallanma çaprazdır.
- `CreeperRenderer.scale`: `wobble = 1 + sin(100·s)·s·0,01`; `g = clamp(s,0,1)⁴`;
  `(1+0,4g)·wobble`, `(1+0,1g)/wobble`, `(1+0,4g)·wobble` ölçeği, `LivingEntityRenderer`'ın
  ters çevirme ölçeğinden sonra uygulanır. `getWhiteOverlayProgress`: `(int)(s·10)` çiftse
  0, değilse `clamp(s,0.5,1)`. `OverlayTexture`: beyaz satırda sütun `(int)(ilerleme·15)`,
  beyazın saydamlığı `(1 − sütun/15·0,75)·255`.
- `Creeper.tick`: yaşıyorsa `oldSwell = swell`; ateşliyse yön 1; `swell += yön` en az 0, en
  çok `maxSwell` (30); `getSwelling = lerp(oldSwell, swell)/(maxSwell−2)`.

## Veri

`Creeper.DATA_SWELL_DIR` (16, int) ve `DATA_IS_IGNITED` (18, boolean) veri indeksleri Java
erişimcilerinden `entity/variant-fields` satırlarıyla her koşuda karşılaştırılır. Overlay
kodu (`overlay_code`): köşe `shade` değeri 1 = hurt kırmızısı, 2·sütun = beyaz yanıp sönme;
shader her ikisini overlay dokusundaki gibi karıştırır.

## Kapsam

`entity/spider-model` 10 satır (iki katman × beş poz), `entity/creeper-model` 5 satır,
`entity/creeper-render` 12 satır (şişme 0'dan 30/28'e), `entity/living-pose`'a dört örümcek
ölüm satırı (flip 180), `movement/living-entity-animations`'a beş creeper satırı (sessiz,
fitil, fitil söndü, ateşli, fitil yanarken ölüm).

## Sapmalar (açık)

- Örümceğin parlayan gözleri (`SpiderEyesLayer`, emissive) ve şarjlı creeper'ın enerji
  girdabı (`CreeperPowerLayer`, `EnergySwirlLayer`) çizilmiyor: yeni render türleri gerekir (T33b).
- `Creeper.DATA_IS_POWERED` okunmuyor; `maxSwell` istemcide hep 30 (sunucunun `Fuse`'u gelmez,
  Java'da da öyle).
- Creeper'ın patlama parçacığı/sesi ve ateşleme etkileşimi kapsam dışı.
- Gümüş balığı, endermite, arı T33b'de.
