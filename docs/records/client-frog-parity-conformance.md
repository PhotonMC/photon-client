# T35g — Kurbağa ve iribaş — [client-parity]

Bu kayıt, kurbağanın ve iribaşın modelinin ve kurbağanın istemci tikinin (poz değişince başlayan
animasyon durumları, suda boşta durma, adım hızı) ile çeşit tablosunun Java 26.2 ile aynı sonucu
vermesini kapsar. Kanıt sınıfı `client-parity`'dir: modeller gerçek `FrogModel` ve `TadpoleModel`'e, tik
gerçek `Frog` varlığına karşı karşılaştırılır. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `FrogModel.setupAnim`: zıplama, vıraklama ve dil animasyonları durumları başlamışsa uygulanır; sonra
  yürüme (`applyWalk(konum, hız, 1.5, 2.5)`), suda ise yüzme (`applyWalk(konum, hız, 1.0, 2.5)`); sonra suda
  boşta durma animasyonu. Boğaz (`croaking_body`) yalnız vıraklama durumu başlamışken görünür. Zıplama, vıraklama
  ve dil süresi dolunca son karede kalır; yürüme, yüzme ve boşta durma döner.
- `Frog.onSyncedDataUpdated`: veride **poz** geldikçe (değişip değişmediğine bakılmaz) uzun zıplama, vıraklama
  ve dil pozlarının durumu o tikte başlar (zaten çalışıyorsa yeniden), pozu başka olanınki durur.
- `Frog.tick` (istemci): suda olan ve yürümeyen kurbağanın boşta durma durumu çalışır; bu, tikin geri
  kalanından **önce** denetlenir, yani son tikin suda olma ve yürüme değerlerini görür.
- `Frog.updateWalkAnimation`: hedef hız mesafe × 25 (üst sınır bir), zıplama çalışırken sıfır; yumuşatma 0,4.
- `TadpoleModel.setupAnim`: kuyruk `-çarpan × 0,25 × sin(0,3 × yaş)` döner; suda çarpan bir, karada bir buçuk.
- Veriler: çeşit 18 (kayıt defteri numarası; gönderilmezse `temperate`), çeşit adları üç dokudan biri
  (`cold`, `temperate`, `warm`). Çeşit tablosu oracle'dan yakalanır ve her koşuda baştan sona karşılaştırılır.

## Kapsam

`entity/frog-model` 18 satır (kurbağada yürüme, yüzme, üst sınır, her animasyon başlamışken ve süresi
dolmuşken, döngüye giren boşta durma, birleşimler; iribaşta suda ve karada); `entity/variants`'e üç
satır; `entity/variant-fields`'e bir satır; `living-entity-animations`'a 13 satır (suda boşta durma,
yüzüp durma, karada, yürüme, zıplama ve yeniden başlaması, hareket ederken zıplama, vıraklama, dil, poz
değiştirme, suda zıplama; iribaş suda ve karada).

## Sapmalar (açık)

- Dil hedefi (`DATA_TONGUE_TARGET_ID`) ve yutma, sesler, parçacıklar, davranışlar yok; yumurta (`Frogspawn`) ve
  iribaşın kurbağaya dönüşmesi yok. Gölge yok. Canlı gözlem yapılmadı.
- İribaşın doku yolu renderer'ın sabitinden elle yazıldı, oracle ile karşılaştırılmaz.
