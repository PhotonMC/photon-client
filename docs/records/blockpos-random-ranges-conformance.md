# P01.c — BlockPos random range sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, BlockPos'ın `randomBetweenClosed` ve `randomInCube` random iterator'larını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:247-285` incelendi. `randomInCube`, merkezden boyut çıkarıp ekleyerek altı sınırı kurar ve `randomBetweenClosed` metoduna yönlenir. Random iterator her elemanda x, sonra y, sonra z için birer bounded random çekimi yapar; limit sıfır veya negatifse çekim yapmadan biter.

## Davranış sözleşmesi

- Her konum için random draw sırası x, y, z olarak korunur.
- Her eksen sonucu ilgili inclusive aralığın alt sınırından başlayarak `nextInt(width)` sonucu ile Java i32 toplamasıyla kurulur.
- `randomInCube` sınırları merkez ± boyut işlemlerinin Java i32 sarmasıyla hesaplanır.
- Limit `<= 0` olduğunda iterator boş kalır ve random kaynağı tüketilmez.
- Mutable cursor çıktıları capture sırasında bağımsız BlockPos snapshot'larına çevrilir; sonrasındaki `nextInt` değerleri tüketim sırasını görünür kılar.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_random_ranges"`, signed decimal seed/limit/altı sınır, `center` BlockPos ve `size` alanlarını taşır. Sonuç `between`, `after_between`, `cube` ve `after_cube` alanlarını içerir.

Java oracle iki overload'u aynı seed ile ayrı random kaynaklarda tüketir. Rust karşılığı `BlockPos::random_between_closed` ve `BlockPos::random_in_cube` yordamlarıdır.

## Kapsam dışı

`withinManhattan`, `spiralAround`, random kaynak implementasyonunun diğer overload'ları ve invalid bound exception metinleri bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `blockpos_random_ranges` işlemi, `randomBetweenClosed` ve `randomInCube` sonuçlarını Java oracle ile karşılaştırır.

Java math capture kimliği `56b3daf65646309d91103555670e86d200d948a91693d48937031ba1dd38eac9` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt finite limit ve geçerli bounded aralıklarla random draw sırasını, limit kısa yolunu ve `randomInCube` merkez forwarding'ini kapsar; invalid bound exception ayrıntıları kapsam dışıdır.

Birleşik capture kimliği `3051db48b90ed8689e2a77545c64b335b50076ec13f7757a51a94a0b03539131` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
