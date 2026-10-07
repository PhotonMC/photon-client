# T43b, T30c — Anahtar kare animasyonu ve tavşan — [client-parity]

Bu kayıt, anahtar kare animasyonlarının oynatıcısını (`KeyframeAnimation`, `AnimationState`)
ve ilk tüketicisi tavşanı (modeller, zıplama ve boşta baş eğme, `Rabbit` istemci tiki)
kapsar. Kanıt sınıfı `client-parity`'dir: bütün tanımlar (74) gerçek `definitions` sınıflarından
üretilir ve her koşuda karşılaştırılır; oynatıcı gerçek `KeyframeAnimation.apply`'a karşı
bit bit; `AnimationState` gerçek sınıfa karşı; tavşan modeli gerçek `RabbitModel.setupAnim`'e
karşı; tavşanın hop/boşta durumları gerçek `Rabbit` varlığının tikiyle karşılaştırılır.
Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `AnimationDefinition`: uzunluk, döngü, kemik başına kanallar (`HashMap` sırasıyla); kanal bir
  hedef (konum, dönüş, ölçek) ve anahtar karelerdir; anahtar kare zaman, öncesi ve sonrası
  vektör, geçiş (doğrusal, Catmull-Rom). Vektörler parçanın birimindedir (konum y ters, dönüş
  radyan, ölçek bir eksik).
- `KeyframeAnimation.apply(millis, scale)`: saniye = milisaniye / 1000, döngüdeyse uzunluğa
  göre kalan; her kanal ikili aramayla önceki kareyi bulur, aradaki oranı `[0, 1]`'e sıkıştırır,
  sonraki karenin geçişiyle vektörü hesaplar (doğrusal: öncekinin "sonrası"ndan sonrakinin
  "öncesi"ne `Vector3f.lerp` (kaynaştırmasız), Catmull-Rom: dört "sonrası" noktası) ve hedef
  ölçekle çarpar; sonuç parçanın konumuna, dönüşüne ya da ölçeğine eklenir
  (`offsetPos/Rotation/Scale`). Parça adı kökten ve çocuklardan aranır (önce doğrudan çocuklar).
- `AnimationState`: başlangıç tiki; `getTimeInMillis(yaş) = (long)((yaş - başlangıç) * 50)`;
  `animateWhen`, `startIfStopped`, `fastForward`.
- `RabbitModel.setupAnim`: boşta baş eğme oynamıyorsa baş bakışla döner; sonra zıplama ve eğme
  animasyonları (başlamışsa). Bebek tavşanın kendi tanımları var.
- `Rabbit`: `baseTick` (istemci) animasyon durumlarını kurar: bekleme süresi dolduysa (en az
  180 + `nextInt(40)` tik) ve ağsız değilse eğme başlar, süre yeniden çekilir; zıplıyorsa
  zıplama başlar ve eğme durur; yoksa süre azalır ve zıplama durur. `aiStep` zıplama
  sayaçlarını ilerletir; olay 1 on beş tiklik zıplamayı başlatır; `getJumpCompletion`.

## Veri

`tools/generate/animations.py` (yenilemek için `all.py`) `ANIMATIONS`'ı yazar:
`entity/animation-definitions` satırlarıyla her koşuda baştan karşılaştırılır. `Rabbit.DATA_TYPE_ID`
(18, int; kahverengi 0, öldürücü 99) `entity/variant-fields`'ta karşılaştırılır.

## Kapsam

`entity/animation-definitions` 74 satır; `keyframe-animation` 52 satır (iki model, zıplama on iki
zamanda ve iki ölçekte, baş eğme sekiz zamanda; iki geçiş, üç hedef, döngülü ve sıkışan);
`animation-states` 4 satır; `rabbit-model` 12 satır; `living-entity-animations`'a beş tavşan satırı.

## Sapmalar (açık)

- Boşta eğmenin beklemesi ve sonraki beklemeler tohumsuz rastgele kaynaktan çıkar (referans
  da tohumlamaz); karşılaştırma için bekleme ve tohum sabitlenir (`fix_idle`), üretimde saat.
- Tavşanın tasması izlenmiyor (`SetEntityLink` yok): bağlıyken eğme beklemesi kesilmez.
- "Toast" adlı tavşanın dokusu (özel ad metni okunmuyor) çizilmez.
- Killer Bunny davranışı, sesler, binme yok.
