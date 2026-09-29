# P03.b — LP Vec3 stream codec sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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

`crates/voxel-core/src/binary.rs` içindeki `read_lp_vec3` ve `write_lp_vec3`, packed header/scale/quantization yollarını uygular. `examples/behavior_adapter.rs` bu API'leri senaryoya bağlar; `tests/behavior/java/BinaryOracle.java` Java LP codec'ini doğrudan çağırır.


## Client-parity doğrulaması

`crates/voxel-core/src/binary.rs`, Java `Vec3.LP_STREAM_CODEC` tarafından kullanılan packed Vec3 yolunu `read_lp_vec3` ve `write_lp_vec3` API'leriyle uygular. Oracle doğrudan Java codec'ini çağırır; beklenen baytlar Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Binary ailesinin güncel 164 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- LP Vec3 codec'i 6 yazma ve 9 okuma olmak üzere 15 senaryoda debug ve release Rust adaptörleriyle **15/15** eşleşti.
- Yazma karşılaştırması prefix dahil packed byte çıktısını ve eklenen byte sayısını; okuma karşılaştırması Vec3 raw f64 bitlerini, tüketimi, kalan veriyi ve hata kategorisini kapsar.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Binary referans kimliği: `faa64f0548d442718b7a8e5783155e27604ddd916aaf9595070c51b91232437e`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-ijck1p8v` (binary debug), `compare-vuahjf17` (binary release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) karşılaştırmalarında; `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt LP packed header'ının 15 bit x/y/z alanlarını, 2 bit ve continuation VarInt ölçeğini, zero marker'ı, sanitize/clamp/quantization yollarını, header ve VarInt underflow tüketimini ve fazla uzun VarInt hatasını kapsar. Normal `Vec3.STREAM_CODEC`, `Vec3.CODEC`, gerçek packet queue ve packed codec'in test edilmeyen daha geniş ağ bağlamı bu dilimin dışındadır.
