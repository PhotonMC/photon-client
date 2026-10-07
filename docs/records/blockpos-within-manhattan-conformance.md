# P01.c — BlockPos withinManhattan sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, BlockPos'ın Manhattan reach iterator'ını ve sequential stream forwarding'ini ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:287-354` incelendi. Iterator toplam reach'i maksimum derinlik olarak kullanır; her derinlikte x ve y eksenlerini tarar, kalan z değerini hesaplar ve reach içinde kalan sıfır dışı z değerinin mirror'ını hemen sonraki sonuç olarak üretir. `withinManhattanStream` bu iterable'ı sequential stream'e verir.

## Davranış sözleşmesi

- Her sonuç `abs(x-originX) + abs(y-originY) + abs(z-originZ)` reach sınırları içinde Java kaynak traversal sırasıyla üretilir.
- x, y ve derinlik döngüsü sırası ile sıfır dışı z ve mirror z çifti korunur.
- Eksen reach'leri ve origin offset'leri Java i32 aritmetiğiyle değerlendirilir.
- Mutable cursor sonuçları capture sırasında bağımsız BlockPos snapshot'larına dönüştürülür.
- Stream çıktısı iterable çıktısıyla aynı sırayı taşır.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_within_manhattan"`, `origin` BlockPos nesnesi ve signed decimal `reach_x`/`reach_y`/`reach_z` alanlarını taşır. Sonuç `positions` ve `stream_positions` listelerini içerir.

Java oracle hem `BlockPos.withinManhattan` hem `withinManhattanStream` yollarını doğrudan tüketir. Rust karşılığı `BlockPos::within_manhattan` yordamıdır.

## Kapsam dışı

`findClosestMatch`, predicate callback'leri, breadth-first traversal, `spiralAround` ve negatif/taşmalı dev reach'lerin iterator yaşam süresi bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `blockpos_within_manhattan` işlemi, iterable ve sequential stream sonuçlarını Java oracle ile karşılaştırır.

Java math capture kimliği `f2171426d9af3c802c99c5b0378ff74fb602d45bbccf5d72aec8b5d5a6bf0a69` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt sonlu, non-negative reach değerleriyle traversal sırasını, z mirror çiftlerini ve origin offset sarmasını kapsar; callback/predicate ve taşmalı dev reach'ler ayrı kapsamdır.

Birleşik capture kimliği `ddbd12001da620eec4b0db4a4ffd76e8f2dce41e0148d13db1129db91231db09` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
