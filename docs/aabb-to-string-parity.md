# P01.d — AABB toString doğrulaması

`src/aabb.rs`, Java `AABB.toString()` davranışını `Aabb::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` adımını bu API'ye taşır; `VoxelShape.toString()` da aynı AABB biçimlendirme yolunu kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 705 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **705/705** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1159/1159** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint, bilimsel double gösterimi, signed zero, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1159 senaryoluk Java referansını yeniden doğruladı.

Math referansı `ee70cefd63c52a2329bae4fec46f29bfb59aefb64f7f48829c4a516dc7685fb9`, birleşik referans `eb9f0578dc53caaa0fab43aebfbe945ab957c5f68d0e3eb2d790a5f4fc1c92f2` kimliğindedir.

Yerel raporlar: `compare-g26jdt9a` (math debug), `compare-wlw80e9f` (math release), `compare-l4qzoucn` (birleşik debug), `compare-ixiuyjvh` (birleşik release) ve `verify-xucqlaod` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java constructor'ı ile oluşturulan beş sentetik AABB ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi ve diğer sınıfların metin biçimleri ayrı sözleşmelerdir.
