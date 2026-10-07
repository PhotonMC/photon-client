# P01.c — BlockPos squareOutSouthEast sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, BlockPos'ın deprecated `squareOutSouthEast` yardımcı metodunun sabit dört konumlu sırasını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:260-263` incelendi. Yardımcı, `Stream.of(from, from.south(), from.east(), from.south().east())` çağrısıyla dört BlockPos sonucu üretir.

## Davranış sözleşmesi

- Çıktı sırası kaynak, south, east ve south-east olarak korunur.
- South yalnızca Z'yi bir artırır; east yalnızca X'i bir artırır; Y değişmez.
- Koordinat adımları Java i32 aritmetiğiyle sarmalanır.
- Stream sonuçları capture sırasında bağımsız BlockPos snapshot'ları olarak gözlemlenir.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_square_out_south_east"` ve signed decimal `x`/`y`/`z` alanlarını taşır. Sonuç `positions` alanında dört snapshot içerir.

Java oracle `BlockPos.squareOutSouthEast` stream'ini doğrudan tüketir. Rust karşılığı `BlockPos::square_out_south_east` yordamıdır.

## Kapsam dışı

Diğer BlockPos stream/iterator yardımcıları, mutable BlockPos API'si ve AABB overload'ları bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `blockpos_square_out_south_east` işlemi, deprecated dört elemanlı stream sırasını Java oracle ile karşılaştırır.

Java math capture kimliği `6fb8e81c6433b1449a63fad69e23d28ceb9d2dabcff2f18ee9ce87187983cb65` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca dört konumlu `squareOutSouthEast` sırasını kapsar; diğer BlockPos iterator'ları ayrı dilimlerdedir.

Birleşik capture kimliği `d198d18c2ddf3f2b80450d9f9b2ed9ee4218d0843ae84e02da47a67103b08bff` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
