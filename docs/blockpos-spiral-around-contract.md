# P01.c — BlockPos spiralAround sözleşmesi

Bu dilim, BlockPos'ın iki farklı eksendeki yönle kurulan spiral iterator'unu ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:445-481` incelendi. Yardımcı yönleri `[first, second, first.opposite, second.opposite]` dizisine koyar; cursor ikinci yönde merkezden başlar, ilk hesapta karşıt ikinci adımla merkeze gelir. Leg boyu `leg / 2 + 1` olarak büyür ve toplam leg sayısı `4 * radius` olur.

## Davranış sözleşmesi

- Her leg yönü ve leg geçiş sırası Java kaynak düzeniyle korunur.
- İlk gözlenen konum merkezdir; sonraki konumlar first/second ve karşıt yönlerle aynı cursor algoritmasından çıkar.
- Radius sıfırda yalnızca merkez sonuçlanır; negatif radius finite normal girdiler için boş sonuç verir.
- Direction adımlarındaki koordinat güncellemeleri Java i32 sarmasıyla yapılır.
- Mutable cursor sonuçları capture sırasında bağımsız BlockPos snapshot'larına dönüştürülür.
- Aynı eksendeki iki yön Java precondition'ıdır ve bu dilimin geçerli girdileri dışında bırakılır.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_spiral_around"`, `center` BlockPos nesnesi, signed decimal `radius` ve lower-case `first_direction`/`second_direction` alanlarını taşır. Sonuç `positions` snapshot listesi içerir.

Java oracle `BlockPos.spiralAround` iterable'ını doğrudan tüketir. Rust karşılığı `BlockPos::spiral_around` yordamıdır.

## Kapsam dışı

Aynı eksenli yön precondition hatasının metni, mutable BlockPos API'si, breadth-first traversal ve diğer iterator'lar bu dilime dahil değildir.
