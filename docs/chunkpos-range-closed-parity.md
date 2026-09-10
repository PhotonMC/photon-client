# P01.c — ChunkPos rangeClosed doğrulaması

`src/binary.rs`, Java `ChunkPos.rangeClosed` center/radius ve from/to overload'larının inclusive x-fastest/z-next sırasını karşılar. Rust adapter sonucu Java stream'inin tüketilmiş `ChunkPos` listesini aynı decimal string alanlarıyla taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos `rangeClosed` için 8 özgün senaryo hazırlandı.
- Tek nokta, artan ve azalan dikdörtgenler, sabit eksenler ile sıfır/pozitif/negatif radius akışları eşleşti.
- Rust debug ve release adaptörleri güncel **1534/1534** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2036/2036** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2036 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `99f42388f3562f04669e01818a39202ee8f698f0151f153acf5352e96f0fcc58`. Birleşik referans kimliği: `5607afc8c59b5ee3393ed20e95d9fbca8ce40846df423d5e52589b902d90b90f`.

Adapter hash'leri: debug `f2bae96f3cabe37ca34a9b3cf81bc9b3665d184e036fffa35f9ecb63f9605225`, release `74bb0cb21890e68723940edd37c175d52df8d1a610bdacda088a903c66d2fc46`.

Yerel raporlar: `compare-t6yvipac` (math debug), `compare-6_snmnk3` (math release), `compare-tgx0qoj4` (birleşik debug), `compare-t1bsi_04` (birleşik release) ve `verify-zwp9iban` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, sınırlı ve sonlu aralıklarda iki `ChunkPos.rangeClosed` overload'unun endpoint, sıra ve yön davranışını kapsar. Devasa overflow aralıkları, spliterator estimate/laziness/parallel davranışı, consumer short-circuit işlemleri, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
