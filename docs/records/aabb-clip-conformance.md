# P01.d — AABB segment clipping referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `AABB.clip(Vec3, Vec3)` ile tek bir kutu ve doğru parçası arasındaki ilk kesişim noktasını içerir. Koordinat endpoint'li statik overload [ayrı dilimdedir](aabb-clip-coordinates-conformance.md); çoklu shape ve `BlockHitResult` yolları bu dilime dahil değildir.

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

Altı ailenin 2801 senaryolu birleşik referansı iki JVM'de doğrulandı. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Math debug ve release Rust adaptörleri **323/323** eşleşti; birleşik debug ve release karşılaştırmaları da **2801/2801** geçti. Yerel raporlar `compare-71jdlsjf` (math debug), `compare-owaw1c7e` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

Bu doğrulama tek kutulu `AABB.clip(Vec3, Vec3)` yoluyla sınırlıdır. Koordinat endpoint'li overload [ayrı kanıttadır](aabb-clip-coordinates-conformance.md); çoklu AABB shape birleştirme, yüz yönü ve istemci hedefleme davranışları ayrı eşitlik dilimleridir.


## Client-parity doğrulaması

`crates/photon-core/src/aabb.rs`, Java `AABB.clip(Vec3, Vec3)` yolunun eksen sırasını, eşiklerini, epsilon'lı yüz kontrollerini ve ilk geçerli segment ölçeğini uygular. Koordinat endpoint'li statik overload [ayrı dilimde](aabb-clip-coordinates-conformance.md) uygulanır. `examples/behavior_adapter.rs` sonucu f64 bitleriyle JSONL senaryolarına bağlar; beklenen değer üretmez.

## Referans ve davranış

Referans Java 26.2 / protokol 776; `AABB.java` içindeki public clip ve private eksen kesme yardımcıları incelendi. Math oracle doğrudan sabitlenen Java JAR sınıflarını çağırır. Ayrıntılı sözleşme ve 6 senaryoluk kapsam [referans belgesindedir](#sözleşme).

| Rust API | Korunan davranış |
| --- | --- |
| `Aabb::clip` | `Vec3` doğru parçasının kutuya ilk geçerli kesişim noktasını veya yok sonucunu döndürür |
| `clip_point` | `1e-7` delta eşiği, `0 < s < scale` koşulu ve diğer eksenlerde `1e-7` toleransı |

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 323 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **323/323** eşleşti: önceki 317 math senaryosu ve 6 yeni AABB clip senaryosu.
- Altı ailenin birleşik corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `b04328d3820db8458e37a31e45b96c1c92ba29348f35c055b49ac522f339a43e`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-71jdlsjf` (math debug), `compare-owaw1c7e` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt 6 yeni tek kutu `AABB.clip(Vec3, Vec3)` senaryosuyla sınırlıdır. Koordinat endpoint'li overload [ayrı kanıttadır](aabb-clip-coordinates-conformance.md); VoxelShape clip tamamlandı; occlusion ve istemci hedefleme davranışları sonraki dilimlerdir.
