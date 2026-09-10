# P07.b — Configuration misc packet codec doğrulaması

Java oracle, `ClientboundCodeOfConductPacket.STREAM_CODEC` ve `ClientboundResetChatPacket.STREAM_CODEC` yollarını gerçek Netty buffer üzerinde çağırır. Rust adapter aynı isteklerde UTF-16 snapshot'ını, yazılan byte sayısını, ham buffer'ı, tüketimi, trailing byte'ları ve hata kategorisini döndürür.

Bu dilimde 18 özgün wire senaryosu kapsandı: code-of-conduct için beş encode ve sekiz decode, reset-chat için iki encode ve üç decode. Boş/Unicode/URL metni, 32767 ve 32768 UTF-16 sınırı, malformed UTF-8 replacement, encoded byte üst sınırı, kesik gövde, replacement sonrası decoded uzunluk, prefix/trailing buffer ve unit codec tüketimsizliği doğrulandı.

Java capture kimliği `60202eb7322330d28ed33e195d458d0c2bd6329c3de9a2042c72d3352b1b312d`; 3748 senaryolu tam capture'ta code-of-conduct/reset-chat vakaları wire ailesine eklendi. `verify-53np8_po`, debug `compare-23e3fgi6` ve release `compare-w14fxvj7` raporlarının tamamında fark yoktur.

Bu sonuç iki payload codec'inin Java eşitliğini doğrular. Configuration listener'ın code-of-conduct metnini UI'ye taşıması, reset-chat olayının chat durumuna uygulanması, registry birikimi ve play aşamasına geçiş sonraki P07.c/P22 entegrasyon kapsamındadır.
