# P07.a — Known pack liste codec sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

`ClientboundSelectKnownPacks.STREAM_CODEC`, KnownPack codec'ini `ByteBufCodecs.list()` ile sarar. `ServerboundSelectKnownPacks.STREAM_CODEC` aynı yapıyı `list(64)` sınırıyla kullanır. Her iki pakette önce eleman sayısı VarInt olarak, sonra kayıtlar giriş sırasıyla taşınır.

Decode sırasında count sınırı aşılırsa count byte'ları tüketilir ve kayıtlar okunmaz. Negatif count, Java listesinin negatif kapasite reddine gider; bu sözleşmede `invalid_count` olarak gözlenir. Bir kayıt ortasında hata oluşursa count ve daha önce başarıyla okunan alanlar tüketilmiş kalır. Encode sırasında sınır aşımı count yazılmadan hata verir; kayıt codec'i daha sonraki bir alanda hata verirse daha önce yazılan count ve alanlar korunur.

Rust `KnownPack::decode_list` ve `encode_list` max-size parametresiyle bu iki packet codec yolunu aynı count, sıra, tüketim ve kısmi yazma kurallarıyla taşır. Clientbound için Java'nın `Integer.MAX_VALUE` list sınırı, serverbound için 64 kullanılır.

Bu dilim packet state geçişini, configuration listener'ını, registry seçimini ve gerçek bağlantıyı kapsamaz. Test verileri özgün sentetik KnownPack kayıtları ve bağımsız framing byte dizileridir.


## Client-parity doğrulaması

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
