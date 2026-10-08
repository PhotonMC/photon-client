# T32b — Kedi ve ocelot — [client-parity]

Bu kayıt, kedi ve ocelotun (yetişkin ve bebek) modelinin Java 26.2 ile Rust'ta aynı köşe konum, doku
koordinatı ve normali üretmesini; kedinin kendi istemci durumunun (yatma, kuyruk yatması, rahatlama
miktarları; oturma, evcil bayrağı, çömelme) aynı ilerlemesini; kedi çeşit dokularının Java ile aynı
tablodan gelmesini kapsar. Kanıt sınıfı `client-parity`'dir (model, tik ve tablo) ve canlı gözlemdir:
gerçek `AdultCatModel`, `BabyCatModel`, `AdultOcelotModel`, `BabyOcelotModel` katman tablosundan
pişirilip gerçek `setupAnim` ile pozlanır; kedinin tiki gerçek `Cat` varlığıyla ve gerçek veri paketleriyle
karşılaştırılır.

## Java kaynak incelemesi

- `AdultFelineModel.setupAnim` ve `BabyFelineModel.setupAnim`: çömelince gövde/baş/kuyruk `ageScale`
  kadar kayar ve kuyruk yatay döner; koşarken ikinci kuyruk parçası birincinin yüksekliğine iner. Baş
  bakışı sonra gelir. Oturmuyorsa bacaklar çaprazlama sallanır (koşarken arka/ön bacak farkı 0,3),
  kuyruk yürüyüşle sallanır; oturuyorsa yetişkinde gövde `π/4` yatar ve bütün parçalar `ageScale` ile
  ölçeklenen kaymalarla katlanır, bebekte küçük sabit kaymalar vardır. Yatma ve rahatlama miktarları
  `Mth.rotLerp` ile (radyan değerlere derece sarması uygulayan hâliyle) başa, bacaklara ve kuyruğa
  karışır; bebekte kuyruk açısı kendi üstüne eklenir (Java'daki gözlenen tuhaflık).
- `CatRenderer.extractRenderState`: doku çeşidin yetişkin/bebek varlığıdır; çömelme `Pose.CROUCHING`,
  koşma paylaşılan bayrak 3, oturma `TamableAnimal` bayrağı 1; evcilse tasma rengi. `OcelotRenderer`
  yalnız çömelme ve koşmayı okur: ocelot hiç oturmaz, yatmaz, rahatlamaz.
- `CatRenderer.setupRotations`: yatan kedi `(0,4, 0,15, 0,1)·miktar` kayar, Z ekseninde `90·miktar`
  derece yatar; uyuyan bir oyuncunun üstündeyse yatışın ardından ek `0,15·miktar` kayar.
- `CatCollarLayer`: evcil ve görünürken `cat_collar[_baby]` dokusu boya rengiyle çarpılarak, kendi
  katman modeliyle (`cat#collar`, `cat_baby#collar`: 0,01 şişirilmiş) aynı durumla pozlanıp çizilir.
- `Cat.tick` (istemci): yatma miktarı yatarken tikte `0,15`, kalkarken `0,22` değişir; kuyruk
  `0,08`/`0,13`; rahatlama `0,1`/`0,13`. Kedi yatıyorsa blok konumunun iki blok çevresindeki kutuda uyuyan
  bir oyuncu varsa "oyuncunun üstünde yatıyor" olur.
- Veri: `TamableAnimal.DATA_FLAGS_ID` 18, `Cat.DATA_VARIANT_ID` 20 (serileştirici 21), `IS_LYING` 21,
  `RELAX_STATE_ONE` 22, `DATA_COLLAR_COLOR` 23; verilmemişse çeşit `black`, tasma kırmızı.

## Tablolar

`entity/variants` tablosuna on bir `cat` çeşidi eklendi (`tools/generate/entity_variants.py` Rust
tablosunu üretir; her koşuda baştan sona karşılaştırılır; kedinin model ailesi yoktur, sütun
`normal`dır). Sürüm değişince `tools/generate/all.py` ile yenilenir. Veri kimlikleri
`entity/variant-fields` satırlarıyla Java'dan doğrulanır.

## Kapsam

`entity/feline-model` 48 satır (dört katman × yürüyüş, bakış, çömelme, koşma, oturma, yatma, rahatlama
ve karışımları); `living-entity-animations`'a altı kedi satırı (yabani, evcil oturan ve tasmalı,
çömelen, yatıp kalkan, rahatlayan, rahatlarken yatan); `entity/variant-fields`'a dört kedi satırı.

## Sapmalar (açık)

- Uyuyan oyuncu araması (`isLyingOnTopOfSleepingPlayer`) Java ile karşılaştırılmıyor: sahnede uyuyan
  bir oyuncu kurulamıyor (başsız yerel oyuncu yok, uzak oyuncu için profil gerekir). Dal
  `photon-world` birim testindedir (yakın uyuyan, uzak uyuyan, yatmayan kedi).
- Ocelotun güven (`DATA_TRUSTING`) verisi çizimi etkilemez; okunmaz.
- Kedi/ocelot sesleri (mırıltı, yalvarma) ve rastgele çekimleri yok.
- Çeşit seçimi, `CatSoundVariant` ve kedi hediyesi (sunucu) kapsam dışı.
