# P03.b — LP Vec3 stream codec doğrulaması

`src/binary.rs`, Java `Vec3.LP_STREAM_CODEC` tarafından kullanılan packed Vec3 yolunu `read_lp_vec3` ve `write_lp_vec3` API'leriyle uygular. Oracle doğrudan Java codec'ini çağırır; beklenen baytlar Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Binary ailesinin güncel 164 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- LP Vec3 codec'i 6 yazma ve 9 okuma olmak üzere 15 senaryoda debug ve release Rust adaptörleriyle **15/15** eşleşti.
- Yazma karşılaştırması prefix dahil packed byte çıktısını ve eklenen byte sayısını; okuma karşılaştırması Vec3 raw f64 bitlerini, tüketimi, kalan veriyi ve hata kategorisini kapsar.
- Birleşik altı aile corpus'u **1449/1449** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1449 senaryoluk Java referansını yeniden doğruladı.

Binary referans kimliği: `faa64f0548d442718b7a8e5783155e27604ddd916aaf9595070c51b91232437e`. Birleşik referans kimliği: `304e1c88da43eaa3ceb24663a0ab71ddb6fb1f5487c293a9f94fb7224b94c363`.

Yerel raporlar: `compare-ijck1p8v` (binary debug), `compare-vuahjf17` (binary release), `compare-tjsmsqxw` (birleşik debug) ve `compare-sks6l8sa` (birleşik release) karşılaştırmalarında; `verify-egp3gvx_` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt LP packed header'ının 15 bit x/y/z alanlarını, 2 bit ve continuation VarInt ölçeğini, zero marker'ı, sanitize/clamp/quantization yollarını, header ve VarInt underflow tüketimini ve fazla uzun VarInt hatasını kapsar. Normal `Vec3.STREAM_CODEC`, `Vec3.CODEC`, gerçek packet queue ve packed codec'in test edilmeyen daha geniş ağ bağlamı bu dilimin dışındadır.
