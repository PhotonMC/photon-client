# P01.d — AABB.expandTowards(Vec3) sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `AABB.expandTowards(Vec3)` overload'ının bileşenleri scalar `expandTowards(double, double, double)` yoluna aktarmasını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:157-163` overload yönlendirmesi ve `AABB.java:161-189` scalar genişletme yolu incelendi. `MathOracle`, Vec3 overload'ını doğrudan çağırır; beklenen kutu yeniden hesaplanmaz.

## Davranış sözleşmesi

- Vec3 overload'ı `delta.x`, `delta.y` ve `delta.z` bileşenlerini scalar overload'a aynı sırayla verir.
- Pozitif delta ilgili eksenin max endpoint'ini, negatif delta min endpoint'ini değiştirir; sıfır endpoint'i korur.
- Sonuç constructor normalizasyonu ve Java double bit davranışıyla taşınır; signed zero girdisi korunur.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `vec3` | Pozitif X, negatif Y ve sıfır Z delta'sı | Genişletilmiş AABB snapshot'ı |
| `signed-zero` | Signed-zero endpoint'li kutu ve zero Vec3 | Endpoint bitleri ve değişmeyen kutu |
| `chain` | İki ardışık Vec3 genişletme adımı | Her adım sonrası AABB snapshot'ı |

Girdiler `family: "math"`, `operation: "aabb"` ve `steps` içindeki `op: "expand_vec3"` alanlarını kullanır. Vec3 bileşenleri f64 bit string'leridir; `events` başlangıç ve her adım sonrası kutuyu taşır.

## Java yakalama

Math ve birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle iki JVM çalıştırmasında doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`src/aabb.rs`, Java `AABB.expandTowards(Vec3)` overload'ını `Aabb::expand_towards_vec3` ile uygular. `examples/behavior_adapter.rs`, `op: "expand_vec3"` adımını f64 Vec3 bileşenleriyle bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `expand_vec3` senaryosunda bileşen yönlendirmesi, signed zero ve ardışık genişletme sonuçları karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math ve birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik Vec3 overload girdisiyle sınırlıdır. Scalar `expandTowards(double, double, double)` yolu, diğer AABB dilimindeki senaryolarla ayrı doğrulanır; world collider toplama ve hareket çözümü bu kapsamda değildir.
