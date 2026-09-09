# P01.e — `nextGaussian` referans sözleşmesi

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](random-gaussian-parity.md)). Kapsam, `RandomSource.create(seed)` ile oluşturulan legacy kaynağın `nextGaussian` çağrısını, polar örnekleme döngüsünü ve ikinci Gaussian değerinin önbellekten tüketilmesini içerir. `fork`, positional random factory ve seed sıfırlama ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `util/RandomSource.java:35-57`, `world/level/levelgen/LegacyRandomSource.java:15-53` ve `world/level/levelgen/MarsagliaPolarGaussian.java:6-38` incelendi. `MathOracle` doğrudan `RandomSource.create(seed)` ve `nextGaussian` çağrılarını çalıştırır; beklenen değerleri yeniden hesaplamaz.

## Davranış sözleşmesi

- Kaynak, her Gaussian çifti için iki `nextDouble` ile `x` ve `y` değerlerini `[-1, 1)` aralığına taşır. Kare yarıçapı `1` veya daha büyük ya da sıfırsa yeni çift denenir.
- Kabul edilen çiftte `sqrt(-2 * log(radiusSquared) / radiusSquared)` çarpanı uygulanır. İlk sonuç döner, eş sonucu ikinci çağrı için önbelleğe alınır.
- Gaussian sonuçları ham f64 IEEE-754 bitleriyle taşınır. Aradaki `nextInt`/`nextDouble` adımları kaynak tüketim sırasını görünür kılar.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Gaussian ardışık çağrılar | 5 | 0, 1, -1, 12345 ve i64 maksimum seed'leri; üç Gaussian çağrısı |
| Gaussian aralıklı çağrılar | 1 | `nextInt`, iki önbellek ilişkili Gaussian, `nextDouble` ve yeni Gaussian |

`family: "math"`, `operation: "random"` ortak girdilerdir. `steps` içindeki `op: "nextGaussian"` sonucu `bits` alanında 16 haneli küçük harfli hex olarak döner.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni kimlik, mevcut random/AABB kayıtlarının üzerine yazmadan oluşturulur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.

329 math senaryosu iki ayrı JVM'de aynı sonucu verdi. Math referans kimliği: `5c0a43bc10b588fbff0d141e430711057cf9e06b60cbc2bd9d6fe4fde4286159`.

Altı ailenin 1149 senaryolu birleşik referansı iki JVM'de doğrulandı. Birleşik referans kimliği: `5f4fd0e48a09b99596c7d025768d8535f8c5ee702c6ef4dffd614eafae39c7e5`.

Math debug ve release Rust adaptörleri **329/329** eşleşti; birleşik debug ve release karşılaştırmaları da **1149/1149** geçti. Yerel raporlar `compare-c3rqn2f5` (math debug), `compare-vdvn52ql` (math release), `compare-oiqblje5` (birleşik debug) ve `compare-pl0oao3t` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

Bu sözleşme yalnızca legacy `nextGaussian` ve seçili çağrı sıralarını kapsar. `setSeed` sonrası Gaussian önbelleğinin temizlenmesi ve istemcinin Gaussian kullanan daha geniş yolları sonraki testlerdir; `fork`/`forkPositional` için ayrı kanıt vardır.
