# P07.a — Known pack liste codec sözleşmesi

`ClientboundSelectKnownPacks.STREAM_CODEC`, KnownPack codec'ini `ByteBufCodecs.list()` ile sarar. `ServerboundSelectKnownPacks.STREAM_CODEC` aynı yapıyı `list(64)` sınırıyla kullanır. Her iki pakette önce eleman sayısı VarInt olarak, sonra kayıtlar giriş sırasıyla taşınır.

Decode sırasında count sınırı aşılırsa count byte'ları tüketilir ve kayıtlar okunmaz. Negatif count, Java listesinin negatif kapasite reddine gider; bu sözleşmede `invalid_count` olarak gözlenir. Bir kayıt ortasında hata oluşursa count ve daha önce başarıyla okunan alanlar tüketilmiş kalır. Encode sırasında sınır aşımı count yazılmadan hata verir; kayıt codec'i daha sonraki bir alanda hata verirse daha önce yazılan count ve alanlar korunur.

Rust `KnownPack::decode_list` ve `encode_list` max-size parametresiyle bu iki packet codec yolunu aynı count, sıra, tüketim ve kısmi yazma kurallarıyla taşır. Clientbound için Java'nın `Integer.MAX_VALUE` list sınırı, serverbound için 64 kullanılır.

Bu dilim packet state geçişini, configuration listener'ını, registry seçimini ve gerçek bağlantıyı kapsamaz. Test verileri özgün sentetik KnownPack kayıtları ve bağımsız framing byte dizileridir.
