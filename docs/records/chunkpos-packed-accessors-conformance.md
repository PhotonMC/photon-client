# P01.c — ChunkPos packed koordinat erişimcileri sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, packed ChunkPos long içinden signed x ve z koordinatlarının çıkarılmasıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/world/level/ChunkPos.java:85-91` incelendi. Oracle `ChunkPos.getX(long)` ve `ChunkPos.getZ(long)` metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Packed giriş, 64 bit ham değeri temsil eden 16 haneli küçük harfli unsigned hex metnidir. Düşük 32 bit alan x, yüksek 32 bit alan z olarak okunur; her alan Java `int` dönüşümüyle signed i32 değerine taşınır.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Packed alan erişimi | 6 | Sıfır, pozitif ve negatif alanlar, iki signed i32 sınır kombinasyonu ve alternatif bit deseni |

Girdiler `family: "math"`, `operation: "chunkpos_packed_accessors"` ve 16 haneli `packed` alanını kullanır. `result.value.x` ve `result.value.z`, signed decimal string sonuçlarıdır.

## Rust karşılığı

`crates/photon-core/src/binary.rs` içindeki `ChunkPos::get_x` ve `ChunkPos::get_z`, packed değerin iki 32 bit alanını Java ile aynı signed yorumla döndürür. `examples/behavior_adapter.rs` ham packed hex girdisini bu API’ye bağlar; `tests/behavior/java/MathOracle.java` karşılaştırma oracle’ı olarak Java erişimcilerini çağırır.


## Client-parity doğrulaması

`crates/photon-core/src/binary.rs`, packed ChunkPos long içindeki düşük ve yüksek 32 bit alanlarını Java `ChunkPos.getX(long)` ve `getZ(long)` ile aynı signed i32 sonuçlara çıkarır. Rust adapter ham packed hex girdisini bu API’ye bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos packed koordinat erişimcileri için 6 özgün senaryo hazırlandı.
- Sıfır, pozitif/negatif alanlar, iki signed i32 sınır yerleşimi ve alternatif bit deseni doğrudan Java/Rust API’leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hash’leri: debug `fe351e4afa245b39c89bece68a6dfdb02699933b5f980b5b82214942e923dc4d`, release `31dad93777713c698bdc325bea831bfa6833f276b66b5d5d665b91c0421b8bf6`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, ham packed long alanlarından x/z çıkarma ve signed i32 yorumuyla sınırlıdır. Packed değer üreticileri, SectionPos dönüşümleri, stream codec tüketimi ve bu accessor’ların daha geniş istemci akışlarındaki kullanımı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
