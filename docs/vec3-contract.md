# P01.b — Vec3 referans sözleşmesi

Bu dilimin **40 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](math-parity.md)). Dot/cross/add/subtract ve diğer Vec3 yardımcıları bu dilime dahil değildir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `world/phys/Vec3.java` içindeki `normalize`, `scale`, `length` ve `lengthSqr` incelendi. Adaptör doğrudan orijinal sınıfı çağırır; oyun algoritması adaptörde yeniden yazılmaz.

`Vec3.<clinit>` Netty buffer istediğinden math ailesine `netty-buffer`/`netty-common` eklendi. Yerel JAR SHA-1 değerleri `tests/behavior/reference.json` içinde sabitlendi. Yeni oyun bağımlılığı veya Rust kütüphanesi eklenmedi.

## Davranış sözleşmesi

- Bileşenler f64'tür; ham IEEE-754 bitleriyle karşılaştırılır, tolerans yoktur.
- `lengthSqr` bileşen karelerinin double toplamıdır; `length` bunun kareköküdür.
- `normalize` önce `length` hesaplar; sonuç `1.0E-5F` değerinden küçükse sıfır vektör döner, yoksa her bileşen uzunluğa ayrı bölünür. Eşik float literalin double genişlemesidir.
- `scale` her bileşeni aynı çarpanla çarpar.
- Her senaryo başlangıç ve her adım sonrası `x`, `y`, `z`, `length`, `length_squared` bitlerini kaydeder.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Vec3 normalize | 34 | 27 diyagonal/eksen yönü, işaretli sıfır, subnormal, en büyük sonlu, karışık bileşen ve 1e-5 eşiğinin bit komşuları |
| Vec3 zincir | 6 | Normalize → ölçek → normalize; sıfır/negatif/pozitif çarpanlar |

`family: "math"`, `operation: "vec3"` ortak girdilerdir. `x`, `y`, `z` ve `factor` 16 haneli küçük harfli f64 bit stringleridir. `steps` içindeki `normalize` ve `factor` alanlı `scale` çağrıları sırayla uygulanır.

## Çalıştırma ve kanıt

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

220 math senaryosu iki ayrı JVM'de aynı sonucu verdi. Math referans kimliği: `9d468b5b5edbbe267fb1a4133bfff61c3e528f274f55cb9decd6738beaca43ed`. Önceki 180 girdisi korunur. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Yeni aileyle toplam 619 Java senaryosu yakalandı ve doğrulandı. Birleşik referans kimliği: `8ca11f8017cf6a605669c6437c7df292092304e47e7b567095dc48b196588b64`. Eski kayıtlar değiştirilmedi.

Mevcut Rust matematik adaptörü Vec3 içerir. Beklenen çıktılar Rust'a göre değiştirilmedi.
