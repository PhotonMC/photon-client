# P01.c — BlockPos findClosestMatch sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, BlockPos'ın Manhattan taramasında predicate tarafından kabul edilen ilk konumu seçen yardımcı metodunu ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:340-350` incelendi. `findClosestMatch`, yatay yarıçapı X ve Z eksenlerine, dikey yarıçapı Y eksenine aktararak `withinManhattan` iterable'ını kaynak sırasıyla tüketir; ilk predicate eşleşmesinde durur.

## Davranış sözleşmesi

- Arama sırası `withinManhattan(start, horizontalRadius, verticalRadius, horizontalRadius)` ile aynıdır.
- İlk eşleşen konum döndürülür; eşleşme yoksa Optional boş kalır.
- Sıfır yarıçap yalnızca başlangıç konumunu aday yapar.
- Origin ve traversal offset'leri Java i32 sarmasıyla değerlendirilir.
- Bu adapter diliminde predicate, adayın verilen target BlockPos'a eşitliği olarak sabitlenmiştir.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_find_closest_match"`, `start` ve `target` BlockPos nesneleri ile signed decimal `horizontal_radius`/`vertical_radius` alanlarını taşır. Eşleşmede sonuç `value` alanında BlockPos snapshot içerir; eşleşme yoksa Java'nın null serialization davranışı nedeniyle sonuç nesnesi boştur.

Java oracle eşitlik predicate'iyle `BlockPos.findClosestMatch` çağrısını doğrudan tüketir. Rust karşılığı generic `BlockPos::find_closest_match` yordamıdır.

## Kapsam dışı

Diğer predicate türleri, `withinManhattan` iterator ayrıntıları, breadth-first traversal ve predicate hata/yan etki sırası bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `blockpos_find_closest_match` işlemi, eşitlik predicate'iyle ilk Manhattan eşleşmesini Java oracle ile karşılaştırır.

Java math capture kimliği `eb48af1e939a1134e04e3c62c429ae66161f024855377cd609092b944ac8707d` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug/release math adapter karşılaştırmaları fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt ilk eşleşme, boş Optional, sıfır yarıçap ve i32 sınır yollarını kapsar; diğer predicate callback'leri ayrı kapsamdır.

Birleşik capture kimliği `bf0c111eb2762fc08b3f09a2eefcb6036a98ca411a5ef33bfa0d1a106ad8b020` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
