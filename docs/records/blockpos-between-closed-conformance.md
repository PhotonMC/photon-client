# P01.c — BlockPos between-closed sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, BlockPos'ın iki BlockPos ucu ve altı int koordinatı alan inclusive `betweenClosed` yollarını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:356-418` incelendi. BlockPos overload'u her eksende min/max seçerek koordinat overload'una yönlenir. Koordinat overload'u `width`, `height`, `depth` ve `end` değerlerini Java int aritmetiğiyle kurar; cursor x'i hızlı, sonra y, sonra z ilerler.

## Davranış sözleşmesi

- BlockPos uçları her eksende bağımsız normalize edilir ve inclusive olarak dahil edilir.
- Koordinat overload'unda index x ekseninde, `index / width` içindeki slice y ekseninde, sonraki bölüm z ekseninde tüketilir.
- Her sonuç BlockPos snapshot'ı olarak gözlemlenir; Java mutable cursor'ın önceki sonuçları değiştirmemesi capture sırasında sabitlenir.
- Genişlik, yükseklik, derinlik ve koordinat toplamlarındaki int aritmetiği Java i32 sarmasıyla ele alınır; bu dilimde sonlu küçük aralıklar kullanılır.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_between_closed"`, nested `from`/`to` BlockPos uçları ve signed decimal `min_x`/`min_y`/`min_z`/`max_x`/`max_y`/`max_z` alanlarını taşır. Sonuç `from_to` ve `coordinates` listelerini içerir.

Java oracle iki `BlockPos.betweenClosed` overload'unu doğrudan tüketir. Rust karşılığı `BlockPos::between_closed` ve `between_closed_coordinates` yordamlarıdır.

## Kapsam dışı

`betweenClosed(AABB)`, stream/spliterator ayrıntıları, mutable BlockPos dönüşümleri, `withinManhattan`, random iterasyonlar ve `neighborColumn` bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `blockpos_between_closed` işlemi, BlockPos inclusive range overload'larını Java oracle ile karşılaştırır.

Java math capture kimliği `0c2b4fba705ce69727eba2b2d4cd8a33879b1de13fcc17988d4e1c16eb08b518` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `00bd6c47a3e256fc3b8aa0f0cd020abced4e22cdb9ce64519f8b9e2e27e5aeb3` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca finite BlockPos between-closed listelerini ve x/y/z traversal sırasını kapsar; AABB/stream overload'ları ayrı dilimlerdedir.
