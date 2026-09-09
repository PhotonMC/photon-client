# P01.c — SectionPos packed koordinat erişimcileri sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, packed SectionPos long içinden signed x, y ve z section koordinatlarının çıkarılmasıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:136-145` incelendi. Oracle `SectionPos.x(long)`, `SectionPos.y(long)` ve `SectionPos.z(long)` metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Packed giriş, 64 bit ham değeri temsil eden 16 haneli küçük harfli unsigned hex metnidir. x, bit 42–63 arasındaki signed 22 bit alanından; z, bit 20–41 arasındaki signed 22 bit alanından; y ise bit 0–19 arasındaki signed 20 bit alanından okunur. Her alan Java arithmetic shift sign extension davranışıyla signed i32 değerine taşınır.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Packed alan erişimi | 6 | Sıfır, pozitif ve negatif alanlar, tüm alanların alt/üst sınır desenleri ve alternatif bit deseni |

Girdiler `family: "math"`, `operation: "sectionpos_packed_accessors"` ve 16 haneli `packed` alanını kullanır. `result.value.x`, `result.value.y` ve `result.value.z`, signed decimal string sonuçlarıdır.

## Rust karşılığı

`src/binary.rs` içindeki `SectionPos::get_x`, `get_y` ve `get_z`, packed değerin 22/20/22 bit alanlarını Java ile aynı sign extension işlemiyle döndürür. `SectionPos::unpack` bu erişimcileri kullanır. `examples/behavior_adapter.rs` ham packed hex girdisini bu API'lere bağlar; `tests/behavior/java/MathOracle.java` karşılaştırma oracle'ı olarak Java erişimcilerini çağırır.
