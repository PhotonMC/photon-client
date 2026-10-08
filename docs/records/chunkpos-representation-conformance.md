# P01.c — ChunkPos temsil ve hash sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, `ChunkPos` record'unun Java metin temsilini, instance `hashCode()` sonucunu ve iki signed koordinat alanı için static hash fonksiyonunu taşımaktır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:93-102` içindeki hash yolları ve `world/level/ChunkPos.java:172-175` içindeki `toString()` metodu incelendi. Oracle, Java metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Bu dilimde record equality, packing/codec'ler, region/block yardımcıları, range/distance ve istemciye bağlı chunk davranışları ölçülmez.

## Davranış sözleşmesi

- `toString()`, x ve z signed decimal değerlerini `[x, z]` biçiminde, araya virgül ve boşluk koyarak üretir.
- `hashCode()`, aynı x/z değerlerini static hash yoluna verir.
- Static `hash(x, z)`, x ve z için sabit çarpan/toplama dönüşümlerini XOR'lar.
- Java int çarpma ve toplama taşmaları i32 içinde sarar; XOR signed bit desenini korur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| Temsil ve hash | 6 | Köken, negatif değerler, signed sınırlar, alternating bit desenleri ve overflow girdileri |

Girdiler `family: "math"`, `operation: "chunkpos_representation"`, `x`, `z`, `hash_x` ve `hash_z` signed decimal i32 metinlerini kullanır. `value` altında Java metin temsili, instance hash ve static hash sonucu decimal string olarak taşınır.

## Rust karşılığı

`crates/photon-core/src/binary.rs` içindeki `ChunkPos::to_string_java`, `hash_code_java` ve `hash` Java'nın temsil biçimini ve wrapping hash aritmetiğini korur. `examples/behavior_adapter.rs` bu API'yi JSONL senaryosuna bağlar; `tests/behavior/java/MathOracle.java` Java çıktısını doğrudan gözlemler.


## Client-parity doğrulaması

`crates/photon-core/src/binary.rs`, Java `ChunkPos.toString()`, `hashCode()` ve static `hash` davranışlarını aynı metin ve i32 wrapping aritmetiğiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik signed koordinat girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Temsil ve hash için 6 özgün senaryo hazırlandı.
- `[x, z]` metin biçimi, instance/static hash yolu, signed sınırlar ve çarpma/toplama overflow davranışı Java i32 sonucuyla eşleşti.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` metin temsilini ve iki hash yolunu kapsar. Record equality, packing/codec'ler, region/block/BlockPos yardımcıları, range/distance ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
