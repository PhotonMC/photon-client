# P01.c — Direction alt enum metin sözleşmesi

Bu dilim, `Direction.AxisDirection` ve `Direction.Plane` enumlarının Java ad ve metin yüzlerini ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/Direction.java:546-574,576-615` incelendi. `AxisDirection` kendi `toString()` metodunu tanımlar; `Plane` özel bir `toString()` tanımlamadığı için Java `Enum.toString()` yolunu kullanır.

## Davranış sözleşmesi

- `AxisDirection.name()` bildirilen sabit adını (`POSITIVE` veya `NEGATIVE`) döndürür.
- `AxisDirection.getName()` ve `AxisDirection.toString()` aynı insan okunur metni (`Towards positive` veya `Towards negative`) döndürür.
- `Plane.name()` bildirilen sabit adını (`HORIZONTAL` veya `VERTICAL`) döndürür.
- `Plane.toString()` varsayılan Java enum metnidir ve `name()` ile aynı değeri taşır.

## JSONL arayüzü

İstek `family: "math"`, `operation: "direction_enum_text"`, `kind` (`axis_direction` veya `plane`) ve küçük harfli `name` alanlarını taşır. Sonuç her zaman `enum_name` ve `text` alanlarını içerir; AxisDirection sonucu ayrıca `name` alanını içerir.

Rust karşılığı `AxisDirection::enum_name`, `AxisDirection::name`, `AxisDirection::to_string_java`, `Plane::enum_name` ve `Plane::to_string_java` yordamlarıdır.

## Kapsam dışı

Enum lookup codec'leri, Plane iterator/random yardımcıları ve entity veya matris bağımlı Direction metotları bu dilime dahil değildir.
