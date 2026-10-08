# P09.i — Statik world-border collision shape — [client-parity] birleşik kayıt

Bu kayıt, Java 26.2 `WorldBorder` içindeki statik extent'in collision shape'ini
merkez, boyut ve absolute-max sınırı üzerinden üretir. Kanıt sınıfı
`client-parity`'dir: aynı dört parametrik durum Java oracle'ında ve Rust
adapter'ında çalıştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/level/border/WorldBorder.java:100-101`, public collision shape sorgusu
  aktif extent'in shape'ini döndürür.
- `world/level/border/WorldBorder.java:163-193,216-219`, merkez ve boyut
  güncellemeleri extent'i yeniler; `setAbsoluteMaxSize` mevcut extent'i
  yeniden hesaplatır.
- `world/level/border/WorldBorder.java:498-509,556-583`, statik extent merkezden
  yarım boyutu çıkarıp ekler, her yatay ucu `absoluteMaxSize` ile clamp eder ve
  değişimlerde kutuyu yeniden kurar.
- `world/level/border/WorldBorder.java:561-572`, iç kutu
  `floor(minX/minZ)`, `ceil(maxX/maxZ)` ve sınırsız Y aralığıyla oluşturulur;
  collision shape `Shapes.INFINITY ONLY_FIRST inner` birleşimidir.
- `util/Mth.java:102-108` double clamp yolunu, `world/phys/shapes/Shapes.java:52-96,135-173`
  ise box ve BooleanOp join yollarını tanımlar.

## Sözleşme

- Statik extent'in yatay min/max değerleri `center ± size / 2.0` üzerinden
  hesaplanır ve `[-absoluteMaxSize, absoluteMaxSize]` aralığına clamp edilir.
- İç collision kutusunun yatay min uçları `floor`, max uçları `ceil` ile
  dönüştürülür. Y aralığı `Double.NEGATIVE_INFINITY` ile
  `Double.POSITIVE_INFINITY` arasındadır.
- Sonuç, sınırsız shape'den iç kutuyu `BooleanOp.ONLY_FIRST` ile çıkaran
  dört dış kutulu shape'tir; non-empty snapshot sınırsız bounds taşır.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik kayıt dört durum taşır:

- merkezde fractional boyut ve floor/ceil dönüşümü;
- fractional merkezde dört yatay uç dönüşümü;
- merkez ve boyutun absolute-max sınırına taşması;
- absolute-max sınırını aşan büyük border.

Java oracle gerçek `WorldBorder.setCenter`, `setAbsoluteMaxSize`, `setSize` ve
`getCollisionShape` çağrılarını kullanır. Beklenen shape kutuları Rust çıktısından
türetilmez.

## Rust doğrulaması

`src/world.rs` içindeki `world_border_collision_shape`, Java statik extent
hesabını ve `Shapes::join(Shapes::infinity(), inner, OnlyFirst)` yolunu uygular.
`crates/photon-core/src/shape.rs` sonsuz coordinate plane'leri join ederken hücre örnekleme için
sonlu temsil noktaları seçer; bu, sınırsız Y ve yatay dış aralık hücrelerinin
boş görünmesini engeller. Adapter aynı shape snapshot'ını döndürür.

Java referans kimliği:
`c5a41c031295236be2fbeec774daea57351f249f3e5a0589a717e8cef46bbb36`.

Math corpus'undaki 2920 kayıt, Java'nın iki ardışık capture çıktısında kararlı
kaldı. Java `verify` başarılı oldu. Rust debug adapter SHA-256 değeri
`bd32fc010890641abb3fc37e0b8c272560e8cec46a5d926a72fd0e1f4444b3cc`, release
adapter SHA-256 değeri
`947c1a696fcb9f4c73b943f3d4af2ee82442415b8b40aa8642fc0d6bc7307fef` ile iki
karşılaştırma da **2920/2920** eşleşti.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

## Açık kapsam

`MovingBorderExtent` boyut interpolasyonu, tick/partial-tick güncellemesi,
gerçek `ClientWorld` border ayarlarının uygulanması, block/entity/world-border
shape'lerinin tek `noCollision` sorgusunda birleştirilmesi ve border distance,
warning, damage, teleport ve player movement etkileri sonraki P09/P10
dilimleridir. Bu kayıt canlı sunucu kabulü değildir; vanilla sunucuda yapılan
başarılı bağlantı ve state probe'ları `server-conformance` kanıtıdır.
