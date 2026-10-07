# T30 — Koyun ve tavuk — [client-parity]

Bu kayıt, koyunun (gövde, alt yün, yün; yetişkin ve bebek; boyalı, kırpılmış,
yerken) ve tavuğun (yetişkin, soğuk, bebek; kanat çırpması) Java 26.2 ile
Rust'ta aynı köşe konum, doku koordinatı ve normali üretmesini, yünün boya
renginin ve yeme ölçeklerinin aynı değerleri vermesini kapsar. Kanıt sınıfı
`client-parity`'dir: gerçek model sınıfları (`SheepModel`, `BabySheepModel`,
`SheepFurModel`, `AdultChickenModel`, `ColdChickenModel`, `BabyChickenModel`)
katman tablosundan pişirilip gerçek `setupAnim` ile pozlanır, parça çizicisiyle
çizilir ve her dörtgen bit bit karşılaştırılır; yeme ölçekleri gerçek `Sheep`
sınıfından, renkler gerçek `ColorLerper`'dan okunur. Canlı gözlem
(boyalı/kırpılmış/bebek koyun, tavuk) `server-conformance`.

## Java kaynak incelemesi

- `SheepModel.setupAnim`: dört ayaklı poz, sonra baş `y += yemePozisyonu * 9 * yaşÖlçeği`
  ve `xRot = yemeAçısı` (yemiyorken açı eğimin kendisidir). `SheepFurModel` aynıdır.
- `SheepRenderer`: gövde dokusu bebekte ayrı; katmanlar sırasıyla alt yün
  (`SheepWoolUndercoatLayer`: yalnız yetişkin ve boyası beyaz değilken, kırpılmışta da)
  ve yün (`SheepWoolLayer`: kırpılmamışsa, bebek modeli/dokusu ayrı). İkisi de
  `entityCutout` ile dokuyu yün rengiyle çarpar; görünmez koyunda yün çizilmez.
- `ColorLerper.Type.SHEEP`: boya renginin 0,75 parlaklığı (beyaz sabit);
  `jeb_` koyun 25 tikte bir boyadan sonrakine tamsayı lerp ile geçer.
- `Sheep`: yeme zamanlayıcısı `EntityEvent` 10 ile 40, istemci `aiStep`'te her tik azalır;
  `getHeadEatPositionScale/AngleScale` zamana göre.
- `ChickenModel.setupAnim`: bacaklar yürüme ile, kanatlar `(sin(flap)+1)*flapSpeed` ile
  (`zRot` karşıt işaretli), yetişkin ve soğuk modelde baş bakışla. `Chicken.aiStep`
  (istemci): `flapSpeed` yerdeyken −0,3, havadayken +1,2 tik (0..1 arası), havadayken
  `flapping` en az 1, her tik ×0,9, `flap += flapping * 2`.
- Soğuk tavuk gövde/baş modeli ve bebek ayrı katmanlar; çeşit (`ChickenVariant`)
  `ModelType` ile model seçer (NORMAL, COLD), bebek her zaman bebek modeli.

## Veri

- `Sheep.DATA_WOOL_ID` (18, bayt: düşük dört bit boya, bit dört kırpılmış) ve
  `Chicken.DATA_VARIANT_ID` (18, serileştirici 30) veri indeksleri Java erişimcilerinden
  `entity/variant-fields` satırlarıyla her koşuda karşılaştırılır.
- Boya renk tablosu (`SHEEP_COLORS`, oracle'dan üretilir: `tools/generate/sheep_colors.py`)
  ve tavuk çeşitleri (`entity_variants.py`, artık üç tür) her koşuda baştan karşılaştırılır;
  `jeb_` rengi Rust'ta tablodan hesaplanır, döngü boyunca on bir noktada karşılaştırılır.
- Varlık köşesi: entity köşe rengi (`Vertex.tint.w`, `0xRRGGBB`, sıfır = yok;
  `scene::entity_colour`) shader'da dokuyla çarpılır; boyalı yün ve (ileride) boyalı
  zırh bunu kullanır.

## Kapsam

`entity/sheep-model` 25 satır (beş katman × beş poz), `entity/chicken-model` 15 satır
(üç katman × beş poz), `entity/sheep-colors` 27 satır (16 boya + 11 `jeb_` noktası),
`entity/sheep-eat` 31 satır (zamanlayıcı × kısmi tik). Aile içi renk/çeşit tablo satırıdır.

## Sapmalar (açık)

- `jeb_` adlı koyunun gökkuşağı renk döngüsü çizimde yok: özel ad bileşeninin metni
  okunmuyor (renk işlevi hazır ve karşılaştırılıyor).
- `Chicken.aiStep`'in flap kolu oracle'a alınamadı: `super.aiStep()` bütün `LivingEntity`
  tikini çalıştırır, oracle yalnız kuyruğu çağıramaz; kaynak okuması + birim test. `onGround`
  sunucunun konum paketlerinden gelir (istemci fiziği tavuğu düşürmüyor).
- Tavuğun `flapping`'e bağlı düşüş yavaşlaması (`deltaMovement.y *= 0.6`) istemci
  fiziğinde yok; yalnız çizime giren `flap`/`flapSpeed` modellenir. Yumurta/binici tavuk yok.
- Işıltılı görünmez koyunun yün çıkışı (`appearsGlowing`) yok.
- Keçi, tavşan, mantar ineği T30b'de.
