# P07.a — Known pack liste codec doğrulaması

Java oracle, `ClientboundSelectKnownPacks.STREAM_CODEC` ve `ServerboundSelectKnownPacks.STREAM_CODEC`'i gerçek Netty `ByteBuf` üzerinde doğrudan çalıştırır. Rust adapter aynı packet türü ve max-size ile count'u, KnownPack listesini, tüketilen/yazılan byte'ları ve kalan buffer'ı karşılaştırır.

Kapsanan durumlar:

- clientbound ve serverbound boş/temel listeler;
- serverbound 64 eleman sınırı ve 65 eleman reddi;
- count VarInt kesilmesi, negatif count ve trailing byte;
- kayıt içinde bozuk UTF-8 ve kesilmiş sonraki kayıt;
- liste elemanı codec'inde kısmi encode.

Java 25.0.4.1 / Linux x86_64 üzerinde 15 özgün resource senaryosu iki JVM çalıştırmasında kararlı bulundu. Capture kimliği `1cffc068905086bd15cdd39656bf942c1f217b3d3e9157540316276ece36cb8d`; verify raporu `verify-zr_epl4d`, debug karşılaştırması `compare-z2qjd84`, release karşılaştırması `compare-3jai7ayw` ve üçü de fark üretmedi.

Güncel bütün corpus 3440 senaryoya çıktı: capture `a74a836c2a3ab44f91c4c8719c498a8cd72d7f21bf671151f46be35d721eed6d`, verify `verify-qcz_u2iv`, debug `compare-z39v7hdv`, release `compare-0y7a9jy9`; dört koşu da fark üretmedi.

Configuration listener'ının seçme/ack sırası ve gerçek server paket akışı P07'nin sonraki dilimleridir.
