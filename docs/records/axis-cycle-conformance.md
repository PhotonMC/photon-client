# P01.c — AxisCycle sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, üç eksenli döndürme yardımcılarının Java AxisCycle sırasını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/AxisCycle.java:3-81` ve `BlockPos.java:675-677` incelendi. NONE bileşenleri olduğu gibi seçer; FORWARD `(z, x, y)` ve BACKWARD `(y, z, x)` seçimini kullanır. Axis dönüşümü ordinali üçlü döngüde ilerletir/geriletir, inverse karşı cycle'ı verir ve `between` hedef ile kaynak ordinal farkını üçe göre normalize eder. MutableBlockPos setter, her hedef eksen için cycle sonucunu yazar.

## Davranış sözleşmesi

- Int ve double cycle yolları aynı eksen permütasyonunu korur; double bit desenleri aktarılır.
- NONE self-inverse'tür; FORWARD/BACKWARD birbirinin inverse'ıdır.
- Axis cycle X→Y→Z→X yönünde, BACKWARD ters yönde ilerler.
- `between(from, to)`, source axis'ten target axis'e ulaşan NONE/FORWARD/BACKWARD cycle'ını döndürür.
- Mutable setter, cycle edilmiş X/Y/Z değerlerini tek mutation ile state'e koyar.

## JSONL arayüzü

İstek `family: "math"`, `operation: "axis_cycle_helpers"`, signed i32 ve f64 bit alanlı üç bileşen taşır. Sonuç cycle başına int/double/axis/inverse gözlemlerini, tüm axis çiftleri arasındaki cycle seçimlerini ve MutableBlockPos setter sonuçlarını içerir.

Rust karşılığı `AxisCycle` ve `MutableBlockPos::set_axis_cycle` yordamlarıdır.

## Kapsam dışı

Shape içindeki özel axis-cycle traversal'ları, AxisCycle codec'i ve MutableBlockPos'ın diğer setter/move yolları bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `axis_cycle_helpers` işlemi, AxisCycle permütasyonlarını ve MutableBlockPos cycle setter'ını Java oracle ile karşılaştırır.

Java math capture kimliği `4e31a6e287300acbb9c7448ccd1d665e09941318f9d2eb5976824e659000cbd3` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu; Rust debug adapter karşılaştırması aynı kimliğe karşı başarılı oldu.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt üç int/double permütasyonunu, axis yönlerini, inverse/between seçimlerini ve MutableBlockPos setter forwarding'ini kapsar.

Tüm davranış ailelerini kapsayan son ortak capture kimliği `e524bd8c6548c77e9d64537affc2c803c904090c8a1e90891f5191acc901623e`'dir. Debug ve release adapter'ları bu capture'a karşı başarılı oldu.
