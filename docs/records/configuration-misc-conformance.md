# P07.b — Configuration misc packet sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ClientboundCodeOfConductPacket` tek bir `STRING_UTF8` alanı taşır. Bu alanın varsayılan sınırı 32767 Java UTF-16 birimidir; wire'da önce UTF-8 byte uzunluğu VarInt olarak, ardından UTF-8 gövdesi yazılır.

Encode işlemi Java'da önce metnin UTF-16 uzunluğunu sınırlar, sonra UTF-8 byte uzunluğunu 32767 birim için izin verilen üst sınıra göre kontrol eder. Sınır aşımında packet gövdesi yazılmaz. Decode işlemi encoded byte uzunluğunu üst sınırla karşılaştırır; gövde eksikse hata verir, gövdeyi UTF-8 replacement davranışıyla metne çevirir ve replacement sonrası UTF-16 uzunluğunu tekrar sınırlar. Başarılı okumada trailing byte'lar kalır; hata sırasında tüketim Java'nın kontrol sırasına göre gözlenir.

`ClientboundResetChatPacket` `StreamCodec.unit` singleton'ıdır. Encode boş çıktı verir, decode readable byte'ları tüketmeden singleton döndürür.

Rust modeli code-of-conduct metnini Java UTF-16 birimleri olarak tutar; mevcut UTF-8 codec'i ile 32767 sınırını uygular. Reset-chat için payload'sız singleton codec kullanılır. Bu sözleşme packet payload ve buffer tüketimiyle sınırlıdır; listener davranışı, code-of-conduct ekranı ve configuration→play geçişi P07.c/P22 kapsamındadır.


## Client-parity doğrulaması

Java oracle, `ClientboundCodeOfConductPacket.STREAM_CODEC` ve `ClientboundResetChatPacket.STREAM_CODEC` yollarını gerçek Netty buffer üzerinde çağırır. Rust adapter aynı isteklerde UTF-16 snapshot'ını, yazılan byte sayısını, ham buffer'ı, tüketimi, trailing byte'ları ve hata kategorisini döndürür.

Bu dilimde 18 özgün wire senaryosu kapsandı: code-of-conduct için beş encode ve sekiz decode, reset-chat için iki encode ve üç decode. Boş/Unicode/URL metni, 32767 ve 32768 UTF-16 sınırı, malformed UTF-8 replacement, encoded byte üst sınırı, kesik gövde, replacement sonrası decoded uzunluk, prefix/trailing buffer ve unit codec tüketimsizliği doğrulandı.

Java capture kimliği `60202eb7322330d28ed33e195d458d0c2bd6329c3de9a2042c72d3352b1b312d`; 3748 senaryolu tam capture'ta code-of-conduct/reset-chat vakaları wire ailesine eklendi. `verify-53np8_po`, debug `compare-23e3fgi6` ve release `compare-w14fxvj7` raporlarının tamamında fark yoktur.

Bu sonuç iki payload codec'inin Java eşitliğini doğrular. Configuration listener'ın code-of-conduct metnini UI'ye taşıması, reset-chat olayının chat durumuna uygulanması, registry birikimi ve play aşamasına geçiş sonraki P07.c/P22 entegrasyon kapsamındadır.
