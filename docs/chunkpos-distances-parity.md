# P01.c — ChunkPos mesafe yardımcıları doğrulaması

`src/binary.rs` ve `src/math.rs`, Java `ChunkPos.getChessboardDistance` ile `distanceSquared` overload'larını signed i32 çıkarma, mutlak değer, kare ve toplam sırasını koruyarak karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk koordinatlarını üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos mesafeleri için 6 özgün senaryo hazırlandı.
- Normal farklar, negatif koordinatlar, çıkarma overflow'u, `Integer.MIN_VALUE` mutlak değer kenarı ve i32 kare/toplam taşmaları Java sonucu ile eşleşti.
- Rust debug ve release adaptörleri güncel **1354/1354** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1856/1856** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1856 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `7bacfa4674e388b5f38f1b0a041e5531afe27e845d4374895c79e4fb56697a24`. Birleşik referans kimliği: `3afb23055305f62e13f7bdef0f8d340c8294187f00de3b26a2c52874e45642a7`.

Yerel raporlar: `compare-z9ypl4_s` (math debug), `compare-0xhgd0yz` (math release), `compare-ote21q3t` (birleşik debug), `compare-r5nijsku` (birleşik release) ve `verify-2b2qykhl` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` chessboard ve kareli mesafe hesaplarının dört public overload yolunu kapsar. `rangeClosed`, `isValid`, diğer chunk sabitleri, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
