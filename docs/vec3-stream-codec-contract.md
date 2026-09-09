# P03.b — Vec3 stream codec sözleşmesi

Bu dilimin **13 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.STREAM_CODEC` içindeki üç ardışık `ByteBuf` double okuma ve yazma yoludur. `Vec3.CODEC` ve packed `LP_STREAM_CODEC` bu dilime dahil değildir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:25-33` incelendi. `decode`, `readDouble()` çağrılarını x, y, z sırasıyla yapar; `encode`, aynı sırada `writeDouble()` çağrılarını yapar. Oracle doğrudan `Vec3.STREAM_CODEC` çağırır; beklenen baytlar Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Codec üç Vec3 bileşenini uzunluk öneki olmadan, x/y/z sırasıyla big-endian IEEE-754 double olarak taşır.
- Yazma, Vec3 bileşenlerinin ham f64 bitlerini 24 bayta ekler; `prefix_hex` yalnızca buffer konumunu ve yazılan toplam çıktıyı gözlemlemek için kullanılır.
- Decode her `readDouble()` çağrısını ayrı bir 8 baytlık buffer okuması olarak yürütür. İlk, ikinci veya üçüncü double eksikse ilgili çağrı `buffer_underflow` üretir.
- Önceki tam double okumaları hata anında tüketilmiş kalır; eksik olan double'ın mevcut parçaları tüketilmez. Başarılı okumada yalnızca kalan suffix okunmamış kalır.
- NaN, Infinity, signed zero, subnormal ve en büyük sonlu değer bitleri snapshot'ta korunur.
- `LP_STREAM_CODEC`, packed ölçek ve VarInt davranışına sahip ayrı bir kaynak dilimidir; burada kapsam dışıdır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| Vec3 yazma | 5 | Sıfır, örnek, signed-zero, subnormal/büyük sonlu ve özel değer bileşenleri; prefix dahil hex ve yazılan bayt sayısı |
| Vec3 okuma | 7 | 0/7/8/15/16/23 baytlık kesikler ve tam 24 bayt; tüketim, kalan hex ve hata/Vec3 bitleri |
| Özel değer okuma | 1 | NaN, Infinity ve -Infinity raw bitleri; kalan suffix ile birlikte |

Girdiler `family: "binary"`, `operation: "read" | "write"` ve `codec: "vec3"` kullanır. Yazmada `x`, `y`, `z` onaltılık f64 bit dizgeleridir. Okumada `hex`, üç double payload'ını taşır; tam double sınırlarından sonra kalan veriyi görünür kılmak için, eksik double'ı tamamlamayacak durumlarda `55aa` suffix'i eklenir. Başarılı okumada x/y/z ham f64 bitleri; her okumada `outcome`, `consumed` ve `remaining_hex`; yazmada `outcome`, `written` ve prefix dahil `hex` karşılaştırılır.

## Rust karşılığı

`src/binary.rs` içindeki `read_vec3` ve `write_vec3`, her bileşeni ayrı big-endian f64 okuma/yazma olarak uygular. `examples/behavior_adapter.rs` bu API'leri doğrudan senaryoya bağlar; `tests/behavior/java/BinaryOracle.java` Java stream codec'i doğrudan çağırır.
