# P04.b — Known pack doğrulaması

Java oracle, hem `KnownPack.vanilla(id)` üreticisini hem de doğrudan üç alanlı `KnownPack` kayıtlarını çalıştırır. Alan snapshot'ı, `isVanilla` sonucu ve `toString` çıktısı Rust adapter ile karşılaştırılır.

Kapsanan durumlar:

- vanilla üreticisinin `minecraft` namespace'i ve 26.2 sürümünü kullanması;
- vanilla üreticisinde farklı id değerlerinin korunması;
- `minecraft` namespace'inin id/version'dan bağımsız olarak vanilla sayılması;
- farklı büyük/küçük harfli ve özel namespace değerlerinin vanilla sayılmaması;
- boş alanların string biçiminde korunması.

Doğrulama Java 25.0.4.1 / Linux x86_64 üzerinde tamamlandı:

- 12 özgün kaynak senaryosu iki ayrı JVM çalıştırmasında kararlı bulundu;
- kaynak capture kimliği `f90806485ee2157e8be700f623c7a2ef7380dc63c6dc9b04f8beb550c30fec75`;
- Java `verify` raporu `verify-hgexcpds`;
- Rust debug/release karşılaştırmaları `compare-vqrms5jq` ve `compare-y71r0lca`, ikisi de fark üretmedi.

KnownPack oracle'ının ortak JVM'de deterministik sürüm kurulumu ile tam corpus doğrulaması da tamamlandı:

- 3408 senaryo için capture kimliği `d7e0b1170ea81802fb874a0d130287173f7fa4f55c71c6640776eed044f8eecf`;
- Java `verify` raporu `verify-xdvqtaon`;
- Rust debug/release karşılaştırmaları `compare-2vjz5_43` ve `compare-pe3guxx7`, ikisi de fark üretmedi.

Known-pack stream codec'i, configuration phase listesi ve pack seçim sırası P07 kapsamındadır.
