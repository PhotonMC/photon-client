# P01.d/e — Random ve AABB referans sözleşmesi

Bu dilimin **12 Java davranış testi hazırdır; Rust implementasyonu henüz yoktur**. Önceki 295 math senaryosunun girdisi ve sonucu değişmedi. Fork/dağıtım varyantları, kalan AABB yardımcıları ve shape/ray işlemleri bu dilime dahil değildir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `util/RandomSource.java` içindeki deterministik `create(seed)` dizileri ve `world/phys/AABB.java` içindeki `contains`, `intersects`, `expandTowards` ve `deflate` incelendi. Adaptör doğrudan orijinal sınıfları çağırır. Yeni bağımlılık eklenmedi.

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

307 math senaryosu iki ayrı JVM'de aynı sonucu verdi. Math referans kimliği: `529c11520bbda1953798018b953d4f6a3d2f46de0527d0ef9d802b210e08f91c`. Önceki 295 girdisi korunur. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Toplam 761 Java senaryosu yakalandı ve doğrulandı. Birleşik referans kimliği: `d9a2a8b214d5bbfff3c1105d5d02029d37fbb09472a12307f14a428f4ae24a39`. Eski kayıtlar değiştirilmedi.

Mevcut Rust matematik adaptörü bu işlemleri içermediğinden ilgili karşılaştırma başarısız olur. Sonraki adım bu sabit referansa karşı random/AABB Rust davranış dilimidir.
