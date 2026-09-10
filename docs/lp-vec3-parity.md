# P03.b — LP Vec3 stream codec doğrulaması

`src/binary.rs`, Java `Vec3.LP_STREAM_CODEC` tarafından kullanılan packed Vec3 yolunu `read_lp_vec3` ve `write_lp_vec3` API'leriyle uygular. Oracle doğrudan Java codec'ini çağırır; beklenen baytlar Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Binary ailesinin güncel 164 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- LP Vec3 codec'i 6 yazma ve 9 okuma olmak üzere 15 senaryoda debug ve release Rust adaptörleriyle **15/15** eşleşti.
- Yazma karşılaştırması prefix dahil packed byte çıktısını ve eklenen byte sayısını; okuma karşılaştırması Vec3 raw f64 bitlerini, tüketimi, kalan veriyi ve hata kategorisini kapsar.
- Birleşik altı aile corpus'u **1952/1952** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1952 senaryoluk Java referansını yeniden doğruladı.

Binary referans kimliği: `faa64f0548d442718b7a8e5783155e27604ddd916aaf9595070c51b91232437e`. Birleşik referans kimliği: `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11`.

Yerel raporlar: `compare-ijck1p8v` (binary debug), `compare-vuahjf17` (binary release), `compare-x0hkkvkv` (birleşik debug) ve `compare-x5wgwswa` (birleşik release) karşılaştırmalarında; `verify-rc21vyy9` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt LP packed header'ının 15 bit x/y/z alanlarını, 2 bit ve continuation VarInt ölçeğini, zero marker'ı, sanitize/clamp/quantization yollarını, header ve VarInt underflow tüketimini ve fazla uzun VarInt hatasını kapsar. Normal `Vec3.STREAM_CODEC`, `Vec3.CODEC`, gerçek packet queue ve packed codec'in test edilmeyen daha geniş ağ bağlamı bu dilimin dışındadır.
