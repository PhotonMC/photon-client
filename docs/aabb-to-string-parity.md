# P01.d — AABB toString doğrulaması

`src/aabb.rs`, Java `AABB.toString()` davranışını `Aabb::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` adımını bu API'ye taşır; `VoxelShape.toString()` da aynı AABB biçimlendirme yolunu kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 953 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **953/953** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1455/1455** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint, bilimsel double gösterimi, signed zero, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1455 senaryoluk Java referansını yeniden doğruladı.

Math referansı `c05b4f6d5d95b034d9034387072c87ac36f1b31427b1fbbbbebe885dec1bfc3b`, birleşik referans `e267ec135794f878c9e54d22faa3451ffe125c25ad17b7ad67e223c1d1fb3b55` kimliğindedir.

Yerel raporlar: `compare-j1r42nvx` (math debug), `compare-yls3sbu1` (math release), `compare-ttkrbj5a` (birleşik debug), `compare-dz8xzceq` (birleşik release) ve `verify-zgd9uosd` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java constructor'ı ile oluşturulan beş sentetik AABB ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi ve diğer sınıfların metin biçimleri ayrı sözleşmelerdir.
