# P03.b — Vec3 stream codec doğrulaması

`src/binary.rs`, Java `Vec3.STREAM_CODEC` için üç ardışık big-endian f64 okuma ve yazma yolunu sunar. Java oracle doğrudan `Vec3.STREAM_CODEC` çağırır; Rust adaptörü `read_vec3` ve `write_vec3` üretim API'lerini çağırır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Binary ailesinin 129 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **129/129** binary senaryosunda eşleşti.
- Beş yazma senaryosu 24 baytlık x/y/z sırasını, prefix'i ve raw f64 bitlerini karşılaştırdı.
- Yedi okuma sınırı ilk, ikinci ve üçüncü double underflow'unda tüketim davranışını; tam okuma ve NaN/Infinity raw bitleri başarı yolunu karşılaştırdı.
- Birleşik altı aile corpus'u **1821/1821** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1821 senaryoluk Java referansını yeniden doğruladı.

Binary referansı `40e3640839f4783a6a3f1b08072a8e7679461e101d4f72b5133ddfe58e06e3d0`, birleşik referans `01a770901948bb0f5650c8b4e80bf54b513d904c9746232c38b93c03a7546957` kimliklerindedir.

Yerel raporlar: `compare-q8wuyafn` (binary debug), `compare-kj2w1vpa` (binary release), `compare-olmqgxpc` (birleşik debug), `compare-jxg0__tj` (birleşik release) ve `verify-m0natom3` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt üç double'ın big-endian stream codec'i, bileşen başına buffer tüketimi ve bu girdilerdeki raw f64 bitleriyle sınırlıdır. `Vec3.CODEC`, `LP_STREAM_CODEC` ve gerçek paket kuyruğu sonraki veri dilimleridir.
