# P01.c — ChunkPos mesafe yardımcıları sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, bir `ChunkPos` ile başka bir chunk arasındaki chessboard mesafesini ve wrapping signed i32 kareli mesafeyi, Java'nın bütün public overload yollarıyla gözlemlemektir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:181-201` içindeki `getChessboardDistance` ve `distanceSquared` yolları ile `util/Mth.java:126-139` içindeki `absMax` ve `chessboardDistance` yolu incelendi. Oracle, Java metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Bu dilimde `rangeClosed`, `isValid`, chunk yaşam döngüsü ve istemciye bağlı chunk davranışları ölçülmez.

## Davranış sözleşmesi

- `getChessboardDistance(ChunkPos)`, diğer chunk'ın x/z değerlerini koordinat overload'una iletir; sonuç `Mth.chessboardDistance` yolundan gelir.
- Chessboard mesafesi, iki signed i32 farkının Java `Math.abs` sonuçlarının `Math.max` değeridir. Fark çıkarma i32 içinde sarar; `Integer.MIN_VALUE` için Java `Math.abs` yine `Integer.MIN_VALUE` döndürür.
- `distanceSquared(ChunkPos)`, iki signed i32 farkını ayrı ayrı hesaplar, her farkı i32 olarak karesini alır ve kareleri i32 toplamıyla birleştirir. Çıkarma, çarpma ve toplama taşmaları Java gibi sarar.
- `distanceSquared(long)`, packed chunk'ın düşük 32 bit x ve yüksek 32 bit z alanlarını açıp aynı kareli mesafe yolunu kullanır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| ChunkPos mesafeleri | 6 | Köken, temel/negatif eksenler, eksen baskın fark, çıkarma overflow'u ve `Integer.MIN_VALUE` kenarı |

Girdiler `family: "math"`, `operation: "chunkpos_distances"` ve signed decimal i32 `x`, `z`, `other_x`, `other_z` alanlarını kullanır. `value` altında ChunkPos overload'ları ve packed long overload'u için dört signed decimal string sonuç bulunur: `chessboard_chunk`, `chessboard_coordinates`, `distance_squared_chunk` ve `distance_squared_packed`.

## Rust karşılığı

`src/math.rs` içindeki `abs_max` ve `chessboard_distance`, Java `Math.abs`/`Math.max` ve `Mth.chessboardDistance` i32 davranışını taşır. `src/binary.rs` içindeki `ChunkPos::get_chessboard_distance`, `get_chessboard_distance_coordinates`, `distance_squared` ve `distance_squared_from_packed` public overload yollarını birleştirir. `examples/behavior_adapter.rs` JSONL bağını sağlar; `tests/behavior/java/MathOracle.java` Java overload'larını doğrudan gözlemler.
