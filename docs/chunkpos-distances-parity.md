# P01.c — ChunkPos mesafe yardımcıları doğrulaması

`src/binary.rs` ve `src/math.rs`, Java `ChunkPos.getChessboardDistance` ile `distanceSquared` overload'larını signed i32 çıkarma, mutlak değer, kare ve toplam sırasını koruyarak karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk koordinatlarını üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos mesafeleri için 6 özgün senaryo hazırlandı.
- Normal farklar, negatif koordinatlar, çıkarma overflow'u, `Integer.MIN_VALUE` mutlak değer kenarı ve i32 kare/toplam taşmaları Java sonucu ile eşleşti.
- Rust debug ve release adaptörleri güncel **1653/1653** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2155/2155** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2155 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `e9b1c1e55b2a03528b712cd304101c7b36378556372ebad60bb330a4a60eb192`. Birleşik referans kimliği: `2a68356f797e134e8a268355635fffb2db6a83b94e11d3afbdf308978093028e`.

Yerel raporlar: `compare-ceusm4y4` (math debug), `compare-hconpy1a` (math release), `compare-rhjx0z7e` (birleşik debug), `compare-jtl411eu` (birleşik release) ve `verify-u5d5zd4c` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` chessboard ve kareli mesafe hesaplarının dört public overload yolunu kapsar. `rangeClosed`, `isValid`, diğer chunk sabitleri, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
