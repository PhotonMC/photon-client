# P01.d/e — Random ve AABB Rust davranış dilimi

`src/random.rs` ve `src/aabb.rs`, Java 26.2 referansından gözlenen random dizilerini ve temel AABB işlemlerini içerir. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryolarına bağlar; beklenen değer üretmez. Çoklu AABB shape birleşimleri sonraki dilimlerdir; tek kutulu segment `clip` yolu [ayrı dilimde](aabb-clip-parity.md), `intersect` yolu [ayrı dilimde](aabb-intersect-parity.md), `minmax` yolu [ayrı dilimde](aabb-minmax-parity.md), scalar `move` yolu [ayrı dilimde](aabb-move-parity.md), `sizes` yolu [ayrı dilimde](aabb-sizes-parity.md), `hasNaN` yolu [ayrı dilimde](aabb-has-nan-parity.md), `positions` yolu [ayrı dilimde](aabb-positions-parity.md), `axis min/max` yolu [ayrı dilimde](aabb-axis-limits-parity.md), `ofSize` yolu [ayrı dilimde](aabb-of-size-parity.md), endpoint setter'ları [ayrı dilimde](aabb-set-endpoint-parity.md), `nextGaussian` yolu [ayrı dilimde](random-gaussian-parity.md), `fork`/positional yolları [ayrı dilimde](random-fork-parity.md) ve `setSeed` yolu [ayrı dilimde](random-set-seed-parity.md) doğrulandı.

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
