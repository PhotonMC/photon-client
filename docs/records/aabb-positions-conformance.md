# P01.d — AABB positions sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `AABB.getCenter()`, `getBottomCenter()`, `getMinPosition()` ve `getMaxPosition()` sorgularının Vec3 sonuçlarını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:447-460` ve midpoint hesabının kullandığı `util/Mth.java:558-560` incelendi. `MathOracle` dört getter'ı doğrudan çağırır; beklenen koordinatları oracle içinde yeniden hesaplamaz.

## Davranış sözleşmesi

- `getCenter()` her eksen için `Mth.lerp(0.5, min, max)` çağrısıyla midpoint Vec3 üretir; lerp'in çıkarma, çarpma ve toplama sırası korunur.
- `getBottomCenter()` x ve z midpoint'lerini aynı şekilde üretir, y bileşeninde saklanan `minY` değerini kullanır.
- `getMinPosition()` ve `getMaxPosition()` kutunun saklanan altı endpoint'ini sırasıyla Vec3 bileşenleri olarak döndürür.
- Constructor normalizasyonu ve signed zero/sonsuz/NaN IEEE-754 sonuçları sorgulara aynen yansır. Getter'lar kutuyu değiştirmez.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Birim ve asimetrik | 2 | Birim kutu ve farklı eksen uzunlukları |
| Normalizasyon | 1 | Ters endpoint sırası |
| Sayısal ayrıntı | 2 | Signed zero ve sonsuz endpoint'ler |
| Zincir | 1 | Sorgu, move ve yeniden sorgu sırası |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "positions"` bulunur. Her positions sonucu `center`, `bottom_center`, `min_position` ve `max_position` altında Vec3 snapshot'ları taşır; snapshot alanları x/y/z, length ve length_squared f64 bit string'leridir.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Positions senaryolarını içeren math referans kimliği `130d0f8ac17bbfe7e8c5803d2c729ede2e7a2c581d12f4694d1e95cb513fcf99` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 412/412, birleşik corpus'ta 2801/2801 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/photon-core/src/aabb.rs`, Java `AABB.getCenter()`, `getBottomCenter()`, `getMinPosition()` ve `getMaxPosition()` sorgularını uygular. `examples/behavior_adapter.rs` `positions` adımını, Java'nın Vec3 snapshot alanlarıyla birlikte JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 412 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **412/412** math senaryosunda eşleşti.
- Altı yeni senaryoda normalizasyon, midpoint aritmetiği, signed zero, sonsuzluk ve move sonrası sorgu sırası karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `130d0f8ac17bbfe7e8c5803d2c729ede2e7a2c581d12f4694d1e95cb513fcf99`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-__ojm90z` (math debug), `compare-j97r39u3` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca dört AABB position getter'ı ve mevcut AABB adım adaptörüyle sınırlıdır. [`collidedAlongVector`](aabb-collided-along-vector-conformance.md) ayrı dilimde tamamlandı; diğer overload'lar sonraki dilimlerdir; VoxelShape clip ayrı [kanıtta](shape-clip-conformance.md) tamamlandı.
