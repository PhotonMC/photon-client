# P04.a — Asset index yolu çözümleme doğrulaması

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
