# P01.c — ChunkPos mesafe yardımcıları doğrulaması

`src/binary.rs` ve `src/math.rs`, Java `ChunkPos.getChessboardDistance` ile `distanceSquared` overload'larını signed i32 çıkarma, mutlak değer, kare ve toplam sırasını koruyarak karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk koordinatlarını üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos mesafeleri için 6 özgün senaryo hazırlandı.
- Normal farklar, negatif koordinatlar, çıkarma overflow'u, `Integer.MIN_VALUE` mutlak değer kenarı ve i32 kare/toplam taşmaları Java sonucu ile eşleşti.
- Rust debug ve release adaptörleri güncel **1743/1743** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2245/2245** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2245 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `6b06e5ddd2219dc868e358d02f9afcfbda6469cddd7391f150f9280440124f6a`. Birleşik referans kimliği: `eb48e23a1245d82fe150fe609e00283d8fa5a86dbd72ebeb887719a74df99968`.

Yerel raporlar: `compare-q3ryrs_k` (math debug), `compare-xtfhknri` (math release), `compare-ag5oxl30` (birleşik debug), `compare-fkrk2srf` (birleşik release) ve `verify-lstfjdye` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` chessboard ve kareli mesafe hesaplarının dört public overload yolunu kapsar. `rangeClosed`, `isValid`, diğer chunk sabitleri, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
