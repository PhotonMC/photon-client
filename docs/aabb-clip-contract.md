# P01.d — AABB segment clipping referans sözleşmesi

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-clip-parity.md)). Kapsam, `AABB.clip(Vec3, Vec3)` ile tek bir kutu ve doğru parçası arasındaki ilk kesişim noktasını içerir. Çoklu shape ve `BlockHitResult` yolları bu dilime dahil değildir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `world/phys/AABB.java` içindeki public `clip` ve onu besleyen eksen kesme dalları incelendi. Adaptör doğrudan orijinal metodu çağırır. Yeni bağımlılık eklenmedi.

## Davranış sözleşmesi

- Her eksen delta'sı `1e-7` eşiğiyle değerlendirilir; eksenler x, y, z sırasındadır.
- Aday ölçek `0 < s < mevcut ölçek` ve diğer iki koordinat için `1e-7` genişletilmiş kutu sınırları içindeyse kabul edilir. En küçük kabul edilen ölçek tutulur.
- Kesişim yoksa `hit: false`, varsa `from + s * (to - from)` f64 bitleriyle döner. Segment başlangıcında veya bitişinde bulunan nokta public `clip` yolunda aday sayılmaz.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| AABB clip | 6 | İki yönlü geçiş, kutu içinden çıkış, köşe geçişi, paralel kaçırma ve eşik altı delta |

`family: "math"`, `operation: "aabb"` ortak girdilerdir. `steps` içindeki `op: "clip"` adımı `from` ve `to` f64 bit nesnelerini taşır; sonuç `hit` ve başarılı durumda `point` alanlarını içerir.

## Çalıştırma ve kanıt

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

323 math senaryosu iki ayrı JVM'de aynı sonucu verdi. Math referans kimliği: `b04328d3820db8458e37a31e45b96c1c92ba29348f35c055b49ac522f339a43e`. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Altı ailenin 815 senaryolu birleşik referansı iki JVM'de doğrulandı. Birleşik referans kimliği: `49951d8be4544e46974e9f2cf546651f4d0f7ef7a6fbc7ebf9dd5be799d9c402`.

Math debug ve release Rust adaptörleri **323/323** eşleşti; birleşik debug ve release karşılaştırmaları da **815/815** geçti. Yerel raporlar `compare-71jdlsjf` (math debug), `compare-owaw1c7e` (math release), `compare-4qr5yaro` (birleşik debug) ve `compare-2w5gzlh_` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

Bu doğrulama tek kutulu public `clip` yoluyla sınırlıdır. Çoklu AABB shape birleştirme, yüz yönü ve istemci hedefleme davranışları ayrı eşitlik dilimleridir.
