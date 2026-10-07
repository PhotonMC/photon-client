# T37d — Warden — [client-parity]

Bu kayıt, warden'ın beş modelinin (gövde ve dört parlayan katman) ve istemci durumunun (poz ve olaylarla
başlayan animasyon durumları, dokunaç ve kalp atışı) Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt sınıfı
`client-parity`'dir: modeller gerçek `WardenModel`'e, durum gerçek `Warden` varlığının tikine karşı
karşılaştırılır. Katman alfasının karışımı ve doku yolları oracle ile karşılaştırılmaz. Canlı gözlem
`server-conformance`.

## Java kaynak incelemesi

- `WardenModel.setupAnim`: bütün parçalar sıfırlanır; baş bakışı alır; yürüyüş (hız `min(0.5, 3·hız)`, konum
  `0.8662` ile ölçeklenir) baş, gövde, bacaklar ve kolları döndürür, kolların yeri her seferinde ilk konuma
  alınır; boşta sallanması baş ve gövdeye eklenir; dokunaçlar `tendrilAnimation · cos(yaş·2.25)·π·0.1`
  (çift duyarlıklı) kadar karşıt döner; sonra saldırı, ses dalgası, kazma, çıkış, kükreme ve koklama
  animasyonları bu sırayla (başlamışlarsa) üstüne uygulanır.
- Beş katman (`main`, `bioluminescent`, `pulsating_spots`, `tendrils`, `heart`) aynı parça hiyerarşisini taşır
  (ayıklanan parçalar boş kalır); aynı `setupAnim` hepsine uygulanır.
- `Warden.onSyncedDataUpdated(DATA_POSE)` (istemci): çıkış, kazma, kükreme ve koklama pozları kendi durumlarını
  (çalışıyorsa yeniden) başlatır; başka poz hiçbir durumu durdurmaz. `recreateFromPacket`: eklenme paketinin
  verisi 1 ise poz çıkıştır.
- `Warden.handleEntityEvent`: 4 kükremeyi durdurur ve saldırıyı başlatır; 61 dokunaç nabzını 10 yapar; 62 ses
  dalgasını başlatır; başkası `LivingEntity`'ye gider.
- `Warden.tick` (istemci, tikin sonunda): kalp nabzı, `tickCount` kalp atış aralığına bölününce 10 olur
  (aralık `40 − floor(clamp(öfke/80, 0, 1)·30)`; öfke, `CLIENT_ANGER_LEVEL` verisidir, 16 numaralı alan);
  dokunaç ve kalp nabzı her tik birer azalır, bir önceki değer saklanır; `getTendrilAnimation` ve
  `getHeartAnimation` bu ikisini kısmi tikle aradeğerler ve 10'a böler.
- `WardenRenderer`: beş katmanın dördü `LivingEntityEmissiveLayer`'dır (`entityTranslucentEmissive`, yüz
  aydınlatması var, dünya ışığı yok, hasar kırmızısı var), rengi `ARGB.white(alfa)`: biyolüminesan 1, iki
  nabızlı leke `max(0, cos(yaş·0.045 [+π])·0.25)`, dokunaçlar ve kalp kendi nabzı; alfa `≤ 1e-5` ise çizilmez.
  Sıra: biyolüminesan, leke 1, leke 2, dokunaçlar, kalp.
- Entity gölgelendiricisi bir bant içi işaret daha kazandı: negatif ışığın ikinci değeri negatif değilse
  köşenin alfasıdır (`texel.a · alfa` karışır).

## Kapsam

`entity/warden-model` 33 satır (gövde: yalnız bakış, yürüyüş üç hızda, dokunaç, altı animasyon erken ve geç,
kükreme ile saldırı, yürürken kükreme; dört katmanda: durumsuz, bakış-yürüyüş-dokunaç, çıkış);
`living-entity-animations`'a 8 satır (sakin, dört poz, kükremeden saldırıya, ses dalgası, dokunaç, üç öfke
düzeyinde kalp atışı). Gerçek `Warden` tiki Java'da gerçek istemci seviyesinde çalıştırılır.

## Sapmalar (açık)

- Parlayan katmanlar derinliğe yazar (Java'da yazmaz); aynı yüzeye çizildikleri için görünür fark beklenmez,
  ama yarı saydam katman sonradan çizilen bir varlığı gizleyebilir. Karışım sırası, gövdeden sonra katmanların
  eklenme sırasıdır; `order(1)` ayrı bir sıralama yapılmaz.
- Kalp atışı sesi, kazma/çıkış blok parçacıkları, dokunaç tıkırtısı yok. Doku yolları renderer sabitinden elle
  yazıldı. Canlı gözlem yapılmadı; gölgelendirici yalnız derlenme düzeyinde denetlendi.
