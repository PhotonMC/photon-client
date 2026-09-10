# P07.a — Known pack stream codec sözleşmesi

Java `KnownPack.STREAM_CODEC`, üç `ByteBufCodecs.STRING_UTF8` alanını sırasıyla `namespace`, `id` ve `version` olarak taşır. Her alan 32767 UTF-16 kod birimi sınırını kullanır; alanlar arasında ayrıca kayıt başlığı veya alan sayısı bulunmaz.

Okuma ve yazma işlemleri alan sırasını korur. Bir alanın başlığı veya gövdesi hata verirse önceki alanların tüketimi/yazımı korunur ve sonraki alanlara geçilmez. UTF-8 kablo verisinin bozuk dizileri `STRING_UTF8` decoder'ının replacement karakter davranışına bırakılır; sınır ve VarInt hataları mevcut UTF-8/wire sözleşmelerindeki kategorilerle gözlenir.

Rust `KnownPack::decode_stream` ve `encode_stream` aynı üç alanı mevcut UTF-8 yardımcılarıyla, aynı sırada ve aynı 32767 sınırıyla işler. Rust `KnownPack` alanları geçerli UTF-8 `String` olarak tutulduğu için Java encoder'ına eşleşmemiş UTF-16 surrogate içeren değerler bu ilk codec diliminin girdisi değildir; kablodan okunan replacement ve supplementary karakterler korunur.

Bu dilim configuration phase known-pack listesini, pack seçim sırasını ve gerçek ağ bağlantısını kapsamaz. Test verileri özgün sentetik alan değerleri ile bağımsız kablo byte dizileridir.
