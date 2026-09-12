# P01.c — ChunkPos rangeClosed sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **8 Java davranış testi hazırlanmıştır**. Kapsam, `ChunkPos.rangeClosed` center/radius ve from/to overload'larının sınırlı, sonlu aralık çıktısıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/world/level/ChunkPos.java:203-237` incelendi. Oracle Java stream'ini doğrudan tüketir; beklenen sonuçlar Rust çıktısından türetilmez.

- Center/radius overload'u Java signed i32 çıkarma ve toplama ile from/to uçlarını kurar.
- From/to uçları dahil edilir.
- X ekseni hızlı değişir; X tamamlanınca satırın başlangıç X'i kullanılarak Z bir yönde ilerler.
- Her eksenin yönü, from koordinatı to koordinatından küçükse `+1`, aksi halde `-1` olur. Eşit eksenlerde bu nedenle ilerleme yapılmaz.
- Negatif radius, endpoint aritmetiğinden sonra oluşan aralığın azalan yönde tüketilmesini korur.

Bu dilim devasa overflow aralıklarını, spliterator estimate değerini, stream tembelliğini, paralel stream davranışını, consumer short-circuit davranışını ve chunk yaşam döngüsünü ölçmez.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| from/to | 5 | Tek nokta, artan/azalan dikdörtgen, sabit X ve sabit Z |
| center/radius | 3 | Sıfır, pozitif ve negatif radius |

Girdiler `family: "math"`, `operation: "chunkpos_range_closed"` ve `mode: "from-to"` veya `mode: "center-radius"` alanlarını kullanır. Koordinatlar signed decimal i32 metinleridir. `from-to` için `from_x`, `from_z`, `to_x`, `to_z`; center/radius için `x`, `z`, `radius` alanları taşınır. `result.value.positions`, her ChunkPos'u decimal string `x` ve `z` alanlarıyla sırayla taşır.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::range_closed` ve `range_closed_from_to`, endpoint aritmetiğini ve Java'nın x-fastest/z-next traversal sırasını üretir. `examples/behavior_adapter.rs` iki overload'u JSONL operation'ına bağlar; `tests/behavior/java/MathOracle.java` Java stream çıktısını gözlemler.


## Client-parity doğrulaması

`src/binary.rs`, Java `ChunkPos.rangeClosed` center/radius ve from/to overload'larının inclusive x-fastest/z-next sırasını karşılar. Rust adapter sonucu Java stream'inin tüketilmiş `ChunkPos` listesini aynı decimal string alanlarıyla taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos `rangeClosed` için 8 özgün senaryo hazırlandı.
- Tek nokta, artan ve azalan dikdörtgenler, sabit eksenler ile sıfır/pozitif/negatif radius akışları eşleşti.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hash'leri: debug `f2bae96f3cabe37ca34a9b3cf81bc9b3665d184e036fffa35f9ecb63f9605225`, release `74bb0cb21890e68723940edd37c175d52df8d1a610bdacda088a903c66d2fc46`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, sınırlı ve sonlu aralıklarda iki `ChunkPos.rangeClosed` overload'unun endpoint, sıra ve yön davranışını kapsar. Devasa overflow aralıkları, spliterator estimate/laziness/parallel davranışı, consumer short-circuit işlemleri, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
