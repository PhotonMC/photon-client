# P01.e — `fork` ve positional random referans sözleşmesi

Bu dilimin **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](random-fork-parity.md)). Kapsam, legacy `RandomSource` içinden `fork` ve `forkPositional` çağrılarını, parent kaynağın `nextLong` tüketimini, koordinat seed hesabını, Java string hash'ini ve doğrudan seed ile factory üretimini içerir. `setSeed` ve diğer random overload'ları bu dilime dahil değildir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `util/RandomSource.java:35-37`, `world/level/levelgen/LegacyRandomSource.java:15-29,56-85`, `world/level/levelgen/PositionalRandomFactory.java:8-24` ve `util/Mth.java:331-335` incelendi. `MathOracle` doğrudan Java `fork`, `forkPositional`, `at`, `fromHashOf` ve `fromSeed` çağrılarını çalıştırır; beklenen değerleri yeniden hesaplamaz.

## Davranış sözleşmesi

- `fork`, parent kaynaktan bir `nextLong` tüketir ve bu signed long ile yeni bir `LegacyRandomSource` oluşturur. Parent ve child bağımsız çağrı sıralarını korur.
- `forkPositional`, aynı şekilde bir factory seed'i tüketir. `at(x, y, z)`, Java `Mth.getSeed` sonucu ile factory seed'ini XOR'layarak yeni kaynak kurar.
- `fromHashOf`, Java `String.hashCode()` değerini factory seed'iyle XOR'lar; `fromSeed`, factory seed'ini kullanmadan verilen signed long ile yeni kaynak kurar.
- Her child kaynağın sonuçları ham integer/float/double bitleriyle veya signed ondalık değerlerle taşınır. Parent sonuçları factory/child çağrılarından sonra ayrıca gözlenir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| `fork` | 2 | Parent Gaussian önbelleği boş/dolu iken child ve parent çağrı dizileri |
| positional coordinates | 1 | Sıfır, i32 sınırları ve karışık negatif koordinatlar |
| positional hash | 1 | Boş, namespaced ve UTF-16 surrogate içeren adlar |
| positional seed | 1 | 0, -1 ve i64 maksimum seed'leri |

`family: "math"`, `operation: "random_fork" | "random_positional"` girdileridir. Her child adımının sonucu `events` dizisinde, parent tüketimi `parent_events` alanında korunur.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni kimlik, önceki random/Gaussian kayıtlarının üzerine yazmadan oluşturulur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.

334 math senaryosu iki ayrı JVM'de aynı sonucu verdi. Math referans kimliği: `aed4472ff0ba0e85f3a9ae2fe61132e8abf7af4145d0d28fcac8a64bb06fedb4`.

Rust debug ve release adaptörleri math ailesinde **334/334**, birleşik corpus'ta **791/791** eşleşti. Yerel raporlar `compare-vzri6usr` (math debug), `compare-03cd0hof` (math release), `compare-tq4mb84i` (birleşik debug) ve `compare-zaisbyxj` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

Bu sözleşme legacy `fork` ve positional factory'nin seçili yollarıyla sınırlıdır. Xoroshiro/ThreadSafe kaynakları, `Identifier` overload'ları ve istemcinin daha geniş random tüketim sıraları sonraki testlerdir; `setSeed` sonrası Gaussian cache temizleme için ayrı [kanıt](random-set-seed-parity.md) vardır.
