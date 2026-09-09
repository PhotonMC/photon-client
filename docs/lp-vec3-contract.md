# P03.b — LP Vec3 stream codec sözleşmesi

Bu dilimin **15 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.LP_STREAM_CODEC` tarafından kullanılan `LpVec3.read/write` packed binary yoludur. Normal `Vec3.STREAM_CODEC` ve JSON `Vec3.CODEC` ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:36`, `network/LpVec3.java:1-93` ve continuation ölçeği için `network/VarInt.java:25-53` incelendi. Oracle doğrudan `Vec3.LP_STREAM_CODEC` çağırır; beklenen baytlar Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Sıfırdan küçük `ABS_MIN_VALUE` chessboard length değeri yalnızca `0` byte'ı yazar; decode bu byte'ı tüketip `Vec3.ZERO` döndürür.
- Diğer yazma yolları x, y ve z'yi `ABS_MAX_VALUE` sınırına clamp eder; NaN bileşenleri pozitif sıfıra çevirir.
- Ölçek `Mth.ceilLong` ile chessboard length'ten seçilir. Ölçeğin düşük iki biti marker içinde taşınır; ölçek 2 bit'e sığmazsa continuation flag ve `(scale >> 2)` signed VarInt eklenir.
- Her bileşen ölçeğe bölünür, `[-1, 1]` aralığı 0–32766 quantized değerine `Math.round` ile taşınır ve x/y/z alanları sırasıyla 3, 18 ve 33 bit offset'leriyle packed edilir.
- Decode ilk byte'ı okur; zero marker'da erken döner. Aksi halde iki header byte'ı, bir big-endian unsigned int ve continuation varsa VarInt ölçeğini okur. Her okuma kendi buffer tüketimini korur.
- Decode quantized alanları 32766 ile sınırlar, `[-1, 1]` aralığına açar ve ölçekle çarpar. Eksik header/VarInt `buffer_underflow`, fazla uzun VarInt `too_big` üretir; önceki byte'lar tüketilmiş kalır.
- LP codec packed çözüm olduğu için normal stream codec'in 24 baytlık raw double davranışını taşımaz.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| LP Vec3 yazma | 6 | Zero ve eşik altı kısa yolu, 2 bit'e sığan ölçek, continuation ölçeği, NaN/Infinity sanitization ve büyük değer clamp'i; prefix dahil packed hex |
| LP Vec3 okuma | 9 | Boş/erken zero, header ve continuation underflow'ları, temel packed değer, continuation payload ve fazla uzun VarInt; tüketim, kalan veri ve Vec3 bitleri |

Girdiler `family: "binary"`, `operation: "read" | "write"` ve `codec: "lp_vec3"` kullanır. Yazmada `x`, `y`, `z` raw f64 bit dizgeleridir. Okumada `hex`, packed byte akışını taşır. Başarılı okumada x/y/z raw f64 bitleri; her okumada `outcome`, `consumed` ve `remaining_hex`; yazmada `outcome`, `written` ve prefix dahil `hex` karşılaştırılır.

## Rust karşılığı

`src/binary.rs` içindeki `read_lp_vec3` ve `write_lp_vec3`, packed header/scale/quantization yollarını uygular. `examples/behavior_adapter.rs` bu API'leri senaryoya bağlar; `tests/behavior/java/BinaryOracle.java` Java LP codec'ini doğrudan çağırır.
