# P01.e — `fork` ve positional random referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, legacy `RandomSource` içinden `fork` ve `forkPositional` çağrılarını, parent kaynağın `nextLong` tüketimini, koordinat seed hesabını, Java string hash'ini ve doğrudan seed ile factory üretimini içerir. `setSeed` ve diğer random overload'ları bu dilime dahil değildir.

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

Rust debug ve release adaptörleri math ailesinde **334/334**, birleşik corpus'ta **2801/2801** eşleşti. Yerel raporlar `compare-vzri6usr` (math debug), `compare-03cd0hof` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

Bu sözleşme legacy `fork` ve positional factory'nin seçili yollarıyla sınırlıdır. Xoroshiro/ThreadSafe kaynakları, `Identifier` overload'ları ve istemcinin daha geniş random tüketim sıraları sonraki testlerdir; `setSeed` sonrası Gaussian cache temizleme için ayrı [kanıt](random-set-seed-conformance.md) vardır.


## Client-parity doğrulaması

`src/random.rs`, Java legacy random kaynağının `fork` ve `forkPositional` yollarını uygular. Positional factory koordinat, UTF-16 string hash'i ve doğrudan seed türevleri üretir. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryolarına bağlar; beklenen değer üretmez.

## Referans ve davranış

Java `LegacyRandomSource.fork` parent kaynaktan bir `nextLong` çekip yeni kaynağı bu signed seed ile kurar. `forkPositional` aynı çekimi factory seed'i olarak saklar. `at` Java `Mth.getSeed` aritmetik taşma sırasını, `fromHashOf` Java UTF-16 `String.hashCode()` değerini, `fromSeed` ise factory seed'inden bağımsız doğrudan seed'i kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 334 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **334/334** eşleşti: önceki 329 math senaryosu ve 5 yeni fork/positional senaryosu.
- Altı ailenin birleşik corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Parent `nextLong` tüketimi, child kaynak dizileri, i32 koordinat sınırları, Java UTF-16 hash'i ve Gaussian child sonuçları karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `aed4472ff0ba0e85f3a9ae2fe61132e8abf7af4145d0d28fcac8a64bb06fedb4`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-vzri6usr` (math debug), `compare-03cd0hof` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt legacy `fork` ve positional factory'nin seçili 5 yeni senaryosuyla sınırlıdır. Xoroshiro/ThreadSafe kaynakları, `Identifier` overload'ları ve istemcinin daha geniş random tüketim sıraları sonraki dilimlerdir; `setSeed` sonrası Gaussian cache temizleme için ayrı [kanıt](random-set-seed-conformance.md) vardır.
