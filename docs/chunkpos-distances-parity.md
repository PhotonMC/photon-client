# P01.c — ChunkPos mesafe yardımcıları doğrulaması

`src/binary.rs` ve `src/math.rs`, Java `ChunkPos.getChessboardDistance` ile `distanceSquared` overload'larını signed i32 çıkarma, mutlak değer, kare ve toplam sırasını koruyarak karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk koordinatlarını üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos mesafeleri için 6 özgün senaryo hazırlandı.
- Normal farklar, negatif koordinatlar, çıkarma overflow'u, `Integer.MIN_VALUE` mutlak değer kenarı ve i32 kare/toplam taşmaları Java sonucu ile eşleşti.
- Rust debug ve release adaptörleri güncel **1001/1001** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1503/1503** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1503 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `7d65abdf4967bf0f7585e849fc4e6783027eaef1ef313c8b7d22e85922955d4a`. Birleşik referans kimliği: `2411fed740f5d2548545a86cee882fb42daaf0d9fbededf1b224dce54a261a5d`.

Yerel raporlar: `compare-12czaqmk` (math debug), `compare-fm0qnyjh` (math release), `compare-sakmrbet` (birleşik debug), `compare-pd6md7hs` (birleşik release) ve `verify-5t2xrfy6` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` chessboard ve kareli mesafe hesaplarının dört public overload yolunu kapsar. `rangeClosed`, `isValid`, diğer chunk sabitleri, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
