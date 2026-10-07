# T36c — Papağan — [client-parity]

Bu kayıt, papağanın modelinin (beş poz), kanat çırpışının ve çeşit dokularının Java 26.2 ile aynı sonucu
vermesini kapsar. Kanıt sınıfı `client-parity`'dir: model gerçek `ParrotModel`'e, tik gerçek `Parrot`
varlığına, doku tablosu gerçek `ParrotRenderer.getVariantTexture`'a karşı karşılaştırılır. Canlı gözlem
`server-conformance`.

## Java kaynak incelemesi

- `ParrotModel.setupAnim`: önce pozun kendi duruşu (`prepare`): uçarken bacaklar `2π/9` kıvrılır; otururken
  başın, kuyruğun, gövdenin, kanatların ve bacakların hepsi 1,9 iner, kanatlar sabit, bacaklar `π/2` bükülür;
  partide bacaklar açılır. Sonra baş bakışı alır. Sonra pozun kolu: ayaktaki papağanın bacakları yürür **ve**
  uçanınki gibi kanat çırpar (`switch`'in kolları birbirine düşer); oturan hiçbirini yapmaz; partideki papağan
  `cos`/`sin(yaş)` ile sallanır (baş bakışı sıfırlanır, bacaklar yürümez).
- `Parrot.aiStep` (istemci, `super.aiStep`'ten sonra): kanat hızı havada ve biniciyken olmayan papağanda her tik
  `+1,2`, yerde ya da binerken `−0,3` (0 ile 1 arası), çırpma kuvveti havada 1'e çıkar ve her tik 0,9 ile çarpılır;
  havada ve düşerken dikey hareket 0,6 ile çarpılır. Çizim açısı `(sin(çırpma) + 1) × hız`, tikler arasında
  enterpolasyonla.
- Poz: parti (jukebox), oturan, havada uçan, yoksa ayakta. Omuz pozu yalnız omuz katmanından gelir.
- Veriler: çeşit 20 (int, sınır dışı kimlik en yakın çeşide kenetlenir); dokular `red_blue`, `blue`, `green`,
  `yellow_blue` ve `grey` dosyası (çeşit adı `gray` ama dosya `grey`).

## Kapsam

`entity/parrot-model` 20 satır (beş poz, dört durumda: durgun, kanat çırparken, bakarken, yürürken);
`entity/parrot-textures` 8 satır (sınır dışı kimlikler dahil); `living-entity-animations`'a 5 satır (yerde, havada,
konma, düşme, çeşit); `entity/variant-fields`'e bir satır.

## Sapmalar (açık)

- Parti pozu (jukebox çalarken) ulaşılamaz: jukebox seviye olayı ve `setRecordPlayingNearby` bağlı değil; model
  pozu oracle'la karşılaştırılır ama çizim onu seçmez. Omuz papağanı (oyuncu katmanı) yok. Başka mobları
  taklit etme sesi (`imitateNearbyMobs`), sesler, parçacıklar, gölge yok. Canlı gözlem yapılmadı.
