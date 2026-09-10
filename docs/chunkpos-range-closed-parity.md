# P01.c — ChunkPos rangeClosed doğrulaması

`src/binary.rs`, Java `ChunkPos.rangeClosed` center/radius ve from/to overload'larının inclusive x-fastest/z-next sırasını karşılar. Rust adapter sonucu Java stream'inin tüketilmiş `ChunkPos` listesini aynı decimal string alanlarıyla taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos `rangeClosed` için 8 özgün senaryo hazırlandı.
- Tek nokta, artan ve azalan dikdörtgenler, sabit eksenler ile sıfır/pozitif/negatif radius akışları eşleşti.
- Rust debug ve release adaptörleri güncel **1398/1398** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1900/1900** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1900 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `4cc46837e07b2a435f9a79ffddebd09d7fed0ad74e1310e64453de08cb4bf4bd`. Birleşik referans kimliği: `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea`.

Adapter hash'leri: debug `f2bae96f3cabe37ca34a9b3cf81bc9b3665d184e036fffa35f9ecb63f9605225`, release `74bb0cb21890e68723940edd37c175d52df8d1a610bdacda088a903c66d2fc46`.

Yerel raporlar: `compare-xljwmsb7` (math debug), `compare-s_oqvkq3` (math release), `compare-uf5y7y3a` (birleşik debug), `compare-wrj9a6ex` (birleşik release) ve `verify-5ju3ok97` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, sınırlı ve sonlu aralıklarda iki `ChunkPos.rangeClosed` overload'unun endpoint, sıra ve yön davranışını kapsar. Devasa overflow aralıkları, spliterator estimate/laziness/parallel davranışı, consumer short-circuit işlemleri, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
