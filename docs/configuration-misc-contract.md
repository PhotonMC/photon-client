# P07.b — Configuration misc packet sözleşmesi

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ClientboundCodeOfConductPacket` tek bir `STRING_UTF8` alanı taşır. Bu alanın varsayılan sınırı 32767 Java UTF-16 birimidir; wire'da önce UTF-8 byte uzunluğu VarInt olarak, ardından UTF-8 gövdesi yazılır.

Encode işlemi Java'da önce metnin UTF-16 uzunluğunu sınırlar, sonra UTF-8 byte uzunluğunu 32767 birim için izin verilen üst sınıra göre kontrol eder. Sınır aşımında packet gövdesi yazılmaz. Decode işlemi encoded byte uzunluğunu üst sınırla karşılaştırır; gövde eksikse hata verir, gövdeyi UTF-8 replacement davranışıyla metne çevirir ve replacement sonrası UTF-16 uzunluğunu tekrar sınırlar. Başarılı okumada trailing byte'lar kalır; hata sırasında tüketim Java'nın kontrol sırasına göre gözlenir.

`ClientboundResetChatPacket` `StreamCodec.unit` singleton'ıdır. Encode boş çıktı verir, decode readable byte'ları tüketmeden singleton döndürür.

Rust modeli code-of-conduct metnini Java UTF-16 birimleri olarak tutar; mevcut UTF-8 codec'i ile 32767 sınırını uygular. Reset-chat için payload'sız singleton codec kullanılır. Bu sözleşme packet payload ve buffer tüketimiyle sınırlıdır; listener davranışı, code-of-conduct ekranı ve configuration→play geçişi P07.c/P22 kapsamındadır.
