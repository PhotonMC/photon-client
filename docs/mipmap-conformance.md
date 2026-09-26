# P16.m — Mip seviyesi üretimi ve sRGB tabloları — [client-parity] birleşik kayıt

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

Tablolar açılışta bir kez kurulur: `srgbToLinear` 256 girdiyi `1023`'e kadar
ölçekler, `linearToSrgb` 1024 girdiyi bayta indirir. İkisi de standart sRGB
transfer fonksiyonunu kullanır ve siyaha yakın bölgede doğrusal parçaya geçer.
Yuvarlama `Math.round`'dur, yani yarım ekleyip aşağı yuvarlama.

Doğrusal ortalamadaki bölme **tamsayı bölmesidir** ve tablo dönüşünden önce
yapılır; kayan noktada ortalayıp sonra çevirmek farklı sonuç verir.

## Girdiler

`image/srgb-tables` her iki tablonun bütün girdilerini karşılaştırır: `256` ve
`1024` değer.

`image/mip-levels` altı satır taşır: iki gradyan, iki gürültülü görüntü, düz
renkli küçük bir görüntü ve hiç mip istenmeyen bir durum. Boyutlar `4`'ten
`32`'ye kadar değişir, böylece halvelemenin tek piksele inişi de kapsanır.

## Sonuçlar

Java ve Rust her tablo girdisini ve her mip seviyesinin her pikselini aynı
verdi. `32x32` girdi `32 → 16 → 8 → 4 → 2` seviyelerini, `16x16` girdi
`16 → 8 → 4 → 2 → 1` seviyelerini üretti; sıfır seviye istendiğinde kaynak
dokunulmadan döndü.

Koşu: `python3 tools/behavior/run.py capture --family image` ardından
`compare --family image --adapter target/release/examples/behavior_adapter`
→ `parity_passed`.

## Açık kapsam

Yalnızca `MEAN` stratejisi kapsanmıştır. `CUTOUT`, `STRICT_CUTOUT` ve
`DARK_CUTOUT` stratejileri alfa kapsamını ölçüp ikili aramayla ölçekler ve
`DARK_CUTOUT` ayrıca doğrusal ışıkta karartılmış bir harman kullanır; bunlar
saydam dokular için gereklidir ve açıktır. `AUTO` stratejisinin saydamlığa
göre seçim yapması da kapsanmadı.

Üretilen mip seviyeleri henüz GPU'ya yüklenmiyor: [çizim yolu](textured-cube-record.md)
atlası tek seviyeyle yüklüyor. Atlas paketleyicinin ayırdığı dolgu mip
seviyesine göre büyüdüğü için ikisi birlikte bağlanmalıdır.
