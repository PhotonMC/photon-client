# P03.b — Vec3 stream codec, LP Vec3 stream codec, UUID, BlockPos, SectionPos, ChunkPos, long array ve BitSet referans sözleşmesi

Bu dilimin **164 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](binary-parity.md)). Java `Vec3.STREAM_CODEC` kapsamı [Vec3 stream codec sözleşmesinde](vec3-stream-codec-contract.md), `Vec3.LP_STREAM_CODEC` kapsamı [LP Vec3 stream codec sözleşmesinde](lp-vec3-contract.md) ve `ChunkPos.STREAM_CODEC` kapsamı [ChunkPos sözleşmesinde](chunkpos-contract.md) ayrıntılıdır. Bu dilimdeki diğer P03 biçimleri aşağıda listelenir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `world/phys/Vec3.java` içindeki iki stream codec, `network/LpVec3.java` packed codec'i ve continuation ölçeği için `network/VarInt.java`; ayrıca `network/FriendlyByteBuf.java` içindeki `readUUID`/`writeUUID`, `readBlockPos`/`writeBlockPos`, `readLongArray`/`writeLongArray`, BitSet'in bu diziye indirgenmiş karşılığı, `core/BlockPos.java` ve `core/SectionPos.java` içindeki `asLong`, `of` ve koordinat çıkarma; `world/level/ChunkPos.java` içindeki packed long dönüşümü incelendi.

Netty buffer/common yanında `core/BlockPos` ve `world/level/ChunkPos` sınıfları için math ailesinin bağımlılıkları ve `commons-lang3` gerekir. Yerel JAR'ın SHA-1 değeri `tests/behavior/reference.json` içine sabitlendi. Yeni oyun bağımlılığı veya Rust kütüphanesi eklenmedi.

## Davranış sözleşmesi

- Vec3.STREAM_CODEC üç ardışık big-endian IEEE-754 f64 taşır; her double okuması ayrı ve atomiktir, yazma x/y/z sırasını korur.
- Vec3.LP_STREAM_CODEC sıfır marker'ı, 15 bit x/y/z quantization alanları, 2 bit ölçek marker'ı ve gerektiğinde VarInt continuation ölçeği kullanır; NaN sıfıra, sonlu değerler mutlak üst sınıra sanitize edilir. Ayrıntılı packed davranış [LP sözleşmesindedir](lp-vec3-contract.md).
- UUID iki big-endian i64 olarak taşınır: önce most, sonra least. Herhangi bir 16 bayt geçerlidir; uzunluk öneki ve sürüm/varyant denetimi yoktur.
- BlockPos tek big-endian i64 olarak taşınır. X/Z 26 bit, Y 12 bit işaretli alanlara maskelenir; aralık dışı i32 girdiler sarar.
- SectionPos tek big-endian i64 olarak taşınır. X/Z 22 bit, Y 20 bit işaretli alanlara maskelenir; aralık dışı girdiler sarar. Okuma tek `readLong` olduğundan eksik veride tüketim olmaz. Açılım aritmetik kaydırma ile işaret genişletir.
- ChunkPos tek big-endian i64 olarak taşınır; x imzalı i32 olarak düşük 32 bite, z imzalı i32 olarak yüksek 32 bite yerleşir ve okuma aynı iki yarıyı işaretli i32 olarak geri verir.
- Okuma her `readLong` için atomiktir: yetersiz veri tüketimi ilerletmez. UUID iki ardışık okumadır; ikinci yarı eksikse ilk 8 bayt tüketilmiş olur. BlockPos, SectionPos ve ChunkPos tek okumadır; eksik veride tüketim olmaz.
- Long array önce VarInt uzunluğu okur. VarInt önek hataları wire tüketimini korur. Negatif uzunluk `NegativeArraySizeException`, kullanılabilir veriyi aşan uzunluk `DecoderException` üretir; ikisi de `decoder_error` kategorisidir ve gövde tüketilmez. Başarılı gövdede bit deseni aynen korunur.
- BitSet, long array gövdesinin `BitSet.valueOf` görünümüdür. Sondaki sıfır long'lar yazmada kısalır; okumada aynı bit kümesi aynı sıralı bit listesini verir.
- Yazma bu dilimde her zaman başarılıdır; uzunluk sınırı ve hata kategorisi yoktur. Mevcut yazma sınırı hatalarında hedef buffer'a yeni veri eklenmez kuralı burada tetiklenmez.
- Sabit BitSet bit boyutu `size` ile taşınır; `ceil(size/8)` bayt okunur/yazılır. Boyut 0 iken tüketim/yazım olmaz. Eksik veri `buffer_underflow`, bit kümesi boyutu aşarsa `encoder_error` üretir; hedefe ekleme yapılmaz.
- Okuma hataları `buffer_underflow`, `decoder_error`, VarInt önekinden gelen `too_big` ve sabit yazmada `encoder_error` kategorileridir. Java exception sınıfı yalnızca `diagnostics` içindedir; mesaj/stack eşitlik ölçütü değildir. NBT kapsam dışıdır.

İlk corpus `prefix_hex` olarak `a55a` kullanır; heap buffer ve sıfırdan farklı reader index ile çalışır. Direct/composite buffer varyantları ve çoklu değer akışları kapsam dışıdır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Vec3 stream codec yazma | 5 | Sıfır, örnek, signed-zero, subnormal/büyük sonlu ve özel değer bileşenleri |
| Vec3 stream codec okuma | 7 | 0/7/8/15/16/23 baytlık kesikler ve tam double sınırları |
| Vec3 stream codec özel değer okuma | 1 | NaN, Infinity ve -Infinity raw bitleri |
| LP Vec3 yazma | 6 | Zero/eşik altı kısa yolu, birim ve partial ölçek, sanitize ve büyük değer clamp'i |
| LP Vec3 okuma | 9 | Zero erken dönüşü, header/continuation underflow'ları ve fazla uzun VarInt |
| UUID yazma | 8 | Sıfır/tüm-bir, örnek, alterne bitler, sürüm benzeri, tek bit ve `deadbeef` desenleri |
| UUID okuma | 9 | 0/1/7/8/9/15 baytlık kesikler, tam 16 bayt ve tüm-bir yük; kalan veri `55aa` ile sınanır |
| BlockPos yazma | 12 | Köken/birim/negatif, yatay/Y sınırları ve sarma komşuları, dünya sınırı, i32 uçları ve karışık koordinat |
| BlockPos okuma | 14 | Bağımsız 8 baytlık yükler ve 0–7 baytlık kesikler; kalan veri `55aa` ile sınanır |
| SectionPos yazma | 12 | Köken/birim/negatif, 22/20 bit sınırları ve sarma komşuları, i32 uçları ve karışık koordinat |
| SectionPos okuma | 12 | Bağımsız 8 baytlık yükler ve 0–7 baytlık kesikler; kalan veri `55aa` ile sınanır |
| ChunkPos yazma | 6 | Köken, birim, negatif ve signed i32 sınır çiftleri; x düşük/z yüksek packed long yerleşimi |
| ChunkPos okuma | 14 | Sıfır, birim, negatif, alternating ve i32 sınır yükleri; 0–7 baytlık kesikler ve kalan veri `55aa` |
| Long array yazma | 6 | Boş, sıfır/örnek, i64 uçları, 7/35 bit sınırları ve `deadbeef` deseni |
| Long array okuma | 17 | Boş/tek/çift yük, kısa/eksik gövde, negatif ve kısa-olmayan sıfır uzunluklar, 127 uzunluk sınırı, 0–6 baytlık kesik önekler ve fazla uzun önek |
| BitSet yazma | 6 | Boş, bit 0, bayt/kelime sınırları, örnek ve sondaki sıfır kırpması |
| BitSet okuma | 6 | Boş/tek/çift kelime, kısa gövde ve negatif uzunluk |
| Sabit BitSet okuma | 8 | Boyut 0/1/8/9/16, yok sayılan yük, kısa/eksik gövde |
| Sabit BitSet yazma | 6 | Boyut 0/8/9/16, sınır içi ve taşan bit kümeleri |

`family: "binary"` altında `codec` `vec3` | `lp_vec3` | `uuid` | `blockpos` | `sectionpos` | `chunkpos` | `longarray` | `bitset` | `fixedbitset` olur. `prefix_hex` küçük harfli bayt hex stringdir. Vec3 yazmada `x`/`y`/`z` 16 haneli küçük harfli raw f64 hex; LP Vec3 yazmada aynı raw f64 alanları; UUID yazmada `most_hex`/`least_hex` 16 haneli küçük harfli hex; BlockPos ve SectionPos yazmada `x`/`y`/`z` işaretli ondalık string; long array yazmada `values` işaretli ondalık string dizisi; BitSet yazmada `bits` bit index dizisidir. Sabit BitSet okuma/yazmada ek olarak `size` bit boyutu bulunur. Okumada `hex` başlık/gövdeyi taşır.

Okuma adaptörü buffer başına `prefix_hex` koyup reader index'i bu alanın sonuna taşır. Başarılı UUID sonucu `most_hex`, `least_hex`; başarılı BlockPos, SectionPos ve ChunkPos sonucu `x`, `y`, `z`; başarılı long array sonucu `values`; başarılı BitSet ve sabit BitSet sonucu `bits` içerir. Her durumda `outcome`, `consumed` ve `remaining_hex` bulunur. Hata sonucu `outcome` ve `error` içerir.

Yazma adaptörü mevcut `prefix_hex` üzerine ekler. Her sonuç `outcome`, eklenen bayt sayısı `written` ve prefix dahil bütün `hex` çıktısını içerir. Sabit BitSet yazmada bit kümesi boyutu aşarsa `encoder_error` döner ve hedefe ekleme yapılmaz; bağımsız okuma senaryoları ayrıdır.

## Çalıştırma ve kanıt

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family binary
python3 tools/behavior/run.py verify --family binary
```

164 senaryo iki ayrı JVM'de aynı sonucu verdi; kayıtlı referans iki yeni JVM ile yeniden doğrulandı. Binary referans kimliği: `faa64f0548d442718b7a8e5783155e27604ddd916aaf9595070c51b91232437e`. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Güncel altı aile corpus'u toplam 1569 Java senaryosundan oluşur ve iki yeni JVM çalıştırmasıyla kayıtla eşitliği doğrulandı. Birleşik referans kimliği: `9f76d4ed1ee97b0d89ef969b0fdf139f7e658f9316895af88b889f790f13b8d1`.

Beklenen çıktılar Rust'a göre değiştirilmedi. ChunkPos partial read senaryolarında 0–5 byte underflow verir; 6–7 byte prefix, fixture içindeki `55aa` suffix ile sekiz byte long'u tamamlayabildiği için Java gibi başarıyla suffix'i tüketir.

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --family timer --family wire --family utf8 --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py compare --family binary --adapter target/debug/examples/behavior_adapter
```

Beklenen çıktılar Rust'a göre değiştirilmedi.
