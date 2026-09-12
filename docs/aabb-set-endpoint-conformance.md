# P01.d — AABB endpoint setter sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java `AABB.setMinX/Y/Z` ve `AABB.setMaxX/Y/Z` metotlarının tek endpoint'i değiştirerek yeni kutu üretmesini kapsar. Altı Java davranış testi hazırlandı ve Rust karşılığı [doğrulandı](#client-parity-doğrulaması).

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:22-29,59-85` içindeki constructor ve altı setter gövdesi incelendi. `MathOracle` her setter'ı doğrudan çağırır; beklenen endpoint'leri oracle içinde yeniden hesaplamaz.

## Davranış sözleşmesi

- Her setter yalnızca adındaki tek endpoint'i yeni değerle değiştirir; kalan beş saklanan endpoint korunur.
- Her çağrı yeni `AABB` nesnesi üretir ve altı endpoint'i constructor'a verir.
- Constructor her eksen çiftini Java `Math.min`/`Math.max` davranışıyla yeniden normalize eder. Yeni min değeri mevcut max'i aşarsa veya yeni max değeri mevcut min'in altına inerse saklanan uçların sırası buna göre düzelir.
- Setter çağrıları zincirlenebilir; her adımın sonucu sonraki setter'ın girdisidir.
- Sıfır, signed zero, sonsuzluk ve NaN değerlerinin IEEE-754 double bitleri gözlenebilir çıktının parçasıdır.
- Setter'lar mevcut kutuyu değiştirmez; her adım yeni kutu snapshot'ı üretir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Tüm setter'lar | 1 | Altı setter'ı sırayla sonlu değerlerle çağırır |
| Min zinciri | 1 | `setMinX`, `setMinY`, `setMinZ` zinciri |
| Max zinciri | 1 | `setMaxX`, `setMaxY`, `setMaxZ` zinciri |
| Ters endpoint | 1 | Setter değerleriyle eksenleri ters çevirir ve normalize eder |
| Signed zero | 1 | Altı setter'da pozitif/negatif sıfır |
| Özel değerler | 1 | NaN ve sonsuzluk setter girdileri |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "set_min_x"`, `set_min_y`, `set_min_z`, `set_max_x`, `set_max_y` veya `set_max_z`; yeni endpoint ise `value` f64 bit string'i olarak taşınır. Her adım kutu snapshot'ı ekler.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Setter senaryolarını içeren math referans kimliği `80065cae9313e93b5d45819cd549e3361f69569df75f00abcd445b0b2b65b506` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Rust ve birleşik corpus kanıtı [parity belgesindedir](#client-parity-doğrulaması).

Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`src/aabb.rs`, Java `AABB.setMinX/Y/Z` ve `AABB.setMaxX/Y/Z` metotlarının tek endpoint'i değiştirip yeni AABB constructor'ı üzerinden normalize etmesini uygular. `examples/behavior_adapter.rs` altı setter adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 436 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **436/436** math senaryosunda eşleşti.
- Altı yeni senaryoda tüm min/max setter'ları, min ve max zincirleri, ters endpoint normalizasyonu, signed zero, sonsuzluk ve NaN karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `80065cae9313e93b5d45819cd549e3361f69569df75f00abcd445b0b2b65b506`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-53_472mo` (math debug), `compare-tta3azob` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.setMinX/Y/Z` ve `AABB.setMaxX/Y/Z` setter'ları ile mevcut AABB adım adaptörüyle sınırlıdır. `AABB.move(BlockPos/Vec3/Vector3f)` için [ayrı kanıt](aabb-move-overloads-conformance.md) vardır; [`collidedAlongVector`](aabb-collided-along-vector-conformance.md) ayrı dilimde tamamlandı, Çoklu AABB clip sonraki dilimdir.
