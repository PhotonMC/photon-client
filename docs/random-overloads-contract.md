# P01.e — random yardımcı overload sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](random-overloads-parity.md)). Kapsam, legacy `RandomSource` üzerindeki inclusive integer aralığı, origin/bound integer aralığı, double/float triangle örneklemesi ve `consumeCount` çağrılarının kaynak tüketim sırasını içerir. `setSeed` ve `fork`/positional yolları ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `util/RandomSource.java:43-80` incelendi. `MathOracle` doğrudan Java default method çağrılarını çalıştırır; beklenen değerleri yeniden hesaplamaz. Bu dilimde yalnızca geçerli integer aralıkları kullanılır; overflow ile oluşan geçersiz bound hataları sonraki hata sözleşmesidir.

## Davranış sözleşmesi

- `nextIntBetweenInclusive(min, maxInclusive)`, Java i32 aritmetiğiyle `maxInclusive - min + 1` bound'u oluşturur, `[min, maxInclusive]` aralığına kaydırır.
- `nextInt(origin, bound)`, `origin < bound` koşuluyla `[origin, bound)` aralığında değer üretir; origin veya bound sonucu değiştirmeden önce random kaynağı çağrılır.
- `triangle(mean, spread)` iki ardışık aynı tür çekimin farkını spread ile çarpıp mean'e ekler. Double ve float yolları kendi hassasiyetinde gözlenir.
- `consumeCount(rounds)`, yalnızca `rounds` pozitifken her turda bir `nextInt` tüketir. Void çağrı JSONL'de `{"op":"consumeCount"}` olayıyla görünür.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| integer ranges | 1 | Inclusive singleton/negatif aralıklar ve origin/bound aralıkları |
| triangle | 1 | Double ve float mean/spread ile aynı kaynakta ardışık çağrılar |
| consume count | 1 | Pozitif ve negatif round sayıları arasında sonraki int çekimleri |

`family: "math"`, `operation: "random"` girdileridir. Integer sonuçları signed decimal, triangle sonuçları ham float/double bitleriyle taşınır; void yardımcılar yalnızca işlem adını kaydeder.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni kimlik, önceki random/setSeed/Gaussian/fork kayıtlarının üzerine yazmadan oluşturulur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.

340 math senaryosu iki ayrı JVM'de aynı sonucu verdi. Math referans kimliği: `fe90194dbcd2294c66724523fa9ff881dae0f6d36ea62df758b4e64626760bf1`.

Altı ailenin 2465 senaryolu birleşik referansı iki JVM'de doğrulandı. Birleşik referans kimliği: `ef5a23b696bb32be24dcf9aa656adce55733813eb85a06cfd10f6418b5a4c573`.

Rust debug ve release adaptörleri math ailesinde **340/340**, birleşik corpus'ta **2465/2465** eşleşti. Yerel raporlar `compare-57rndqkc` (math debug), `compare-uqisv1v5` (math release), `compare-k17hah5m` (birleşik debug) ve `compare-hiiz6u35` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

Bu sözleşme legacy kaynağın seçili default random yardımcılarıyla sınırlıdır. Overflow kaynaklı geçersiz bound hataları, diğer `RandomSource` implementasyonları ve istemcinin daha geniş random tüketim sıraları sonraki testlerdir.
