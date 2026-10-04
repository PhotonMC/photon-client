# T35f — Aksolotl — [client-parity]

Bu kayıt, aksolotlun (yetişkin ve bebek) modelinin ve istemci tikinin (eased karışım çarpanları, bebeğin
animasyon durumları, suda ve yerde olma, yürüme) Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt
sınıfı `client-parity`'dir: modeller gerçek `AdultAxolotlModel` ve `BabyAxolotlModel`'e, tik gerçek
`Axolotl` varlığına karşı karşılaştırılır. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `AdultAxolotlModel.setupAnim`: gövde başın dönüşünü alır; sonra beş poz kendi çarpanıyla karıştırılır
  (yüzme `min(hareket, suda)`, havada asılma `min(hareketsiz, suda)`, sürünme `min(hareket, yerde)`,
  yerde yatma `min(hareketsiz, yerde)`, ölü taklidi), çarpan `1e-5` veya altındaysa poz atlanır; en
  sonda sağ bacaklar sol bacakları aynalayarak ekler (`1 - min(yerde, hareket)`).
- `BabyAxolotlModel.setupAnim`: yürüme animasyonu başlamışsa yürüyüşe göre (`applyWalk(konum, hız, 15, 30)`),
  sonra yüzme, **yürüme durumuyla** sualtı yürüyüşü (referansın kendi tuhaflığı: sualtı yürüme durumu
  kullanılmaz), yerde boş, sualtı boş, sualtı-yerde boş ve ölü taklidi animasyonları, başlamış olanlar.
- `Axolotl.baseTick` (istemci): bebekte hareket çarpanı ve yedi durumdan biri tek başına çalışır (öncelik:
  ölü taklidi, hareketliyse suda-havada yüzme / karada yürüme / öteki sualtı yürüme, hareketsizse suda-havada
  / suda-yerde / karada boş); yetişkinde dört çarpan (ölü taklidi, suda, yerde, hareket) 10 tikte yükselir
  ve `in_out_sine` ile yumuşatılır. Hareket `walkAnimation.isMoving()` (hız `> 1e-5`); referansın dönüş
  karşılaştırması işe yaramaz, çünkü `LivingEntity.baseTick` eski dönüşü bundan önce yeni dönüşe eşitler.
- Veriler: tür 18 (int, tanınmayan kimlik ilk tür) ve ölü taklidi 19 (boolean). Suda olma durumu izlenir.
- Bulgu: bebeğin yürüyüş konumu üç kat ölçeklenir (`updateWalkAnimation`); Rust'ta yoktu, tüm bebek mob'lar
  için eklendi. Bebek yunus, `AgeableMob`'un bebek verisini okumuyordu (T35d); düzeltildi.

## Kapsam

`entity/axolotl-model` 22 satır (yetişkinde saf ve karışık çarpanlar, bebekte her animasyon);
`entity/variant-fields`'e iki satır; `living-entity-animations`'a 16 satır (suda, yüzerken, karada,
sürünürken, havada, ölü taklidi ve kalkış, türler; bebekte suda, yüzerken, karada, sürünürken, dipte,
ölü taklidi; bebek yunusun kutusu).

## Sapmalar (açık)

- Kova ile tutma, sesler, parçacıklar, davranışlar yok; gölge yok. Canlı gözlem yapılmadı.
- Doku yolu türün adından türetilir, gerçek renderer sabitiyle karşılaştırılmaz.
