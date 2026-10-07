# P01.c — Direction alt enum metin sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

Rust adapter `direction_enum_text` işlemi, `Direction.AxisDirection` ve `Direction.Plane` için Java enum adını ve `toString()` sonucunu karşılaştırır. AxisDirection için `getName()` yüzü de aynı sonuçta taşınır.

Java math capture kimliği `fd64bf62e981916ab8c6a92c9834c3ba532e49914347d37cb23c65a70090390d` olarak `.data/captures/behavior/` altında tutulur. Java capture iki ayrı JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `6d816436db044dcfab0ec694e39d8f236761dd4ccc91f061688f8c6b59ecf83d` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca iki alt enumun ad ve metin yüzlerini kapsar; Direction'ın diğer davranışları ayrı davranış dilimlerindedir.
