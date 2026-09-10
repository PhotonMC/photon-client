# P01.c — Direction ve Axis enum adı sözleşmesi

Bu dilim, ana `Direction` ve `Direction.Axis` enumlarının Java sabit adı ile serialized/metin yüzlerini ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/Direction.java:32-42,355-363,402-482,514-517` incelendi. Her iki enumda Java `name()` bildirilen sabit adını verir; `StringRepresentable` ve özel `toString()` yolları küçük harfli serialized adı taşır.

## Davranış sözleşmesi

- Direction enum adları bildirim sırasıyla `DOWN`, `UP`, `NORTH`, `SOUTH`, `WEST`, `EAST` değerleridir.
- Axis enum adları `X`, `Y`, `Z` değerleridir.
- Direction ve Axis için `getName()`, `getSerializedName()` ve `toString()` küçük harfli serialized adla aynıdır.
- Enum adı ile serialized/metin adının farklı olması korunur.

## JSONL arayüzü

İstek `family: "math"`, `operation: "direction_enum_name"`, `kind` (`direction` veya `axis`) ve küçük harfli `name` alanlarını taşır. Sonuç `enum_name`, `name`, `serialized_name` ve `text` alanlarını içerir.

Rust karşılığı `Direction::enum_name` ve `Axis::enum_name` yordamlarıdır; küçük harfli yüzler mevcut `name` yordamlarından gelir.

## Kapsam dışı

Enum codec DataResult ayrıntıları, enum bildirim sırası ve Direction/Axis davranış yardımcıları bu dilimin dışındadır.
