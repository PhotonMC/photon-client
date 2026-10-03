# T34e — Bakır golem — [client-parity]

Bu kayıt, bakır golemin modelinin (`CopperGolemModel.setupAnim`: bakış, yürüyüş, eşya tutarken
kolların kalkması, boşta baş dönüşü ve dört sandık animasyonu), istemci durumunun (`CopperGolem`
animasyon durumları ve boşta bekleme) ve oksitlenme aşamasına göre dokunun Java 26.2 ile aynı
sonucu vermesini kapsar. Kanıt sınıfı `client-parity`'dir: model gerçek `CopperGolemModel`'e,
durum gerçek `CopperGolem` varlığının tikine karşı karşılaştırılır; çizim (doku seçimi, parlayan
gözler) birim koduyla bağlanır, canlı gözlemi `server-conformance`'tır.

## Java kaynak incelemesi

- `CopperGolemModel.setupAnim`: baş bakışla döner (`xRot`, `yRot`); iki eli de boş golem
  `COPPER_GOLEM_WALK`'u, bir eli dolu olan `COPPER_GOLEM_WALK_ITEM`'ı `applyWalk(konum, hız, 2, 2,5)`
  ile oynatır; eşya varken kollar yürümeyle bile `x ≤ -0,8727`, sağda `y ≤ -0,1134` ve `z ≤ -0,0646`,
  solda `y ≥ 0,1134` ve `z ≥ 0,0646`'dan aşağı inmez (`poseHeldItemArmsIfStill`); sonra boşta, ve dört
  sandık animasyonu başlamışlarsa oynar. Durum adları ile animasyonlar çaprazdır: `interactionGetItem`
  `..._NOITEM_GET`, `interactionDropNoItem` `..._ITEM_NODROP`'tur.
- `CopperGolem.setupAnimationStates` (istemci tiki, `NoAI` değilse): `IDLE`'da dört etkileşim durur;
  boşta başlangıcı bugünse boşta başlar, sıfırsa `tickCount + nextInt(200, 240)` çekilir, başlangıçtan on
  tik sonra (başlık dönüş sesi çalarken) sıfırlanır; öteki durumlarda boşta durur, başlangıç sıfırlanır
  ve o durumun animasyonu `startIfStopped` ile başlar, diğerleri durur.
- Veri: `DATA_WEATHER_STATE` (16) ve `COPPER_GOLEM_STATE` (17), ikisi varint kimlik; durum kimliği aralık
  dışındaysa ilki, hava durumu en yakın uca sıkışır. Ağırlıklı aşama dokuyu ve göz dokusunu seçer.
- `Mob.baseTick` her tik, varlığın kendi rastgele kaynağından `nextInt(1000)` çeker (ortam sesi sayacı,
  golemin aralığı 120, sesi yoktur): boşta beklemenin çekilişi bu yüzden tohumdan yalnız bu çekilişlerle
  birlikte tahmin edilir; Rust aynı sırayı izler.

## Kapsam

`entity/copper-golem-model` 45 satır (bakış, yürüyüş, eşyalı yürüyüş ve kol sınırı, boşta yedi yaş,
dört sandık animasyonu yedi yaşta, ikisi birleşik); `living-entity-animations`'a 19 gerçek `CopperGolem`
tiki satırı (boşta başlangıcı ve ikinci dönüş, dört sandık eylemi ve geçişleri, `NoAI`, iki hava
aşaması); `entity/data-ids`'a iki satır (hava durumu 16, durum 17, serileştirici kimlikleriyle).

## Sapmalar (açık)

- Eldeki eşya, antenin üstündeki blok ve kafa eşyası çizilmez (T34f); eşya tutmak yalnız kol duruşunu ve
  yürüyüş animasyonunu etkiler.
- Boşta bekleme tohumsuz kaynaktan çıkar (referans da tohumlamaz); karşılaştırmada tohum sabitlenir
  (`fix_copper_golem_seed`), üretimde saat.
- Baş dönüş sesi, adım ve hasar sesleri yok; heykele dönüşme ve düğme/mum etkileşimleri kapsam dışı.
- Gölge çizilmez (yalnız domuz, inek, eşya ve orb gölgesi var).
