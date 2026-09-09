# P01.c — ChunkPos blok koordinatı doğrulaması

`src/binary.rs`, Java `ChunkPos` blok koordinatı yardımcılarını aynı i32 kaydırma ve toplama kurallarıyla karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik x/z/offset girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos blok koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Minimum, orta, maksimum ve offset'li koordinatlar x/z sırasını korudu; negatif chunk koordinatları, shift taşması ve offset toplama taşması Java i32 sarma davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **983/983** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1485/1485** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1485 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `420532c31a46dc73c7456731fb356a539ca36511c8ae3b916c7b882d3703a8e5`. Birleşik referans kimliği: `7868dda2fe1dd0a7fbb9e78e734b7ecf0ffc505e37763257267a72556cdbc671`.

Yerel raporlar: `compare-wkwr1b75` (math debug), `compare-4mcmj101` (math release), `compare-3w52gug0` (birleşik debug), `compare-09rh6rbh` (birleşik release) ve `verify-dcqd08ai` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos`'un blok x/z alt-orta-üst uçlarını ve offset'li blok koordinatlarını kapsar. Region koordinatları, `BlockPos` üretimi, contains/distance/range yardımcıları, stream/JSON codec'leri ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
