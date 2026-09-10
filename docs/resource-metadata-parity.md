# P04.b — Pack metadata uyumluluğu doğrulaması

Java oracle, `PackCompatibility.forVersion` metodunu 26.2 resource-pack formatı ile çağırır. Her sentetik declared range için enum sonucu ve `isCompatible` değeri karşılaştırılır.

Kapsanan durumlar:

- bilinmeyen format sentinel'ı;
- oyunun formatının altında ve üstünde kalan aralıklar;
- `88.0` sınırı ve minor sürüm farkı;
- oyunun formatını kesen alt/üst aralıklar.

Doğrulama Java 25.0.4.1 / Linux x86_64 üzerinde tamamlandı:

- dokuz özgün senaryo iki ayrı JVM çalıştırmasında kararlı bulundu;
- kaynak capture kimliği `75e93ed2b00aeaa36847d083ce4493ec4bc63f34b44b7c439f9f36732d39bfd2`;
- Java `verify` raporu `verify-1zf1_q9w`;
- Rust debug karşılaştırması `compare-bf15jwnr`, release karşılaştırması `compare-hsc0t3c5` ve her ikisi de fark üretmedi.

Tam corpus capture kimliği `89ac436d24ebf5e2c3dc72dcb68d2279302841a0f5f14ea36324c6472a0ca503`; debug/release karşılaştırmaları `compare-7bh7p_5r` ve `compare-5nn2jb9y`, Java doğrulaması `verify-91a0olg4` olarak kaydedildi.

Pack metadata JSON parse, feature flag, overlay ve repository keşfi sonraki P04 dilimleridir.
