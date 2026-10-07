# P01.c — BlockPos mutable helpers sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, MutableBlockPos'ın Java'daki yerinde güncelleme ve immutable sonuç davranışını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:231-237` ve `618-740` incelendi. BlockPos `immutable()` kendisini, `mutable()` yeni MutableBlockPos'ı üretir. MutableBlockPos integer/double constructor'ları ve `set` overload'ları koordinatları yazar; double yolları `Mth.floor`, packed yol `BlockPos.getX/Y/Z` kullanır. `setWithOffset`, `move`, `clamp` ve setX/Y/Z durumu değiştirir. Mutable override'ları `offset`, `multiply`, `relative` ve `rotate` için mutasyonu koruyup yeni immutable BlockPos sonucu döndürür.

## Davranış sözleşmesi

- Mutable state her mutation'dan sonra yeni koordinatları taşır ve method zincirleri aynı mutable nesneyi hedefler.
- Double constructor/set floor dönüşümünü Java ile aynı yapar; packed set signed alanları BlockPos packed düzeninden çıkarır.
- Direction ve integer offset/move yolları Java i32 toplama ve çarpma sarmasını korur.
- `clamp` yalnızca seçilen ekseni `Mth.clamp(int)` ile sınırlar.
- `offset`/`multiply`/`relative`/`rotate` override'ları mutable state'i değiştirmeden immutable dönüş değeri verir.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_mutable_helpers"`, bir initial BlockPos, f64 bit alanlı double constructor değerleri ve sıralı mutation adımları taşır. Sonuç constructor gözlemleri ile her adımın dönen değerini ve mutable state snapshot'ını içerir.

Rust karşılığı `BlockPos::immutable`, `BlockPos::mutable`, `MutableBlockPos` constructor/set/move/clamp ve immutable-result yordamlarıdır.

## Kapsam dışı

`AxisCycle` set overload'u, gerçek subclass polymorphism'i, codec'ler ve dünya traversal'ı bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `blockpos_mutable_helpers` işlemi, MutableBlockPos constructor, mutation ve immutable-result yollarını Java oracle ile karşılaştırır.

Java math capture kimliği `f0efc0bd24b33e5bc88940e628a54342c8f3e4d440f1341bc44a419b54d0d2e2` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu; Rust debug adapter karşılaştırması aynı kimliğe karşı başarılı oldu.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt floor/packed set yollarını, offset/move/clamp state değişimini, i32 sarmasını ve mutable override'larının immutable dönüşlerini kapsar.

Tüm davranış ailelerini kapsayan son ortak capture kimliği `55adde7cba770b1ef6a8ccd4f1c273c13a5a4294399ac117cbdbd31596c530f9`'dir. Debug ve release adapter'ları bu capture'a karşı başarılı oldu.
