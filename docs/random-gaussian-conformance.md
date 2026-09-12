# P01.e — `nextGaussian` referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `RandomSource.create(seed)` ile oluşturulan legacy kaynağın `nextGaussian` çağrısını, polar örnekleme döngüsünü ve ikinci Gaussian değerinin önbellekten tüketilmesini içerir. `fork`, positional random factory ve seed sıfırlama ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `util/RandomSource.java:35-57`, `world/level/levelgen/LegacyRandomSource.java:15-53` ve `world/level/levelgen/MarsagliaPolarGaussian.java:6-38` incelendi. `MathOracle` doğrudan `RandomSource.create(seed)` ve `nextGaussian` çağrılarını çalıştırır; beklenen değerleri yeniden hesaplamaz.

## Davranış sözleşmesi

- Kaynak, her Gaussian çifti için iki `nextDouble` ile `x` ve `y` değerlerini `[-1, 1)` aralığına taşır. Kare yarıçapı `1` veya daha büyük ya da sıfırsa yeni çift denenir.
- Kabul edilen çiftte `sqrt(-2 * log(radiusSquared) / radiusSquared)` çarpanı uygulanır. İlk sonuç döner, eş sonucu ikinci çağrı için önbelleğe alınır.
- Gaussian sonuçları ham f64 IEEE-754 bitleriyle taşınır. Aradaki `nextInt`/`nextDouble` adımları kaynak tüketim sırasını görünür kılar.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Gaussian ardışık çağrılar | 5 | 0, 1, -1, 12668 ve i64 maksimum seed'leri; üç Gaussian çağrısı |
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

Altı ailenin 2801 senaryolu birleşik referansı iki JVM'de doğrulandı. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Math debug ve release Rust adaptörleri **329/329** eşleşti; birleşik debug ve release karşılaştırmaları da **2801/2801** geçti. Yerel raporlar `compare-c3rqn2f5` (math debug), `compare-vdvn52ql` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

Bu sözleşme yalnızca legacy `nextGaussian` ve seçili çağrı sıralarını kapsar. `setSeed` sonrası Gaussian önbelleğinin temizlenmesi ve istemcinin Gaussian kullanan daha geniş yolları sonraki testlerdir; `fork`/`forkPositional` için ayrı kanıt vardır.


## Client-parity doğrulaması

`src/random.rs`, Java `LegacyRandomSource.nextGaussian` yolunun polar örnekleme ve eş sonuç önbelleğini uygular. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryolarına bağlar; beklenen değer üretmez.

## Referans ve davranış

Java `RandomSource.create(seed)` legacy kaynağı seçer. `MarsagliaPolarGaussian` iki `nextDouble` ile birim disk içinde örnek arar, kabul edilen çiftten iki Gaussian sonucu hesaplar ve ikinci sonucu sonraki çağrıya saklar. Rust aynı çağrı sırasını ve sonuçları ham f64 bitleriyle taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 329 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **329/329** eşleşti: 323 önceki math senaryosu ve 6 yeni Gaussian senaryosu.
- Altı ailenin birleşik corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Sonuç bitleri, önbellekten dönen ikinci değer ve Gaussian çağrıları arasındaki `nextInt`/`nextDouble` tüketim sırası karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `5c0a43bc10b588fbff0d141e430711057cf9e06b60cbc2bd9d6fe4fde4286159`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-c3rqn2f5` (math debug), `compare-vdvn52ql` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt legacy `nextGaussian` için seçili 6 yeni senaryoyla sınırlıdır. Diğer random overload'ları ve Gaussian kullanan istemci yolları sonraki dilimlerdir; `setSeed` için ayrı [kanıt](random-set-seed-conformance.md), `fork`/`forkPositional` için ayrı [kanıt](random-fork-conformance.md) vardır.
