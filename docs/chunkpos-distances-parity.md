# P01.c — ChunkPos mesafe yardımcıları doğrulaması

`src/binary.rs` ve `src/math.rs`, Java `ChunkPos.getChessboardDistance` ile `distanceSquared` overload'larını signed i32 çıkarma, mutlak değer, kare ve toplam sırasını koruyarak karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk koordinatlarını üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos mesafeleri için 6 özgün senaryo hazırlandı.
- Normal farklar, negatif koordinatlar, çıkarma overflow'u, `Integer.MIN_VALUE` mutlak değer kenarı ve i32 kare/toplam taşmaları Java sonucu ile eşleşti.
- Rust debug ve release adaptörleri güncel **1398/1398** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1900/1900** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1900 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `4cc46837e07b2a435f9a79ffddebd09d7fed0ad74e1310e64453de08cb4bf4bd`. Birleşik referans kimliği: `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea`.

Yerel raporlar: `compare-xljwmsb7` (math debug), `compare-s_oqvkq3` (math release), `compare-uf5y7y3a` (birleşik debug), `compare-wrj9a6ex` (birleşik release) ve `verify-5ju3ok97` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` chessboard ve kareli mesafe hesaplarının dört public overload yolunu kapsar. `rangeClosed`, `isValid`, diğer chunk sabitleri, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
