# P01.d/e — Random ve AABB referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin 12 Java davranış testi ve Rust karşılaştırması hazırdır. Önceki 295 math senaryosunun girdisi ve sonucu değişmedi. AABB contract, AABB inflate, AABB intersect, AABB minmax, AABB move, AABB sizes, AABB hasNaN, AABB positions, AABB axis min/max, AABB distanceToSqr, AABB ofSize, AABB endpoint setter'ları, AABB clip, fork/dağıtım varyantları ve çoklu shape işlemleri ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `util/RandomSource.java` içindeki deterministik `create(seed)` dizileri ve `world/phys/AABB.java` içindeki `contains`, `intersects`, `expandTowards`, `intersect` ve `deflate` incelendi. Adaptör doğrudan orijinal sınıfları çağırır. Yeni bağımlılık eklenmedi.

## Davranış sözleşmesi

- Aynı seed aynı çağrı sırasını üretir; tamsayılar işaretli ondalık, float/double ham bitleriyle taşınır.
- AABB uçları f64 bitleriyle taşınır; `contains`/`intersects` boolean, `expand`/`deflate` yeni uçlar üretir. Sınır kapsama kuralı Java çıktısıyla sabitlenir.
- Her random senaryosu boş başlangıç olayı ve her çağrı sonrası değer/bit olayı kaydeder. Her AABB senaryosu başlangıç uçları ve her adım sonrası vuruş/sonuç olayı kaydeder.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Random dizi | 5 | Sabit seed'lerde int/float/double/long/boolean sırası |
| Random sınır | 4 | 42 seed'inde 1/2/16/100 sınırlarıyla ikişer çekim |
| AABB kapsama/kesişim/genişleme | 3 | Birim kutuda iç/dış nokta, çakışan/ayrık kutu, genişletme ve büzme |

`family: "math"`, `operation: "random" | "aabb"` ortak girdilerdir.

## Çalıştırma ve kanıt

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

307 math senaryosu iki ayrı JVM'de aynı sonucu verdi. Math referans kimliği: `529c11520bbda1959798018b965d4f6a3d2f46de0527d0ef9d802b210e08f91c`. Önceki 295 girdisi korunur. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Toplam 1049 Java senaryosu yakalandı ve doğrulandı. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`. Eski kayıtlar değiştirilmedi.

Debug ve release Rust adaptörleri **307/307** eşleşti: 295 önceki math senaryosu ve bu dilimin 12 random/AABB senaryosu. Karşılaştırma random dizilerinin tamsayı/float/double/boolean olaylarını ve AABB uçları, üyelik, kesişim, genişletme ve büzme sonuçlarını kesin bit karşılaştırmasıyla kapsadı.

Yerel raporlar: `compare-37kc_otn` (debug), `compare-q246ny7w` (release) dizinlerinde, `.data/runs/behavior/` altında bulunur. Ayrıntılı Rust API'si [parite kanıtında](#client-parity-doğrulaması) kayıtlıdır.


## Client-parity doğrulaması

`crates/voxel-core/src/random.rs` ve `crates/voxel-core/src/aabb.rs`, Java 26.2 referansından gözlenen random dizilerini ve temel AABB işlemlerini içerir. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryolarına bağlar; beklenen değer üretmez. Çoklu AABB shape birleşimleri sonraki dilimlerdir; tek kutulu segment `clip` yolu [ayrı dilimde](aabb-clip-conformance.md), `intersect` yolu [ayrı dilimde](aabb-intersect-conformance.md), `minmax` yolu [ayrı dilimde](aabb-minmax-conformance.md), scalar `move` yolu [ayrı dilimde](aabb-move-conformance.md), `sizes` yolu [ayrı dilimde](aabb-sizes-conformance.md), `hasNaN` yolu [ayrı dilimde](aabb-has-nan-conformance.md), `positions` yolu [ayrı dilimde](aabb-positions-conformance.md), `axis min/max` yolu [ayrı dilimde](aabb-axis-limits-conformance.md), `ofSize` yolu [ayrı dilimde](aabb-of-size-conformance.md), endpoint setter'ları [ayrı dilimde](aabb-set-endpoint-conformance.md), `nextGaussian` yolu [ayrı dilimde](random-gaussian-conformance.md), `fork`/positional yolları [ayrı dilimde](random-fork-conformance.md) ve `setSeed` yolu [ayrı dilimde](random-set-seed-conformance.md) doğrulandı.

## Referans ve davranış

Java `RandomSource.create(seed)` yolu `LegacyRandomSource` seçer. Rust kaynağı seed'i Java'daki 48 bitlik başlangıç dönüşümüyle kurar; her çekimde LCG durumunu maskeler. Bounded int için güç-of-two ve rejection dalları ile Java'nın işaretli i32 taşması korunur. `nextLong`, Java'daki signed lower-half eklemesini; float ve double çekimleri referans bit genişliklerini ve ölçeklerini korur.

`Aabb::new` her eksen çiftini Java kurucusu gibi normalize eder. `contains` alt sınırı dahil, üst sınırı hariç tutar. `intersects` üç eksende de sıkı örtüşme ister; yalnızca yüze temas eden kutular kesişmiş sayılmaz. `expand_towards` delta işaretine göre tek uçta ilerler; `deflate` Java'nın `inflate(-amount)` yolundaki endpoint aritmetiğini uygular. Kurucu karşılaştırmaları NaN ve signed-zero için Java `Math.min`/`Math.max` davranışını taşır.

| Rust API | Korunan davranış |
| --- | --- |
| `RandomSource::create` | Java seed scrambling ve 48 bit LCG durumu |
| `next_int` / `next_int_bound` | 32 bit çekim, pozitif bound, power-of-two ve rejection örneklemesi |
| `next_long` | İki 32 bit çekimin Java signed long birleştirmesi |
| `next_float` / `next_double` | 24/53 bit çekim ve ham IEEE-754 sonuçları |
| `next_boolean` | Tek bitten boolean üretimi |
| `Aabb::new` | Her eksende min/max normalizasyonu |
| `Aabb::contains` / `intersects` | Java sınır ve sıkı kesişim koşulları |
| `Aabb::expand_towards` / `deflate` | Endpoint seçimi ve f64 aritmetik sırası |

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math referansı 307 senaryoda iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **307/307** eşleşti.
- 5 random dizi, 4 bounded random ve 3 AABB senaryosunda olay sırası, signed tamsayı metinleri, float/double ham bitleri, AABB uçları ve boolean sonuçları karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve beş sentetik davranış-runner testi geçti.

Math referans kimliği: `529c11520bbda1959798018b965d4f6a3d2f46de0527d0ef9d802b210e08f91c`.

Yerel raporlar: `compare-37kc_otn` (debug) ve `compare-q246ny7w` (release), `.data/runs/behavior/` altında.

Bu kanıt yalnızca 307 seçili math senaryosuyla sınırlıdır. Tek kutulu AABB segment clip, `nextGaussian`, fork/positional ve setSeed yolları için ayrı kanıtlar vardır. P01'in çoklu shape yolları ve istemcinin diğer davranışları henüz eşitlik kanıtı değildir.
