# P07.a — Known pack stream codec doğrulaması

Java oracle, `KnownPack.STREAM_CODEC`'i gerçek Netty `ByteBuf` üzerinde doğrudan çalıştırır. Rust adapter aynı girdide üç alanı sırayla encode/decode eder; alan değerleri, yazılan/tüketilen byte sayısı, kalan buffer ve hata kategorisi karşılaştırılır.

Kapsanan durumlar:

- ASCII, Unicode, supplementary karakter, birleşen karakter ve boş alanların üçlü sıralı encode/decode yolları;
- 32767 UTF-16 birimi sınırını aşan namespace, id ve version alanları;
- ilk/ikinci/üçüncü alanın kesilmiş başlık veya gövdesi;
- fazla uzun VarInt, kodlanmış uzunluk sınırı, bozuk UTF-8 gövdesi ve trailing byte;
- ikinci veya üçüncü alana gelene kadar oluşan kısmi yazma ve kısmi tüketim.

Java 25.0.4.1 / Linux x86_64 üzerinde 29 özgün resource senaryosu iki JVM çalıştırmasında kararlı bulundu. Yeni referans bağımlılığı (`netty-buffer`) açıkça `tests/behavior/reference.json` içine eklendi; önceki capture'lar değiştirilmedi.

Odaklı kayıt kimliği `c700153bea66d3ff1caacebb6cfc7951705a79164f5ef6893e2f55a0440b6ca8`; verify raporu `verify-2vlc0m0d`, debug karşılaştırması `compare-p_tve_3k`, release karşılaştırması `compare-qtabx7k6` ve hepsi fark üretmedi.

Bu değişiklikten sonra bütün aileler 3425 senaryo ile açıkça yeniden yakalandı: capture `3e468a0843e9587c28b0a3769d5414233e1a5dd44154253aeb02066b00424e45`, verify `verify-201suji9`, debug `compare-bj_c47e1`, release `compare-jtr4asqk`; dört koşu da fark üretmedi.

Codec'in configuration phase listesine bağlanması ve gerçek server paketleri P07'nin sonraki dilimleridir.
