# P01.c — Direction referans sözleşmesi

Bu dilimin **18 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](math-parity.md)). Önceki 220 math senaryosunun girdisi ve sonucu değişmedi. Yön bulma (entity bakışı), kuaterniyon, codec ve kalan Direction yardımcıları bu dilime dahil değildir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `core/Direction.java` içindeki ad, veri değerleri, eksen, adım, karşıt yön ve eksenli saat yönü/saat tersi döndürmeler incelendi. Adaptör doğrudan orijinal enum'u çağırır; oyun algoritması adaptörde yeniden yazılmaz.

`Direction.<clinit>` için math ailesine `netty-codec-base`, `commons-lang3`, `authlib` ve `jspecify` gerekir. Yerel JAR SHA-1 değerleri `tests/behavior/reference.json` içinde sabitlendi. Yeni oyun bağımlılığı veya Rust kütüphanesi eklenmedi.

## Davranış sözleşmesi

- 6 yön vardır: `down`, `up`, `north`, `south`, `west`, `east`. Veri değerleri, eksenler ve adım vektörleri sabitlenir; `north` Z-eksende -1 adımdır.
- `getOpposite` karşıt yönü verir; eksen ve adım tutarlılığı ayrıca gözlenir.
- Eksenli döndürme kendi eksenine hizalı yönde birimdir (ör. X etrafında `west`/`east` değişmez). Yatay Y döndürmesi `north→east→south→west` sırasını izler.
- Her senaryo başlangıç ve her adım sonrası `name`, `data3d`, `data2d`, `opposite`, `axis`, `axis_direction` ve `step_x`/`step_y`/`step_z` kaydeder. Bilinmeyen ad/eksen girdisi testi geçersiz kılar; hata sonucu üretilmez.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Özellikler | 6 | 6 yönün adı, verileri, karşıtı, ekseni ve adımı |
| Y-döndürme | 6 | Her yönden iki saat yönü Y adımı |
| X/Z-döndürme | 6 | Seçili yön-eksen çiftlerinde tek saat yönü adımı |

`family: "math"`, `operation: "direction"` ortak girdilerdir. `name` küçük harfli yön adıdır. `steps` içindeki her adım `axis` (`x`/`y`/`z`) ve `clockwise` alanı taşır.

## Çalıştırma ve kanıt

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

238 math senaryosu iki ayrı JVM'de aynı sonucu verdi. Math referans kimliği: `07b321ff382a45bdabd9e3a26e16d6e436ce85e6e064d33e00d3f1c1257f1b97`. Önceki 220 girdisi korunur. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Yeni aileyle toplam 637 Java senaryosu yakalandı ve doğrulandı. Birleşik referans kimliği: `c34a0c2bb442fe7576fbe4de19455d484cfe2aeb0eabe5ac96ca4a37aa128494`. Eski kayıtlar değiştirilmedi.

Beklenen çıktılar Rust'a göre değiştirilmedi.
