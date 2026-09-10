# P01.d — AABB toString doğrulaması

`src/aabb.rs`, Java `AABB.toString()` davranışını `Aabb::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` adımını bu API'ye taşır; `VoxelShape.toString()` da aynı AABB biçimlendirme yolunu kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1703 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1703/1703** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2205/2205** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint, bilimsel double gösterimi, signed zero, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 2205 senaryoluk Java referansını yeniden doğruladı.

Math referansı `b93cda4c5257c03011378a24fe916e8b8cd7e11d2fe75e1a1de108e4b4ebd630`, birleşik referans `413b981c2a619373545c6c4c74d3e7fec6b1a2e1dc34d5a49ee2f4eed5231162` kimliğindedir.

Yerel raporlar: `compare-a8qhmnrl` (math debug), `compare-usm27y8z` (math release), `compare-8vwtvwwl` (birleşik debug), `compare-uey51w68` (birleşik release) ve `verify-l3irw60r` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java constructor'ı ile oluşturulan beş sentetik AABB ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi ve diğer sınıfların metin biçimleri ayrı sözleşmelerdir.
