# P06.c — Login compression threshold codec sözleşmesi

Bu dilim, login aşamasında sunucunun compression threshold değerini bildiren `ClientboundLoginCompressionPacket` alanını kapsar. Packet gövdesi tek bir signed i32 VarInt'tir; yazma ve okuma mevcut Java VarInt kodlamasının bayt, tüketim ve hata kurallarını kullanır. Prefix/trailing byte'lar packet codec'inin dışındaki buffer verisi olarak kalır.

Bu packet'in encode/decode davranışı threshold değerini taşır. Threshold'ı kullanarak sonraki frame'leri sıkıştırma veya açma, compression handler'ını pipeline'a ekleme, protocol state ve login listener geçişi bu sözleşmenin dışındadır.

Rust `ClientboundLoginCompressionPacket::encode` ve `decode`, threshold alanını mevcut wire codec'ine bağlar. Decode sonucu değer, buffer underflow veya fazla uzun VarInt olabilir; yazma sonucu prefix sonrasına tek VarInt eklenmesidir.

Test girdileri özgün sentetik threshold değerleri ve wire byte dizileridir. Java 26.2 referans kaynakları ve capture çıktıları `.data/` altında yerel tutulur.
