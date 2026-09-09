# P01.d — AABB toString doğrulaması

`src/aabb.rs`, Java `AABB.toString()` davranışını `Aabb::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` adımını bu API'ye taşır; `VoxelShape.toString()` da aynı AABB biçimlendirme yolunu kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 675 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **675/675** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1129/1129** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint, bilimsel double gösterimi, signed zero, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1129 senaryoluk Java referansını yeniden doğruladı.

Math referansı `4c3937883bf18dab7461a7bf7250926a138778f87dbcaa0127ae5950d94de7f4`, birleşik referans `cde30e979941342736ed7dcc8995baa3a0f857ed47dbf23c18cad23127043901` kimliğindedir.

Yerel raporlar: `compare-d110h2ng` (math debug), `compare-t3jgwbff` (math release), `compare-lkmx56jk` (birleşik debug), `compare-666jyr9x` (birleşik release) ve `verify-6unzhzb2` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java constructor'ı ile oluşturulan beş sentetik AABB ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi ve diğer sınıfların metin biçimleri ayrı sözleşmelerdir.
