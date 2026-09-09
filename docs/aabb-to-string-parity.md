# P01.d — AABB toString doğrulaması

`src/aabb.rs`, Java `AABB.toString()` davranışını `Aabb::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` adımını bu API'ye taşır; `VoxelShape.toString()` da aynı AABB biçimlendirme yolunu kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 771 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **771/771** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1225/1225** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint, bilimsel double gösterimi, signed zero, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1225 senaryoluk Java referansını yeniden doğruladı.

Math referansı `ffefa3418c35a5dbf05a16b8c6bb664f38665423ada7b039029bea24f0e3f8ac`, birleşik referans `c8d3f97682c7983d7226696163a3943fe4ef6723170037ad5205d10ed0494654` kimliğindedir.

Yerel raporlar: `compare-nyuxtnwe` (math debug), `compare-c83i9uhh` (math release), `compare-2v85255z` (birleşik debug), `compare-4_se5ekn` (birleşik release) ve `verify-q8mzlbkq` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java constructor'ı ile oluşturulan beş sentetik AABB ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi ve diğer sınıfların metin biçimleri ayrı sözleşmelerdir.
