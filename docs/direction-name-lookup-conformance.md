# P01.c — Direction ve Axis name lookup sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java `Direction.byName(String)` ve `Direction.Axis.byName(String)` lookup yollarını ölçer.

## Kaynak ve sınır

`core/Direction.java:40,279-280,483,490-491` ile `util/StringRepresentable.java:23-58,80-82` incelendi. Java oracle, enum codec lookup sonucunu doğrudan çağırır; bulunan enumun ad, serialized name ve `toString` yüzlerini de aynı çağrıdan okur.

## Davranış sözleşmesi

- Lookup, serialized name ile case-sensitive karşılaştırma yapar.
- Direction için altı küçük harfli yön adı, Axis için `x`, `y`, `z` geçerlidir.
- Büyük/küçük harf farkı, boşluk, boş string ve bilinmeyen ad `found: false` sonucu verir.
- Bulunan enumda `name`, `serialized_name` ve `text` aynı küçük harfli değeri taşır.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_name_lookup"`, `kind` (`direction` veya `axis`) ve string `name` alanlarını taşır. Bulunan sonuç üç ad alanını; bulunamayan sonuç yalnızca `found: false` alanını içerir.

## Kapsam dışı

Codec JSON/DataResult dönüşümleri, varsayılan değer overload'ları ve Direction'ın diğer davranışları bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `direction_name_lookup` işlemi, `Direction::from_name` ve `Axis::from_name` yollarını Java `byName` lookup'larıyla karşılaştırır. Geçerli enumlarda ad, serialized name ve metin yüzleri; geçersiz adlarda null/`found: false` sonucu taşınır.

Java math capture kimliği `ec517b984123384c87f5509cd22e1828ce38ff75f205dd0f2ded4ab6f21d6da3` olarak `.data/` altında tutulur. Java capture iki ayrı JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `490b72d70518ecaea73be8f84e7112792a18c7ca3ea95c5804a3dbf762ba6afb` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Bu kanıt yalnızca Direction ve Axis serialized-name lookup yüzlerini kapsar; codec DataResult dönüşümleri ve diğer enum davranışları ayrı kapsamdır.
