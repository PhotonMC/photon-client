# P01.d — AABB toString doğrulaması

`src/aabb.rs`, Java `AABB.toString()` davranışını `Aabb::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` adımını bu API'ye taşır; `VoxelShape.toString()` da aynı AABB biçimlendirme yolunu kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1113 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1113/1113** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1615/1615** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint, bilimsel double gösterimi, signed zero, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1615 senaryoluk Java referansını yeniden doğruladı.

Math referansı `23d8549371228c5fe370021e1df1ce61a2d1869ad767cf1a029bcbf1930b5ac9`, birleşik referans `747c3645458af2ae3b9de41407461996409d550b8fca05e990e18b3b1983f461` kimliğindedir.

Yerel raporlar: `compare-y28xdodh` (math debug), `compare-_7cfq3in` (math release), `compare-cxv1an3r` (birleşik debug), `compare-1jjs0aun` (birleşik release) ve `verify-71u5jh9x` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java constructor'ı ile oluşturulan beş sentetik AABB ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi ve diğer sınıfların metin biçimleri ayrı sözleşmelerdir.
