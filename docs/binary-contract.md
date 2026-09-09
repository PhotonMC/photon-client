# P03.b — UUID, BlockPos, SectionPos, long array ve BitSet referans sözleşmesi

Bu dilimin **116 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](binary-parity.md)). Diğer P03 biçimleri bu dilime dahil değildir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `network/FriendlyByteBuf.java` içindeki `readUUID`/`writeUUID`, `readBlockPos`/`writeBlockPos`, `readLongArray`/`writeLongArray`, BitSet'in bu diziye indirgenmiş karşılığı, ayrıca `core/BlockPos.java` ve `core/SectionPos.java` içindeki `asLong`, `of` ve koordinat çıkarma incelendi. Adaptör doğrudan orijinal sınıfları çağırır; üst katman paket mantığı veya gerçek ağ bağlantısı bu testte yapılmaz. BitSet okuma/yazma, instance metoda girmeden `readLongArray`/`writeLongArray` ile `BitSet.valueOf`/`toLongArray` üzerinden kurulur; bu, referansın ilgili iki satırının birebir karşılığıdır.

Netty buffer/common yanında `core/BlockPos` sınıfı için math ailesinin bağımlılıkları ve `commons-lang3` gerekir. Yerel JAR'ın SHA-1 değeri `tests/behavior/reference.json` içine sabitlendi. Yeni oyun bağımlılığı veya Rust kütüphanesi eklenmedi.

## Davranış sözleşmesi

- UUID iki big-endian i64 olarak taşınır: önce most, sonra least. Herhangi bir 16 bayt geçerlidir; uzunluk öneki ve sürüm/varyant denetimi yoktur.
- BlockPos tek big-endian i64 olarak taşınır. X/Z 26 bit, Y 12 bit işaretli alanlara maskelenir; aralık dışı i32 girdiler sarar.
- SectionPos tek big-endian i64 olarak taşınır. X/Z 22 bit, Y 20 bit işaretli alanlara maskelenir; aralık dışı girdiler sarar. Okuma tek `readLong` olduğundan eksik veride tüketim olmaz. Açılım aritmetik kaydırma ile işaret genişletir.
- Okuma her `readLong` için atomiktir: yetersiz veri tüketimi ilerletmez. UUID iki ardışık okumadır; ikinci yarı eksikse ilk 8 bayt tüketilmiş olur. BlockPos tek okumadır; eksik veride tüketim olmaz.
- Long array önce VarInt uzunluğu okur. VarInt önek hataları wire tüketimini korur. Negatif uzunluk `NegativeArraySizeException`, kullanılabilir veriyi aşan uzunluk `DecoderException` üretir; ikisi de `decoder_error` kategorisidir ve gövde tüketilmez. Başarılı gövdede bit deseni aynen korunur.
- BitSet, long array gövdesinin `BitSet.valueOf` görünümüdür. Sondaki sıfır long'lar yazmada kısalır; okumada aynı bit kümesi aynı sıralı bit listesini verir.
- Yazma bu dilimde her zaman başarılıdır; uzunluk sınırı ve hata kategorisi yoktur. Mevcut yazma sınırı hatalarında hedef buffer'a yeni veri eklenmez kuralı burada tetiklenmez.
- Sabit BitSet bit boyutu `size` ile taşınır; `ceil(size/8)` bayt okunur/yazılır. Boyut 0 iken tüketim/yazım olmaz. Eksik veri `buffer_underflow`, bit kümesi boyutu aşarsa `encoder_error` üretir; hedefe ekleme yapılmaz.
- Okuma hataları `buffer_underflow`, `decoder_error`, VarInt önekinden gelen `too_big` ve sabit yazmada `encoder_error` kategorileridir. Java exception sınıfı yalnızca `diagnostics` içindedir; mesaj/stack eşitlik ölçütü değildir. NBT kapsam dışıdır.

İlk corpus `prefix_hex` olarak `a55a` kullanır; heap buffer ve sıfırdan farklı reader index ile çalışır. Direct/composite buffer varyantları ve çoklu değer akışları kapsam dışıdır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| UUID yazma | 8 | Sıfır/tüm-bir, örnek, alterne bitler, sürüm benzeri, tek bit ve `deadbeef` desenleri |
| UUID okuma | 9 | 0/1/7/8/9/15 baytlık kesikler, tam 16 bayt ve tüm-bir yük; kalan veri `55aa` ile sınanır |
| BlockPos yazma | 12 | Köken/birim/negatif, yatay/Y sınırları ve sarma komşuları, dünya sınırı, i32 uçları ve karışık koordinat |
| BlockPos okuma | 14 | Bağımsız 8 baytlık yükler ve 0–7 baytlık kesikler; kalan veri `55aa` ile sınanır |
| SectionPos yazma | 12 | Köken/birim/negatif, 22/20 bit sınırları ve sarma komşuları, i32 uçları ve karışık koordinat |
| SectionPos okuma | 12 | Bağımsız 8 baytlık yükler ve 0–7 baytlık kesikler; kalan veri `55aa` ile sınanır |
| Long array yazma | 6 | Boş, sıfır/örnek, i64 uçları, 7/35 bit sınırları ve `deadbeef` deseni |
| Long array okuma | 17 | Boş/tek/çift yük, kısa/eksik gövde, negatif ve kısa-olmayan sıfır uzunluklar, 127 uzunluk sınırı, 0–6 baytlık kesik önekler ve fazla uzun önek |
| BitSet yazma | 6 | Boş, bit 0, bayt/kelime sınırları, örnek ve sondaki sıfır kırpması |
| BitSet okuma | 6 | Boş/tek/çift kelime, kısa gövde ve negatif uzunluk |
| Sabit BitSet okuma | 8 | Boyut 0/1/8/9/16, yok sayılan yük, kısa/eksik gövde |
| Sabit BitSet yazma | 6 | Boyut 0/8/9/16, sınır içi ve taşan bit kümeleri |

`family: "binary"` altında `codec` `uuid` | `blockpos` | `sectionpos` | `longarray` | `bitset` | `fixedbitset` olur. `prefix_hex` küçük harfli bayt hex stringdir. UUID yazmada `most_hex`/`least_hex` 16 haneli küçük harfli hex; BlockPos ve SectionPos yazmada `x`/`y`/`z` işaretli ondalık string; long array yazmada `values` işaretli ondalık string dizisi; BitSet yazmada `bits` bit index dizisidir. Sabit BitSet okuma/yazmada ek olarak `size` bit boyutu bulunur. Okumada `hex` başlık/gövdeyi taşır.

Okuma adaptörü buffer başına `prefix_hex` koyup reader index'i bu alanın sonuna taşır. Başarılı UUID sonucu `most_hex`, `least_hex`; başarılı BlockPos ve SectionPos sonucu `x`, `y`, `z`; başarılı long array sonucu `values`; başarılı BitSet ve sabit BitSet sonucu `bits` içerir. Her durumda `outcome`, `consumed` ve `remaining_hex` bulunur. Hata sonucu `outcome` ve `error` içerir.

Yazma adaptörü mevcut `prefix_hex` üzerine ekler. Her sonuç `outcome`, eklenen bayt sayısı `written` ve prefix dahil bütün `hex` çıktısını içerir. Sabit BitSet yazmada bit kümesi boyutu aşarsa `encoder_error` döner ve hedefe ekleme yapılmaz; bağımsız okuma senaryoları ayrıdır.

## Çalıştırma ve kanıt

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family binary
python3 tools/behavior/run.py verify --family binary
```

116 senaryo iki ayrı JVM'de aynı sonucu verdi; kayıtlı referans iki yeni JVM ile yeniden doğrulandı. Binary referans kimliği: `8ae00c7621477db27bbb1744c78d0d55d67b5dd4854c14a578aba7d155be0119`. Önceki 78 girdisi ve sonucu değişmedi. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Yeni aileyle toplam 679 Java senaryosu yakalandı ve iki yeni JVM çalıştırmasıyla kayıtla eşitliği doğrulandı. Birleşik referans kimliği: `34ba8e91cef5a27e4d84dd229d2dd1468ba20c443bbe0d362fceb3110ccd294d`. Eski kayıtlar değiştirilmedi.

Beklenen çıktılar Rust'a göre değiştirilmedi.

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --family timer --family wire --family utf8 --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py compare --family binary --adapter target/debug/examples/behavior_adapter
```

Beklenen çıktılar Rust'a göre değiştirilmedi.
