# P01.d — AABB toString doğrulaması

`src/aabb.rs`, Java `AABB.toString()` davranışını `Aabb::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` adımını bu API'ye taşır; `VoxelShape.toString()` da aynı AABB biçimlendirme yolunu kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 854 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **854/854** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1308/1308** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint, bilimsel double gösterimi, signed zero, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1308 senaryoluk Java referansını yeniden doğruladı.

Math referansı `0ba4ea9be106c1ef86d8a7180620724b6127bb00eb89447c2677101d0067f400`, birleşik referans `94ba7a39728d11e2636807a8347713b80c9849396db8a78a6093edab3bc18c80` kimliğindedir.

Yerel raporlar: `compare-o4ywsb7q` (math debug), `compare-7enf_yu2` (math release), `compare-dheuiwbd` (birleşik debug), `compare-vkmyk8cn` (birleşik release) ve `verify-32kafz9f` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java constructor'ı ile oluşturulan beş sentetik AABB ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi ve diğer sınıfların metin biçimleri ayrı sözleşmelerdir.
