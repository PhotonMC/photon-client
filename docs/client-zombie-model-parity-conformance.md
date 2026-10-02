# T28 — Zombi ailesi modeli — [client-parity]

Bu kayıt, zombi, husk, boğulmuş, dev ve bebek biçimlerinin model pozunun
(`HumanoidModel.setupAnim`, `AnimationUtils.animateZombieArms`, `DrownedModel`)
Java 26.2 ile Rust'ta aynı köşe konum, doku koordinatı ve normali üretmesini
kapsar. Kanıt sınıfı `client-parity`'dir: gerçek model sınıfları katman
tablosundan pişirilip gerçek `setupAnim` ile pozlanır ve parça çizicisiyle
çizilir; her dörtgen bit bit karşılaştırılır. Canlı kabul (zombi, husk,
boğulmuş + dış katman, bebek, dev) `server-conformance` gözlemidir.

## Java kaynak incelemesi

- `client/model/HumanoidModel.java:215-330`: baş bakışı, yürüme sallanması
  (`speedValue` 1), bacak titreşimi 0.005, kol nefesi (`bobModelPart`), saldırı
  sallaması, eğilme ve yüzme. Oyuncu modeli de aynı kodu kullanır; Rust'ta bu
  çekirdek `humanoid_anim` serbest işlevine çıkarıldı, oyuncu ve zombi aynı
  işlevi çağırır.
- `model/AnimationUtils.java:68-95`: zombi kolları `armDrop = -π / (saldırgan ?
  1.5 : 2.25)` ile öne uzanır, saldırı sallaması `animateAttackArms` ile
  eklenir, sonra kol nefesi yeniden uygulanır.
- `model/monster/zombie/DrownedModel.java:32-56`: `swimAmount > 0` iken kollar
  `-4π/5`'e, bacaklar `0.55 * sin(0.1 * yaş)` ile sallanır, baş düz.
- Bebek zombi (`BabyZombieModel`), husk, boğulmuş ve dev aynı parça adlarını
  paylaşır; dev `GiantMobRenderer`'ın altı ölçeğiyle çizilir. Boğulmuşun dış
  katmanı (`DrownedOuterLayer`) aynı pozla ikinci bir model ve dokudur.

## Kapsam ve dal kararı

`entity/zombie-model` 66 satır: altı katman (zombi, bebek, husk, dev, boğulmuş,
bebek boğulmuş) × dokuz poz (durma, saldırgan, bakış, yürüme, üç saldırı sallaması
aşaması, yürüme+saldırgan+sallama) ve üç katmanda iki yüzme durumu. Husk, dev ve
bebek aynı kod olduğundan tablo satırıdır. Bebek bayrağı artık zombinin tüm alt
türlerinde (`Zombie` sınıfı) okunur.

## Sapmalar (açık)

- Elde eşya yok: kol pozu hep boş el (`ArmPose.EMPTY`); trident fırlatma ve mızrak
  pozları ile bebeğin eşyalı kol bırakması T41'de. Saldırı sallaması
  (`attackTime`) `Animate` paketi gelene dek (T43) sıfır.
- Boğulmuş yüzüş (`swimAmount`) mob için tiklenmez (yalnız oyuncu); model dalı
  oracle'lanmış, canlı veri yok.
- Köylü zombisi (`ZombieVillagerModel`, meslek katmanı) T34'te.
- Yatış (ölüm) yönü bu ailede 90° (`getFlipDegrees` varsayılanı).
