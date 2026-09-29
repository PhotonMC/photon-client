# P01.d — AABB equals/hashCode sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, iki AABB arasındaki endpoint eşitliği ile Java `hashCode()` hesabını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:91-126` içindeki `equals(Object)` ve `hashCode()` yolları incelendi. `MathOracle`, karşılaştırılan AABB'leri Java constructor'ı ile oluşturup doğrudan bu metotları çağırır.

## Davranış sözleşmesi

- Java, altı endpoint'i `Double.compare` ile sırayla karşılaştırır; AABB olmayan nesne eşleşmez.
- `Double.compare` signed zero değerlerini ayırır ve bütün NaN payload'larını eşit kabul eder.
- `hashCode()` her endpoint'te `Double.doubleToLongBits` kullanır; NaN payload'larını canonical değere indirger, signed zero ayrımını korur ve sonuçları Java'nın 31 tabanlı signed int taşmasıyla birleştirir.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `identical` | Aynı normal endpoint'lere sahip iki AABB | `equals` ve hash sonucu |
| `different-endpoint` | Tek üst endpoint'i farklı AABB | Eşitsizlik ve hash sonucu |
| `signed-zero` | `-0.0` ve `+0.0` alt endpoint'leri | Signed-zero eşitsizliği ve hash sonucu |
| `nan-payload` | Farklı NaN payload'ları taşıyan endpoint'ler | NaN eşitliği ve canonical hash sonucu |
| `infinities` | Sonsuz endpoint'li AABB ve sonlu AABB | Sonsuzluk eşitliği, eşitsizlik ve hash sonucu |

Girdiler `family: "math"`, `operation: "aabb"` ve `steps` içindeki `op: "equals"` veya `op: "hash_code"` alanlarını kullanır. `equals` AABB operand'ını, `hash_code` mevcut kutunun Java hash değerini event olarak taşır.

## Java yakalama

Math ve birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle iki JVM çalıştırmasında doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/aabb.rs`, Java endpoint karşılaştırmasını `Aabb::equals_java`, Java hash hesabını `Aabb::hash_code_java` ile uygular. `examples/behavior_adapter.rs`, `equals` ve `hash_code` adımlarını bu API'lere taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint eşitliği, signed zero, farklı NaN payload'ları, sonsuzluk ve Java hash sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math ve birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt beş sentetik AABB operand çiftine ve mevcut kutunun hash sorgusuna sınırlıdır. Java `equals(Object)` içindeki AABB olmayan nesne türü dalı Rust tip sınırında ayrı tutulur; AABB dışı world/container eşitliği bu dilimin parçası değildir.
