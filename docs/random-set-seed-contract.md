# P01.e — `setSeed` referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](random-set-seed-parity.md)). Kapsam, `RandomSource.create(seed)` ile oluşturulan legacy kaynağın `setSeed` çağrısını, 48 bitlik LCG durumunun yeniden kurulmasını ve mevcut Gaussian önbelleğinin temizlenmesini içerir. Farklı `RandomSource` uygulamaları ve overflow ile oluşan geçersiz bound hataları ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `util/RandomSource.java:35-57`, `world/level/levelgen/LegacyRandomSource.java:15-38` ve `world/level/levelgen/MarsagliaPolarGaussian.java:6-17` incelendi. `MathOracle` doğrudan Java `setSeed` ve çekim metotlarını çalıştırır; beklenen değerleri yeniden hesaplamaz.

## Davranış sözleşmesi

- `setSeed(seed)`, dışarıdan verilen signed long'u Java'nın multiplier sabitiyle XOR'layıp 48 bit maskeyle LCG durumuna dönüştürür.
- Çağrı, Gaussian üreticisinin önbelleğini temizler. Önceden üretilmiş fakat henüz dönmemiş eş değer, yeniden seed işleminden sonra gözlenmez.
- `setSeed` rastgele değer üretmez; JSONL olayı yalnızca `{"op":"setSeed"}` alanını taşır. Sonraki değerler yeni seed'in ilk çekiminden başlar.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Gaussian cache reset | 1 | Gaussian üret, aynı seed'i uygula, ilk ve eş Gaussian'ı yeniden gözle |
| state reset | 1 | Çekimlerden sonra sıfır seed'ine dön ve double/Gaussian tüketimini karşılaştır |
| boundary seeds | 1 | i64 maksimum ve minimum seed ile int/long başlangıçlarını karşılaştır |

`family: "math"`, `operation: "random"` girdileridir. `steps` içindeki `setSeed` adımı yeni seed'i signed decimal `seed` alanında taşır; değer üreten adımlar önceki random sözleşmesindeki bit/ondalık biçimleri korur.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni kimlik, önceki random/Gaussian/fork kayıtlarının üzerine yazmadan oluşturulur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.

337 math senaryosu iki ayrı JVM'de aynı sonucu verdi. Math referans kimliği: `8dc5cad800d19ec1507e2454e97aeb79e53b77267dddada36f77b45771b44f99`.

Altı ailenin 2501 senaryolu birleşik referansı iki JVM'de doğrulandı. Birleşik referans kimliği: `838ea33f41ccc8a9281f9614ce843c3bb820ec822d6601ba52ebcb496a9f6040`.

Rust debug ve release adaptörleri math ailesinde **337/337**, birleşik corpus'ta **2501/2501** eşleşti. Yerel raporlar `compare-yxttfhcx` (math debug), `compare-h6jl6e0l` (math release), `compare-h1kbcaxz` (birleşik debug) ve `compare-js7_mas4` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

Bu sözleşme legacy kaynağın `setSeed` ve Gaussian cache reset yoluyla sınırlıdır. Xoroshiro/ThreadSafe kaynakları, overflow ile oluşan geçersiz bound hataları ve istemcinin daha geniş random tüketim sıraları sonraki testlerdir.
