# P01.c — ChunkPos rangeClosed doğrulaması

`src/binary.rs`, Java `ChunkPos.rangeClosed` center/radius ve from/to overload'larının inclusive x-fastest/z-next sırasını karşılar. Rust adapter sonucu Java stream'inin tüketilmiş `ChunkPos` listesini aynı decimal string alanlarıyla taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos `rangeClosed` için 8 özgün senaryo hazırlandı.
- Tek nokta, artan ve azalan dikdörtgenler, sabit eksenler ile sıfır/pozitif/negatif radius akışları eşleşti.
- Rust debug ve release adaptörleri güncel **1249/1249** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1751/1751** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1751 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `29be5693737a944f40d9534d4a126556f5aeaac24dd9d2f5fccd0c6b1db3a3aa`. Birleşik referans kimliği: `a92e7ef25d7ffcf101102bab4b05061d96a3b0511495a77464f282a0775abfd0`.

Adapter hash'leri: debug `f2bae96f3cabe37ca34a9b3cf81bc9b3665d184e036fffa35f9ecb63f9605225`, release `74bb0cb21890e68723940edd37c175d52df8d1a610bdacda088a903c66d2fc46`.

Yerel raporlar: `compare-dj1scyn3` (math debug), `compare-1g72i2us` (math release), `compare-f0bjlm2c` (birleşik debug), `compare-eaqfihsj` (birleşik release) ve `verify-p2v9lu2o` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, sınırlı ve sonlu aralıklarda iki `ChunkPos.rangeClosed` overload'unun endpoint, sıra ve yön davranışını kapsar. Devasa overflow aralıkları, spliterator estimate/laziness/parallel davranışı, consumer short-circuit işlemleri, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
