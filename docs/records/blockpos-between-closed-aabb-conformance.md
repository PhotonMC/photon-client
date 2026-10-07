# P01.c — BlockPos betweenClosed AABB sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, BlockPos'ın AABB alan `betweenClosed` overload'unu ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:356-360` ve `.data/minecraft/source/net/minecraft/world/phys/AABB.java:22-29` incelendi. AABB constructor'ı uçları eksen bazında normalize eder; BlockPos overload'u normalize edilmiş min/max double değerlerini `Mth.floor` ile integer BlockPos uçlarına çevirip mevcut inclusive range yoluna verir.

## Davranış sözleşmesi

- AABB'nin her eksendeki küçük ve büyük ucu önce Java `Math.min`/`Math.max` semantiğiyle seçilir.
- Min ve max double uçları `Mth.floor` ile bağımsız i32 koordinatlara dönüştürülür.
- Floor sonrası uçlar inclusive olarak ve x hızlı, sonra y, sonra z sırasında taranır.
- Signed zero sonucu koordinat davranışını değiştirmez; negatif kesirler bir önceki integer koordinata iner.
- Bu dilimde yalnızca sonlu, küçük aralıklar kullanılır.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_between_closed_aabb"` ve altı f64 bit alanlı `aabb` nesnesini taşır. Sonuç `positions` alanında BlockPos snapshot listesi içerir.

Java oracle `BlockPos.betweenClosed(AABB)` overload'unu doğrudan tüketir. Rust karşılığı `BlockPos::between_closed_aabb` yordamıdır.

## Kapsam dışı

BoundingBox/stream overload'ları, sonsuz veya dev aralıklar, `withinManhattan` ve diğer BlockPos iterator'ları bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `blockpos_between_closed_aabb` işlemi, AABB uçlarının floor dönüşümünü ve inclusive BlockPos traversal'ını Java oracle ile karşılaştırır.

Java math capture kimliği `e7910890bfd37b6cf2c46e9fda186fb61be379d695c909b1235d1ba79bf1c025` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt sonlu küçük AABB'leri kapsar; BoundingBox/stream ve büyük veya sonsuz aralıklar ayrı kapsamdır.

Birleşik capture kimliği `b88bab35dcec6d29f33c93274632b3bb9a433620ae69e00d2498abc5a02ee835` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
