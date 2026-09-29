# P01.e — `setSeed` referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `RandomSource.create(seed)` ile oluşturulan legacy kaynağın `setSeed` çağrısını, 48 bitlik LCG durumunun yeniden kurulmasını ve mevcut Gaussian önbelleğinin temizlenmesini içerir. Farklı `RandomSource` uygulamaları ve overflow ile oluşan geçersiz bound hataları ayrı dilimlerdir.

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

Altı ailenin 2801 senaryolu birleşik referansı iki JVM'de doğrulandı. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Rust debug ve release adaptörleri math ailesinde **337/337**, birleşik corpus'ta **2801/2801** eşleşti. Yerel raporlar `compare-yxttfhcx` (math debug), `compare-h6jl6e0l` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

Bu sözleşme legacy kaynağın `setSeed` ve Gaussian cache reset yoluyla sınırlıdır. Xoroshiro/ThreadSafe kaynakları, overflow ile oluşan geçersiz bound hataları ve istemcinin daha geniş random tüketim sıraları sonraki testlerdir.


## Client-parity doğrulaması

`crates/voxel-core/src/random.rs`, Java legacy random kaynağının `setSeed` ile LCG durumunu yeniden kurmasını ve Gaussian önbelleğini temizlemesini uygular. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryolarına bağlar; beklenen değer üretmez.

## Referans ve davranış

Java `LegacyRandomSource.setSeed` signed seed'i 48 bitlik başlangıç durumuna dönüştürür ve `MarsagliaPolarGaussian` önbelleğini sıfırlar. Rust constructor'ı da aynı reset yolunu kullanır; sonradan çağrılan `set_seed` eski eş Gaussian'ı korumaz.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 337 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **337/337** eşleşti: önceki 334 math senaryosu ve 3 yeni setSeed senaryosu.
- Altı ailenin birleşik corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Gaussian önbelleği doluyken aynı seed'e dönme, yeniden seed sonrası yeni LCG başlangıcı ve i64 sınır seed'leri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `8dc5cad800d19ec1507e2454e97aeb79e53b77267dddada36f77b45771b44f99`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-yxttfhcx` (math debug), `compare-h6jl6e0l` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt legacy `setSeed` ve Gaussian cache reset yolunun seçili 3 yeni senaryosuyla sınırlıdır. `nextIntBetweenInclusive`, origin/bound overload'ları, triangle/consumeCount, Xoroshiro/ThreadSafe kaynakları ve istemcinin daha geniş random tüketim sıraları sonraki dilimlerdir.
