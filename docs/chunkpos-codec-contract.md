# P01.c — ChunkPos.CODEC sözleşmesi

Bu dilimde **12 Java davranış testi hazırlanmıştır**. Kapsam, Java `ChunkPos.CODEC` sabitinin iki signed int'i x ve z koordinatlarına dönüştürmesi ve bir ChunkPos'u aynı sıradaki iki int listesine kodlamasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:19-22` içindeki codec tanımı ve `util/Util.java:945-951` içindeki `IntStream` sabit boyut kontrolü incelendi. Oracle, `JsonOps` üzerinde Java `ChunkPos.CODEC` çağrısını doğrudan yapar; beklenen sonuçlar Rust çıktısından türetilmez.

Bu dilimde genel JSON sayı türü ayrıştırması ölçülmez. Sentetik decode girdileri signed decimal int metinlerinden oluşturulan Java `JsonArray` değerleridir. Rust üretim API'si JSON veya `serde_json` almaz; sabit liste sözleşmesini i32 slice ve sabit boyutlu dizi ile taşır.

## Davranış sözleşmesi

- Decode girdisi tam iki int içerirse ilk değer x, ikinci değer z olur ve yeni bir ChunkPos üretilir.
- Decode girdisi iki int'ten kısa veya boşsa `Input is not a list of 2 ints` hatası döner ve partial ChunkPos bulunmaz.
- Decode girdisi ikiden uzunsa aynı hata döner; partial değer ilk iki int'ten oluşturulan ChunkPos'tur. Sonraki değerler partial değere girmez.
- Encode sonucu x ve z koordinatlarını aynı sıradaki iki int listesi olarak taşır.
- i32 sınırları ve negatif değerler Java/Rust karşılaştırmasında signed değerler olarak korunur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| Decode tam liste | 5 | Temel, sıfır, negatif/sınır, signed sınır ve alternatif x/z değerleri |
| Decode yanlış boyut | 3 | Boş, kısa ve uzun listeler; hata metni ve uzun listede ilk iki öğenin partial ChunkPos'u |
| Encode | 4 | Temel, sıfır, signed sınır ve alternatif x/z değerleri; iki int'in x/z sırası |

Girdiler `family: "math"`, `operation: "chunkpos_codec"` ve `mode: "decode" | "encode"` kullanır. Decode isteğinde `values` signed decimal int metin listesi; encode isteğinde `x` ve `z` aynı biçimdeki bileşenlerdir. Başarılı decode `result.value` içinde x/z snapshot'ı, başarılı encode iki öğeli int listesi taşır. Hatalı decode `result.outcome`, `result.error` ve uzun liste için `result.partial` alanlarını taşır.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::codec_decode`, tam iki öğeyi kabul eder; yanlış boyutta gerçek liste uzunluğunu ve yalnızca iki veya daha fazla öğe için ilk iki öğeden oluşan partial değeri `ChunkPosCodecError::WrongLength` içinde taşır. `ChunkPos::codec_encode`, bileşenleri `[x, z]` sırasıyla döndürür. `examples/behavior_adapter.rs` bu API'yi JSONL senaryosuna bağlar; `tests/behavior/java/MathOracle.java` Java codec sonucunu doğrudan `DataResult` üzerinden gözlemler.
