# P01.c — Direction ve Axis name lookup sözleşmesi

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
