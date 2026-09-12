# P01.c — Direction ve Axis enum adı sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

Rust adapter `direction_enum_name` işlemi, ana Direction ve Axis enumlarının `name()`, `getName()`, `getSerializedName()` ve `toString()` sonuçlarını Java oracle ile karşılaştırır.

Java math capture kimliği `c26c352cd654343beebcfd73509e537efe7648d40c625692586886c7977757f2` olarak `.data/captures/behavior/` altında tutulur. Java capture iki ayrı JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `c59b062efb49d8fcac9683b5d87bcc7929220c35f63dd99dcaf57c97ce937dc6` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Direction ve Axis enum ad/metin yüzlerini kapsar; alt enumların özel metinleri ayrı dilimdedir.
