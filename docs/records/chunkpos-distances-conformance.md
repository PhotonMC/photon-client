# P01.c — ChunkPos mesafe yardımcıları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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

`crates/photon-core/src/math.rs` içindeki `abs_max` ve `chessboard_distance`, Java `Math.abs`/`Math.max` ve `Mth.chessboardDistance` i32 davranışını taşır. `crates/photon-core/src/binary.rs` içindeki `ChunkPos::get_chessboard_distance`, `get_chessboard_distance_coordinates`, `distance_squared` ve `distance_squared_from_packed` public overload yollarını birleştirir. `examples/behavior_adapter.rs` JSONL bağını sağlar; `tests/behavior/java/MathOracle.java` Java overload'larını doğrudan gözlemler.


## Client-parity doğrulaması

`crates/photon-core/src/binary.rs` ve `crates/photon-core/src/math.rs`, Java `ChunkPos.getChessboardDistance` ile `distanceSquared` overload'larını signed i32 çıkarma, mutlak değer, kare ve toplam sırasını koruyarak karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk koordinatlarını üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos mesafeleri için 6 özgün senaryo hazırlandı.
- Normal farklar, negatif koordinatlar, çıkarma overflow'u, `Integer.MIN_VALUE` mutlak değer kenarı ve i32 kare/toplam taşmaları Java sonucu ile eşleşti.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` chessboard ve kareli mesafe hesaplarının dört public overload yolunu kapsar. `rangeClosed`, `isValid`, diğer chunk sabitleri, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
