# P01.c — ChunkPos mesafe yardımcıları doğrulaması

`src/binary.rs` ve `src/math.rs`, Java `ChunkPos.getChessboardDistance` ile `distanceSquared` overload'larını signed i32 çıkarma, mutlak değer, kare ve toplam sırasını koruyarak karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk koordinatlarını üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos mesafeleri için 6 özgün senaryo hazırlandı.
- Normal farklar, negatif koordinatlar, çıkarma overflow'u, `Integer.MIN_VALUE` mutlak değer kenarı ve i32 kare/toplam taşmaları Java sonucu ile eşleşti.
- Rust debug ve release adaptörleri güncel **1999/1999** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2501/2501** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2501 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `d0cf0bb203c8ea48ad8144fae6a58022846d1a5cfb95b398018620bae7b0bc3d`. Birleşik referans kimliği: `838ea33f41ccc8a9281f9614ce843c3bb820ec822d6601ba52ebcb496a9f6040`.

Yerel raporlar: `compare-druvbg3h` (math debug), `compare-k2ed8y70` (math release), `compare-h1kbcaxz` (birleşik debug), `compare-js7_mas4` (birleşik release) ve `verify-d2ptfzig` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` chessboard ve kareli mesafe hesaplarının dört public overload yolunu kapsar. `rangeClosed`, `isValid`, diğer chunk sabitleri, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
