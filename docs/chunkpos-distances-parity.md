# P01.c — ChunkPos mesafe yardımcıları doğrulaması

`src/binary.rs` ve `src/math.rs`, Java `ChunkPos.getChessboardDistance` ile `distanceSquared` overload'larını signed i32 çıkarma, mutlak değer, kare ve toplam sırasını koruyarak karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk koordinatlarını üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos mesafeleri için 6 özgün senaryo hazırlandı.
- Normal farklar, negatif koordinatlar, çıkarma overflow'u, `Integer.MIN_VALUE` mutlak değer kenarı ve i32 kare/toplam taşmaları Java sonucu ile eşleşti.
- Rust debug ve release adaptörleri güncel **1548/1548** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2050/2050** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2050 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `8d3d3f4690e190086d874fa8e58bdb837efcc4649b07170de736639c38e7aa7d`. Birleşik referans kimliği: `0d4c76950a37451c226bb147399cfa08349c81c96e4a8ea2b85e47696f0caf11`.

Yerel raporlar: `compare-m8bramo3` (math debug), `compare-20kwpz7m` (math release), `compare-hmu92wq0` (birleşik debug), `compare-ha45dyno` (birleşik release) ve `verify-46sjfnqt` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` chessboard ve kareli mesafe hesaplarının dört public overload yolunu kapsar. `rangeClosed`, `isValid`, diğer chunk sabitleri, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
