# P04.a — Asset index yolu çözümleme sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `IndexedAssetSource.createIndexFs` yolunun geçerli asset index girdilerini sanal kaynak ağacına bağlamasını kapsar.

Her `objects` girdisinin mantıksal yolu slash bileşenlerine ayrılır. Girdinin hash'i, fiziksel object kökünün hash'in ilk iki karakterinden oluşan alt dizini ve hash dosya adı ile birleştirilir. Aynı düğümün hem dosya hem de alt yolları varsa sanal ağaçta alt yol dizin görünümü kazanır. Index'te bulunan dosya düğümü, fiziksel object dosyası eksik olsa da varlığını korur; dosya yalnızca hedef mevcutsa okunabilir.

Rust üretim modülü, bu davranış için geçerli mantıksal yolları ve hash hedeflerini salt-okunur bir ağaçta tutar. JSON parse hataları, hash biçimi doğrulaması, pack metadata'sı ve birden fazla pack'in öncelik sırası bu sözleşmeye dahil değildir.

Test girdileri yalnızca özgün sentetik yollar, hash'ler ve byte dizileridir. Orijinal asset içerikleri `.data/` dışına alınmaz.


## Client-parity doğrulaması

Java oracle, sentetik bir `indexes/<name>.json` ile `objects/<ilk iki hash>/<hash>` dosya ağacı oluşturup `IndexedAssetSource.createIndexFs` çağrısını yürütür. Her sorguda sanal yolun varlığı, dizin olup olmadığı, kararlı object yolu, hedefin okunabilirliği ve okunabilen byte'lar karşılaştırılır.

Kapsanan durumlar:

- farklı namespace ve çok seviyeli yolların object hedeflerine bağlanması;
- dosya ile alt dizin çakışmasında dizin görünümünün seçilmesi;
- index bağlantısının fiziksel object dosyası yokken görünür kalması;
- sanal dizinlerin ve bilinmeyen yolların ayrıştırılması.

Doğrulama Java 25.0.4.1 / Linux x86_64 üzerinde tamamlandı:

- dört özgün senaryo iki ayrı JVM çalıştırmasında kararlı bulundu;
- kaynak capture kimliği `cb843fff4e4f68f49e6542abab2999b99bc9208833217654b69adcc6d71d837a`;
- kaynak `verify` raporu `verify-qytfxxwa`;
- Rust debug karşılaştırması `compare-hsy0tpwz`, release karşılaştırması `compare-jrj3hyy6` ve her ikisi de fark üretmedi;
- aynı değişiklikleri içeren ortak capture kimliği `d359fb21989648e3f021e399346afdfc47a6c95bd7c4b6a84959bdbe090dedb8`; ortak debug/release karşılaştırmaları `compare-264cbr2_` ve `compare-eapgr9d6`, ortak Java doğrulaması `verify-7p755w41` olarak kaydedildi.

Bozuk index JSON'u, kısa/bozuk hash, pack metadata ve pack reload davranışı P04'ün sonraki dilimleridir.
