# P04.a — Asset index yolu çözümleme sözleşmesi

Bu dilim, Java Edition 26.2 `IndexedAssetSource.createIndexFs` yolunun geçerli asset index girdilerini sanal kaynak ağacına bağlamasını kapsar.

Her `objects` girdisinin mantıksal yolu slash bileşenlerine ayrılır. Girdinin hash'i, fiziksel object kökünün hash'in ilk iki karakterinden oluşan alt dizini ve hash dosya adı ile birleştirilir. Aynı düğümün hem dosya hem de alt yolları varsa sanal ağaçta alt yol dizin görünümü kazanır. Index'te bulunan dosya düğümü, fiziksel object dosyası eksik olsa da varlığını korur; dosya yalnızca hedef mevcutsa okunabilir.

Rust üretim modülü, bu davranış için geçerli mantıksal yolları ve hash hedeflerini salt-okunur bir ağaçta tutar. JSON parse hataları, hash biçimi doğrulaması, pack metadata'sı ve birden fazla pack'in öncelik sırası bu sözleşmeye dahil değildir.

Test girdileri yalnızca özgün sentetik yollar, hash'ler ve byte dizileridir. Orijinal asset içerikleri `.data/` dışına alınmaz.
