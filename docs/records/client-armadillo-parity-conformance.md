# T37f — Armadillo — [client-parity]

Bu kayıt, armadillo'nun iki modelinin (yetişkin, yavru) ve istemci durumunun (eşzamanlı durumla başlayan
animasyon durumları, kabukta saklanma, bakışın gövdeye tutulması) Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt
sınıfı `client-parity`'dir: modeller gerçek `AdultArmadilloModel`/`BabyArmadilloModel`'e, durum gerçek `Armadillo`
varlığının tikine karşı karşılaştırılır. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `ArmadilloModel.setupAnim`: kabukta saklanırken gövdenin kendi küpleri çizilmez (`skipDraw`; çocukları çizilir),
  bacaklar ve kuyruk gizlenir, top (`cube`) görünür, yürüyüş uygulanmaz; değilse top gizlenir, baş bakışı alır
  (yukarı en çok 22.5°, aşağı 25°; yan 32.5°) ve yürüyüş (`applyWalk(konum, hız, 16.5, 2.5)`) çalışır; sonra
  yuvarlanma-açılma, yuvarlanma ve göz atma animasyonları (başlamışlarsa). Yavrunun animasyonları ayrı tanımlardır.
- `Armadillo.ARMADILLO_STATE` (veri 17, serileştirici 36; boşta, yuvarlanıyor, korkmuş, açılıyor): her gelişinde
  (aynı değer olsa da) durumda geçen süre sıfırlanır. `shouldHideInShell`: yuvarlanırken 5 tikten sonra, korkmuşken
  hep, açılırken 26 tikten önce.
- `Armadillo.tick` (istemci, tikin sonunda): `setupAnimationStates` (boşta üçü de durur; yuvarlanırken yuvarlanma
  başlar; korkmuşken göz atma, durumun ilk tikinde başlayıp 50 tik ileri sarılır, sonradan yalnız durmuşsa başlar,
  olay 64 geldiyse önce durdurulur; açılırken açılma başlar); korkmuşsa baş gövdeye tutulur
  (`Mob.clampHeadRotationToBody`, sınır 75°); durumda geçen süre bir artar.
- Yavru ayrı model katmanı ve doku kullanır (`AgeableMobRenderer`).

## Kapsam

`entity/armadillo-model` 28 satır (iki modelde: durumsuz, bakış ve sınırların dışı, yürüyüş, saklanma, üç animasyon
erken ve geç, açılırken yürüyüş); `living-entity-animations`'a 6 satır (boşta, yuvarlanma, yuvarlanıp korkma, olayla
yeniden göz atma, açılma, durumun yeniden gönderilmesi); oracle baş yönünü de karşılaştırır, ama satırlarda baş ve
gövde hep aynı yöne baktığı için sınırlama tam denenmez.

## Sapmalar (açık)

- Sesler, parçacıklar, davranışlar (yuvarlanma kararı, kabuk düşürme) ve gölge yarıçapı ayrıca denetlenmedi.
  Doku yolları renderer sabitinden elle yazıldı. Canlı gözlem yapılmadı.
- Parçalara `skip_draw` eklendi (`ModelPart.skipDraw`); başka model kullanmıyor.
