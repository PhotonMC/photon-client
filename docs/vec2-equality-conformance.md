# P01.b — Vec2 equals/hashCode sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec2.equals(Object)` ve `Vec2.hashCode()` metodlarının iki f32 bileşen üzerindeki davranışıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec2.java:45-60` içindeki `equals` ve `hashCode` gövdeleri incelendi. Oracle doğrudan Java çağrılarını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `equals(Object)`, Vec2 olmayan nesneleri reddeder; Vec2 bileşenlerini Java `==` ile karşılaştırır.
- `==` signed zero değerlerini eşit, NaN değerlerini eşitsiz ve aynı işaretli Infinity değerlerini eşit kabul eder.
- `hashCode()`, `Objects.hash(this.x, this.y)` yolunu kullanır: başlangıç değeri 1, her Float hash'i 31 tabanlı Java int aritmetiğiyle eklenir.
- Float hash'i NaN'i `0x7fc00000` bitlerine canonicalize eder; signed zero hash bitleri korunur. Java'daki eşitlik/hash ayrışması ham sonuçlarla korunur.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Aynı sonlu Vec2 | Eşitlik ve iki hash |
| `different-component` | Tek bileşeni farklı Vec2 | Eşitsizlik ve hash farkı |
| `signed-zero` | Negatif/pozitif sıfır | Java `==` eşitliği ve farklı hash bitleri |
| `nan-payloads` | Farklı NaN payload'ları | Eşitsizlik ve canonical hash'ler |
| `infinities` | Aynı pozitif/negatif Infinity | Eşitlik ve Infinity hash'leri |
| `special-mismatch` | Farklı Infinity ve NaN bileşenleri | Eşitsizlik ve hash sonuçları |

Girdiler `family: "math"`, `operation: "vec2_equality"` ve f32 bit dizgeli `first`/`second` Vec2 nesnelerinden oluşur. Sonuç `equals` boolean'ı ile iki Java int hash değerini decimal string olarak taşır.

## Rust karşılığı

`src/math.rs` içindeki `Vec2::equals_java` ve `Vec2::hash_code_java`, Java'nın f32 `==`, canonical NaN ve wrapping int hash aritmetiğini uygular. `examples/behavior_adapter.rs` sonucu taşır; `tests/behavior/java/MathOracle.java` aynı Java metodlarını çağırır.


## Client-parity doğrulaması

`src/math.rs`, Java `Vec2.equals(Object)` ve `Vec2.hashCode()` metodlarını karşılayan `Vec2::equals_java` ve `Vec2::hash_code_java` metodlarını içerir. Java oracle doğrudan iki Java metodunu çağırır; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Altı yeni senaryoda f32 `==`, signed zero/NaN eşitliği, canonical NaN hash'i, Infinity ve 31 tabanlı Java int hash aritmetiği karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliklerindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt altı özgün sentetik Vec2 çifti ve equals/hashCode sonuçlarıyla sınırlıdır. Vec2 negated, add, dot, rotate ve distanceToSqr ayrı davranış dilimleridir.
