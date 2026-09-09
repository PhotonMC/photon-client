# P01.d — VoxelShape closestPointTo sözleşmesi

Bu dilim Java 26.2 `VoxelShape.closestPointTo(Vec3)` davranışını kapsar. Empty shape, iç nokta, eksen ve diagonal clamp, OR birleşiminde eşit uzaklık ve negatif koordinatlı kutu için altı özgün senaryo hazırlanmıştır. Rust sonucu [parite kanıtında](shape-closest-parity.md) Java çıktısıyla karşılaştırılmıştır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:184-200` ve `util/Mth.java:106-108` incelendi. `MathOracle` doğrudan shape API'sini çağırır; beklenen noktayı Rust sonucundan üretmez.

Bu dilim yalnızca shape içindeki en yakın noktayı sorgular. Yüz şekilleri, diğer occlusion predicate'leri, shape collider toplama ve diğer merger dalları ayrı dilimlerdir.

## Davranış sözleşmesi

- Empty shape için sonuç `Optional.empty` olur. Gson çıktısında isteğe bağlı `point` alanı bu durumda bulunmaz.
- Non-empty shape kutuları Java'nın `forAllBoxes` sırasıyla taranır. Her kutu için sorgu noktasının üç bileşeni kutunun min/max uçları arasına Java double clamp davranışıyla alınır.
- Sorgu noktası kutunun içindeyse aday nokta sorgunun kendisidir; dışarıdaysa her eksen bağımsız olarak ilgili kutu yüzüne taşınır.
- Adaylar arasından kareli uzaklığı daha küçük olan seçilir. Karşılaştırma kesin küçüktür; eşit uzaklıkta daha önce taranan kutunun adayı korunur.
- Sonuç `Vec3` bileşenleri f64 olarak taşınır. JSON snapshot'ı x, y, z, length ve length_squared bitlerini içerir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| empty | 1 | `1e-7` eşiğinin altındaki sliver ve merkez sorgusu |
| iç nokta | 1 | Birim kutunun merkezi |
| eksen clamp | 1 | Tek eksende kutu dışındaki sorgu |
| diagonal clamp | 1 | Üç eksende kutu dışındaki sorgu |
| OR eşitlik | 1 | Ayrık iki kutunun tam ortası; ilk aday korunur |
| taşınmış kutu | 1 | Negatif ve pozitif koordinatları olan kutu |

Girdiler `family: "math"`, `operation: "shape"` ve `kind: "closest"` veya `kind: "closest_join"` taşır. Shape kutuları ve `point` bileşenleri f64 bit string'leridir. Sonuçta non-empty sorgu için `point`, empty sorgu için boş shape sonucu bulunur.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Bu dilimin altı senaryosunu içeren math referans kimliği `1209dbe51a271188f3d745fe39b8795d0bed606525a2c1744f5d46c696907e5b`'dir. Birleşik altı aile referansı `c121387466c02d22c597319b4f93bfab63c1d5b60513759b3db27ed352523c07` ile doğrulanmıştır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
