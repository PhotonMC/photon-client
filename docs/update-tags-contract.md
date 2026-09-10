# P07.b — Update tags packet sözleşmesi

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ClientboundUpdateTagsPacket` önce registry map count'unu VarInt olarak, sonra her registry key'i Identifier UTF-8 alanı ve `TagNetworkSerialization.NetworkPayload` olarak taşır. Network payload kendi tag map count'unu, tag Identifier'ını ve registry entry id'lerinin VarInt listesi count'unu izler.

Map yazımı Java `HashMap.forEach` sırasındadır. Decode'da map kapasitesi count'tan türetilir; aynı Identifier tekrar gelirse son değer önceki değeri değiştirir ve map konumu korunur. Entry id listesinde negatif count hata değildir: Java döngüsü hiç çalışmaz ve boş liste döner. Pozitif listede her id signed VarInt'tir. Identifier parse/UTF-8 hataları decoder hatası, kesik VarInt underflow veya too-big sonucu üretir.

Rust tarafı registry/tag map'lerini Identifier hash davranışı ve Java map kapasite/iteration kurallarıyla, id listelerini sıralı signed VarInt'lerle taşır. Bu dilimde wire sırasını deterministik tutmak için tek registry map senaryoları karşılaştırıldı; birden fazla `ResourceKey` içeren üst map'in JVM identity hash sırası ve tag'lerin registry'ye uygulanması sonraki P07.b/c entegrasyon kapsamıdır.
