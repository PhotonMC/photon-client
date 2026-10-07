# P16.m — Mip seviyesi üretimi, cutout stratejileri ve sRGB tabloları — [client-parity] birleşik kayıt

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik görüntü pinned JAR'daki
`MipmapGenerator.generateMipLevels` yoluna ve üretim kütüphanesindeki
`voxel_client::render::mipmap` modülüne verilir; her seviyenin boyutu ve bütün
pikselleri karşılaştırılır. İki renk tablosu da ayrıca satır satır sınanır.

## Java kaynak incelemesi

Bir dokuyu yarıya indirmek baytların ortalaması **değildir**. Referans her renk
kanalını sabit bir tabloyla doğrusal ışığa çevirir, orada ortalar ve geri
çevirir. Bu yüzden koyu ve parlak bir texel'in mip'i, bayt ortalamasının
vereceğinden parlak çıkar.

Alfa bu yoldan geçmez; dört değerin düz bayt ortalaması alınır.

Saydam dokular bu yolu olduğu gibi kullanamaz. İki ayrı sorun var. Birincisi
renk: tamamen saydam bir texel'in rengi çoğu dokuda siyahtır ve yarıya inerken
görünen renge karışır, yani yaprak kamera geri çekildikçe koyu bir kenar
kazanır. Referans bunu halvelemeden **önce** seviye sıfırı yerinde doldurarak
çözer — `TextureUtil.solidify` her saydam texel'e en yakın opak rengi yayar
(alfa sıfır kalır), `fillEmptyAreasWithDarkColor` ise görüntünün en koyu
renginin dörtte üçünü yazar. İkincisi boyut: halveleme sonucu alfa harmanı
sıklıkla kesme eşiğinin altına düşer, yani şekil her seviyede küçülür.
`scaleAlphaToCoverage` kaynağın kapsamını hedef alıp `0.0`–`4.0` arasında
**beş adım** ikili arama yapar ve en küçük hatayı veren ölçeği uygular.

Doldurmanın üç ayrıntısı sonucu belirler. `solidify`'ın tohum döngüsü `x`
dışta `y` içte ilerler ve komşular `{{1,0},{-1,0},{0,1},{0,-1}}` sırasıyla
gezilir; iki opak texel bir saydam texel'den aynı uzaklıktaysa hangisinin
kazandığına bu sıra karar verir. `item/` ile başlayan dokularda doldurma hiç
çalışmaz. Doldurma erken çıkıştan önce geldiği için sıfır mip istendiğinde de
uygulanır, yani seviye sıfırı değişmiş olarak döner.

Ölçekleme adımında son yazma `alpha * bestAlphaScale + bias + 0.025F` olup
`as8BitChannel` ile bayta iner; bu **taban alma**, yuvarlama değil. Sabit
`0.025` toplandığı için ölçek bir olsa bile alfa değişir: tamamen saydam bir
texel `6` alfa ile döner. Bir texel genişliğinde bir görüntüde örneklenecek
köşe kalmadığından bölen sıfırdır ve kapsam `NaN` çıkar; aramanın bütün
karşılaştırmaları başarısız olur, `break` çalışır ve ölçek bir kalır — bu bir
kaza değil, referansın o durumdaki davranışıdır ve pinlenmiştir.

`DARK_CUTOUT` harmanı `darkenedAlphaBlend`'dir: alfası sıfır olan texel'ler
toplama katılmaz ama bölen yine dörttür, yani deliğin yanındaki texel kendi
rengi değil gölgesi olarak okunur.

`AUTO` kendi başına bir strateji değildir: sprite'ın tamamen saydam texel'i
varsa `CUTOUT`'a, yoksa `MEAN`'e çözülür. Yarı saydamlık (`hasTranslucent`)
bu seçimi etkilemez.

Tablolar açılışta bir kez kurulur: `srgbToLinear` 256 girdiyi `1023`'e kadar
ölçekler, `linearToSrgb` 1024 girdiyi bayta indirir. İkisi de standart sRGB
transfer fonksiyonunu kullanır ve siyaha yakın bölgede doğrusal parçaya geçer.
Yuvarlama `Math.round`'dur, yani yarım ekleyip aşağı yuvarlama.

Doğrusal ortalamadaki bölme **tamsayı bölmesidir** ve tablo dönüşünden önce
yapılır; kayan noktada ortalayıp sonra çevirmek farklı sonuç verir.

## Girdiler

`image/srgb-tables` her iki tablonun bütün girdilerini karşılaştırır: `256` ve
`1024` değer.

`image/mip-levels` yirmi satır taşır. Altısı `MEAN` yolunu sürer: iki gradyan,
iki gürültülü görüntü, düz renkli küçük bir görüntü ve hiç mip istenmeyen bir
durum. Boyutlar `4`'ten `32`'ye kadar değişir, böylece halvelemenin tek piksele
inişi de kapsanır.

Kalan on dördü cutout yollarını sürer: sert saydam kenarlı bir elmas şekli
(`CUTOUT`, `STRICT_CUTOUT`, sıfırdan farklı bias, `DARK_CUTOUT`), ölçekleme
aramasının yakınsamak zorunda kaldığı rampalı bir alfa, `item/` yolundaki bir
kopya, sıfır mip istenen iki durum, hiç opak texel'i olmayan iki görüntü ve
`AUTO`'nun üç saydamlık kombinasyonu.

## Sonuçlar

Java ve Rust her tablo girdisini ve her mip seviyesinin her pikselini aynı
verdi. `32x32` girdi `32 → 16 → 8 → 4 → 2` seviyelerini, `16x16` girdi
`16 → 8 → 4 → 2 → 1` seviyelerini üretti; sıfır seviye `MEAN` ile istendiğinde
kaynak dokunulmadan döndü.

Cutout satırları yukarıdaki üç ayrıntıyı doğrudan pinledi. Elmas şeklinin
`(0,0)` texel'i `0000b400`'den `34e8b400`'e döndü, yani doldurma `(4,8)`
texel'inin rengini seçti — tohum sırası ve komşu sırası birlikte bu sonucu
veriyor. `item/parity` kopyası `0000b400` kaldı. Hiç opak texel'i olmayan
görüntü `CUTOUT` ile siyah, `DARK_CUTOUT` ile `bfbfbf` (yani `3 × 255 / 4`)
doldu. Sıfır mip istenen cutout satırları tek seviye döndürdü ama o seviye
dolduruldu. `NaN` kapsamına düşen `1x1` seviyesi `06` alfa verdi.

`AUTO` üç satırda da beklendiği gibi çözüldü: saydamlık yoksa ve yalnızca yarı
saydamlık varsa `MEAN` çıktısı, tamamen saydamlık varsa `CUTOUT` çıktısı.

Koşu: `python3 tools/behavior/run.py capture --family image` ardından
`compare --family image --adapter target/release/examples/behavior_adapter`
→ `parity_passed`.

## Açık kapsam

Beş stratejinin hepsi kapsanmıştır ve stratejinin geldiği iki yol da artık
bağlı: sprite meta verisi ve piksellerden ölçülen saydamlık
[atlas kaydında](../atlas-layout-and-mip-upload.md) pinlendi, `build_atlas` ikisini
de okuyor. Üretilen seviyeler GPU'ya da yükleniyor.

Açık kalan, GPU'nun hangi seviyeyi seçtiği: sondanın kamerası küpü büyüttüğü
için her karede seviye sıfır baskın.
