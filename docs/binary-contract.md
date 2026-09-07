# P03.b — UUID, BlockPos, long array ve BitSet referans sözleşmesi

Bu dilimin **78 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](binary-parity.md)). Diğer P03 biçimleri bu dilime dahil değildir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `network/FriendlyByteBuf.java` içindeki `readUUID`/`writeUUID`, `readBlockPos`/`writeBlockPos`, `readLongArray`/`writeLongArray` ve BitSet'in bu diziye indirgenmiş karşılığı, ayrıca `core/BlockPos.java` içindeki `asLong`, `of` ve koordinat çıkarma incelendi. Adaptör doğrudan orijinal sınıfları çağırır; üst katman paket mantığı veya gerçek ağ bağlantısı bu testte yapılmaz. BitSet okuma/yazma, instance metoda girmeden `readLongArray`/`writeLongArray` ile `BitSet.valueOf`/`toLongArray` üzerinden kurulur; bu, referansın ilgili iki satırının birebir karşılığıdır.

Netty buffer/common yanında `core/BlockPos` sınıfı için math ailesinin bağımlılıkları ve `commons-lang3` gerekir. Yerel JAR'ın SHA-1 değeri `tests/behavior/reference.json` içine sabitlendi. Yeni oyun bağımlılığı veya Rust kütüphanesi eklenmedi.

## Davranış sözleşmesi

- UUID iki big-endian i64 olarak taşınır: önce most, sonra least. Herhangi bir 16 bayt geçerlidir; uzunluk öneki ve sürüm/varyant denetimi yoktur.
- BlockPos tek big-endian i64 olarak taşınır. X/Z 26 bit, Y 12 bit işaretli alanlara maskelenir; aralık dışı i32 girdiler sarar. Açılım aritmetik kaydırma ile işaret genişletir.
- Okuma her `readLong` için atomiktir: yetersiz veri tüketimi ilerletmez. UUID iki ardışık okumadır; ikinci yarı eksikse ilk 8 bayt tüketilmiş olur. BlockPos tek okumadır; eksik veride tüketim olmaz.
- Long array önce VarInt uzunluğu okur. VarInt önek hataları wire tüketimini korur. Negatif uzunluk `NegativeArraySizeException`, kullanılabilir veriyi aşan uzunluk `DecoderException` üretir; ikisi de `decoder_error` kategorisidir ve gövde tüketilmez. Başarılı gövdede bit deseni aynen korunur.
- BitSet, long array gövdesinin `BitSet.valueOf` görünümüdür. Sondaki sıfır long'lar yazmada kısalır; okumada aynı bit kümesi aynı sıralı bit listesini verir.
- Yazma bu dilimde her zaman başarılıdır; uzunluk sınırı ve hata kategorisi yoktur. Mevcut yazma sınırı hatalarında hedef buffer'a yeni veri eklenmez kuralı burada tetiklenmez.
- Okuma hataları `buffer_underflow`, `decoder_error` ve VarInt önekinden gelen `too_big` kategorileridir. Java exception sınıfı yalnızca `diagnostics` içindedir; mesaj/stack eşitlik ölçütü değildir. Sabit BitSet varyantları ve NBT kapsam dışıdır.

İlk corpus `prefix_hex` olarak `a55a` kullanır; heap buffer ve sıfırdan farklı reader index ile çalışır. Direct/composite buffer varyantları ve çoklu değer akışları kapsam dışıdır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| UUID yazma | 8 | Sıfır/tüm-bir, örnek, alterne bitler, sürüm benzeri, tek bit ve `deadbeef` desenleri |
| UUID okuma | 9 | 0/1/7/8/9/15 baytlık kesikler, tam 16 bayt ve tüm-bir yük; kalan veri `55aa` ile sınanır |
| BlockPos yazma | 12 | Köken/birim/negatif, yatay/Y sınırları ve sarma komşuları, dünya sınırı, i32 uçları ve karışık koordinat |
| BlockPos okuma | 14 | Bağımsız 8 baytlık yükler ve 0–7 baytlık kesikler; kalan veri `55aa` ile sınanır |
| Long array yazma | 6 | Boş, sıfır/örnek, i64 uçları, 7/35 bit sınırları ve `deadbeef` deseni |
| Long array okuma | 17 | Boş/tek/çift yük, kısa/eksik gövde, negatif ve kısa-olmayan sıfır uzunluklar, 127 uzunluk sınırı, 0–6 baytlık kesik önekler ve fazla uzun önek |
| BitSet yazma | 6 | Boş, bit 0, bayt/kelime sınırları, örnek ve sondaki sıfır kırpması |
| BitSet okuma | 6 | Boş/tek/çift kelime, kısa gövde ve negatif uzunluk |

`family: "binary"`, `operation: "read" | "write"`, `codec: "uuid" | "blockpos" | "longarray" | "bitset"` ortak girdilerdir. `prefix_hex` küçük harfli bayt hex stringdir. UUID yazmada `most_hex`/`least_hex` 16 haneli küçük harfli hex; BlockPos yazmada `x`/`y`/`z` işaretli ondalık string; long array yazmada `values` işaretli ondalık string dizisi; BitSet yazmada `bits` artan bit index dizisidir. Okumada `hex` başlık/gövdeyi taşır.

Okuma adaptörü buffer başına `prefix_hex` koyup reader index'i bu alanın sonuna taşır. Başarılı UUID sonucu `most_hex`, `least_hex`; başarılı BlockPos sonucu `x`, `y`, `z`; başarılı long array sonucu `values`; başarılı BitSet sonucu `bits` içerir. Her durumda `outcome`, `consumed` ve `remaining_hex` bulunur. Hata sonucu `outcome` ve `error` içerir.

Yazma adaptörü mevcut `prefix_hex` üzerine ekler. Her sonuç `outcome`, eklenen bayt sayısı `written` ve prefix dahil bütün `hex` çıktısını içerir. Bu dilimde yazma hatası beklenmez; bağımsız okuma senaryoları ayrıdır.

## Çalıştırma ve kanıt

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family binary
python3 tools/behavior/run.py verify --family binary
```

78 senaryo iki ayrı JVM'de aynı sonucu verdi; kayıtlı referans iki yeni JVM ile yeniden doğrulandı. Binary referans kimliği: `e75ce61ebc174879fff859d20c605110eee12d0cf54b480e2f575e055017d2a0`. Önceki 43 UUID/BlockPos girdisi ve sonucu değişmedi. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Yeni aileyle toplam 565 Java senaryosu yakalandı ve iki yeni JVM çalıştırmasıyla kayıtla eşitliği doğrulandı. Birleşik referans kimliği: `47f992f4554d8aea1b33f0a1bbf9c793871c17af2fbbec51e8851e9f54de7a02`. Eski kayıtlar değiştirilmedi.

Mevcut Rust kodu UUID/BlockPos için geçerlidir; long array ve BitSet henüz uygulanmadığından desteklenen aileleri açıkça seç:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --family timer --family wire --family utf8 --adapter target/debug/examples/behavior_adapter
```

Varsayılan bütün-aile karşılaştırması bu kapsamı içerir; yeni testler sessizce atlanmaz.

Beklenen çıktılar Rust'a göre değiştirilmedi.
