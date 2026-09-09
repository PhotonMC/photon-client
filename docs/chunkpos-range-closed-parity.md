# P01.c — ChunkPos rangeClosed doğrulaması

`src/binary.rs`, Java `ChunkPos.rangeClosed` center/radius ve from/to overload'larının inclusive x-fastest/z-next sırasını karşılar. Rust adapter sonucu Java stream'inin tüketilmiş `ChunkPos` listesini aynı decimal string alanlarıyla taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos `rangeClosed` için 8 özgün senaryo hazırlandı.
- Tek nokta, artan ve azalan dikdörtgenler, sabit eksenler ile sıfır/pozitif/negatif radius akışları eşleşti.
- Rust debug ve release adaptörleri güncel **1031/1031** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1533/1533** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1533 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `cbae17cd88ce5b1617d01034d27ad104db9b471eefc8bebf83ed4e8d1dc0aa70`. Birleşik referans kimliği: `338731e44c45c8acc6fbd70e7fe1439cdaa28ec5996f6a4d9517412cc50106a0`.

Adapter hash'leri: debug `f2bae96f3cabe37ca34a9b3cf81bc9b3665d184e036fffa35f9ecb63f9605225`, release `74bb0cb21890e68723940edd37c175d52df8d1a610bdacda088a903c66d2fc46`.

Yerel raporlar: `compare-2ftkouxd` (math debug), `compare-g204xxpd` (math release), `compare-ag9x0lbh` (birleşik debug), `compare-sl2ed6ln` (birleşik release) ve `verify-5bpx38hs` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, sınırlı ve sonlu aralıklarda iki `ChunkPos.rangeClosed` overload'unun endpoint, sıra ve yön davranışını kapsar. Devasa overflow aralıkları, spliterator estimate/laziness/parallel davranışı, consumer short-circuit işlemleri, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
