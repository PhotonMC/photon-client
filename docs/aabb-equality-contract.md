# P01.d — AABB equals/hashCode sözleşmesi

Bu dilimde **5 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-equality-parity.md)). Kapsam, iki AABB arasındaki endpoint eşitliği ile Java `hashCode()` hesabını içerir.

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

Math ve birleşik altı aile referansı `6251a239b5efd4862d79c5b740dcb208ec9e1932f8a04d0d86c05a044353fd90` kimliğiyle iki JVM çalıştırmasında doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
