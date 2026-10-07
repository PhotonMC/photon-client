# T35e — Nautilus ve zombi nautilus — [client-parity]

Bu kayıt, nautilusun (yetişkin, bebek), zombi nautilusun ve mercanlı çeşidinin modelinin ve çeşit
tablosunun Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt sınıfı `client-parity`'dir: modeller
gerçek `NautilusModel` ve `ZombieNautilusCoralModel`'e karşı karşılaştırılır. Canlı gözlem
`server-conformance`.

## Java kaynak incelemesi

- `NautilusModel.setupAnim`: gövde (`body`) başın dönüşünü ve eğimini ±10 derece içinde izler; yüzme
  animasyonu her zaman çalışır: `applyWalk(konum + yaş / 5, hız + 0,2, 2, 3)`, yani durağan nautilus da
  gövdesini uzatıp kısaltır. Bebeğin kendi katmanı (`nautilus_baby`) vardır, aynı kod çalışır.
- `ZombieNautilusCoralModel.setupAnim`: mercanlar yalnız gövde zırhı boşken görünür.
- `ZombieNautilusRenderer`: model ve doku, sunucunun adını verdiği çeşitten gelir (`temperate`: normal,
  `warm`: mercanlı); çeşit yoksa çizilmez. Zombi nautilus bebek olmaz.
- Veriler: zombi nautilus çeşidi 21 (kayıt defteri numarası, gönderilmezse `temperate`). Yürüme için
  özel bir kural yoktur (`updateWalkAnimation` ezilmez).
- Çeşit tablosu oracle'dan yakalanır ve her koşuda baştan sona karşılaştırılır.

## Kapsam

`entity/nautilus-model` 18 satır (nautilus ve bebekte durgun, ileri yaş, hafif ve sınır aşan bakış,
yüzme ve hız üst sınırı; zombi ve mercanlı zombide durgun, bakıp yüzme, zırhlı); `entity/variants`'e iki
satır; `entity/variant-fields`'e bir satır.

## Sapmalar (açık)

- Eyer ve gövde zırhı katmanları (`SimpleEquipmentLayer`: `nautilus_saddle`, `nautilus_armor`) çizilmez;
  zırhın varlığı yalnız mercanları gizler. Sesler, kabarcıklar, atılış, binme yok. Gölge yok. Canlı
  gözlem yapılmadı.
- Nautilusun dokuları (`nautilus`, `nautilus_baby`) renderer sabitinden elle yazıldı, oracle ile
  karşılaştırılmaz.
