# P01.d — AABB toString doğrulaması

`src/aabb.rs`, Java `AABB.toString()` davranışını `Aabb::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` adımını bu API'ye taşır; `VoxelShape.toString()` da aynı AABB biçimlendirme yolunu kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 747 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **747/747** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1201/1201** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint, bilimsel double gösterimi, signed zero, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1201 senaryoluk Java referansını yeniden doğruladı.

Math referansı `6a81202124ebb62fb52fe67c65508a75f8eab5935a82637586267fcd663b1d5a`, birleşik referans `faa59d430a7f5ffebd3b634a6e1cb6970d92ff920794ed661c7eaf6175f68fb7` kimliğindedir.

Yerel raporlar: `compare-id7epnq0` (math debug), `compare-otdg_h7s` (math release), `compare-loq4vhwu` (birleşik debug), `compare-d1k2vrim` (birleşik release) ve `verify-cbpvrvrp` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java constructor'ı ile oluşturulan beş sentetik AABB ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi ve diğer sınıfların metin biçimleri ayrı sözleşmelerdir.
