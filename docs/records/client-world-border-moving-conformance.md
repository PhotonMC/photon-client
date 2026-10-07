# P09.j — Moving world-border extent — [client-parity] birleşik kayıt

Bu kayıt, Java 26.2 `WorldBorder` içindeki hareketli extent'in boyut
interpolasyonunu, edge sorgularını ve collision shape geçişini taşır. Kanıt
sınıfı `client-parity`'dir: aynı parametrik büyüme, küçülme ve clamp girdileri
Java oracle'ında ve Rust adapter'ında çalıştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/level/border/WorldBorder.java:195-201`, `lerpSizeBetween` eşit başlangıç
  ve hedefte statik extent, farklı boyutlarda `MovingBorderExtent` kurar.
- `world/level/border/WorldBorder.java:330-400`, hareketli extent başlangıç ve
  hedef boyutu, duration, kalan progress, current/previous size alanlarını
  tutar; `calculateSize` tamamlanmamış ilerlemeyi `Mth.lerp` ile hesaplar.
- `world/level/border/WorldBorder.java:352-386`, yatay edge sorguları
  `partialTick` ile previous/current size arasında lerp yapıp sonucu
  `absoluteMaxSize` ile clamp eder.
- `world/level/border/WorldBorder.java:402-420`, lerp süresi, hedef ve
  büyüme/küçülme status metadata'sını döndürür.
- `world/level/border/WorldBorder.java:281-283,431-452`, her tick previous/current
  state'i ilerletir; süre bitince statik extent'e döner. Hareketli collision
  shape'i edge'leri `0.0F` partial tick ile kurduğu için tick sırasında previous
  size'ı kullanır.
- `util/Mth.java:102-108,550-560`, double clamp ve double lerp yollarını
  tanımlar; f32 partial tick double lerp çağrısına genişler.

## Sözleşme

- Farklı `from` ve `to` boyutları hareketli extent oluşturur; başlangıçta
  current ve previous size `from`, lerp time duration ve target `to` olur.
- Her tick'te kalan süre bir azalır, önceki current size previous state olur ve
  yeni current size `(duration - progress) / duration` oranıyla Java double
  `Mth.lerp` yolundan hesaplanır. Progress sıfıra ulaştığında extent statik
  hedef boyuta geçer.
- Edge sorguları `partialTick` değerini önceki ve güncel boyut arasında lerp
  eder; merkez ± yarım boyut sonucu `[-absoluteMaxSize, absoluteMaxSize]`
  aralığına clamp edilir.
- Hareketli extent'in collision shape'i `partialTick = 0.0F` ile sorgulanan
  edge'lerden üretilir. Bu nedenle hareket sürerken collision shape previous
  size'ı, tamamlandıktan sonra hedef boyutu kullanır.
- Hedef küçükse status `SHRINKING`, aksi durumda `GROWING`; statik extent
  `STATIONARY` ve lerp time `0` döndürür.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik kayıt üç durum taşır:

- absolute-max sınırına taşmadan büyüyen border;
- küçülen ve edge uçları absolute-max ile clamp edilen border;
- merkez ve hedef boyut nedeniyle clamp edilen büyüyen border.

Her durum birden fazla f32 partial tick ve tick sonrası son statik state ile
izlenir. Java oracle gerçek `WorldBorder.setCenter`, `setAbsoluteMaxSize`,
`lerpSizeBetween`, `getMinX`/`getMaxX`/`getMinZ`/`getMaxZ`, `getSize`,
`getLerpTime`, `getLerpTarget`, `getStatus`, `getCollisionShape` ve `tick`
çağrılarını kullanır. Beklenen sonuç Rust çıktısından türetilmez.

## Rust doğrulaması

`src/world.rs` içindeki `ClientWorldBorder`, static/moving extent state'ini,
current/previous size geçişini, f32 partial tick genişletmesini, edge clamp'ini
ve tick sonunda statikleşmeyi modeller. Collision shape, P09.i'de doğrulanan
`world_border_collision_shape` yoluna bağlanır. Adapter aynı state ve shape
snapshot'ını taşır.

Java referans kimliği:
`c687eac09cb66a4c9b9bf0e2f63218989881a3497b46a111ae7a1c2d67f52f87`.

Math corpus'undaki 2921 kayıt, Java'nın iki ardışık capture çıktısında kararlı
kaldı. Java `verify` başarılı oldu. Rust debug adapter SHA-256 değeri
`8be568995432e85fdf46d4a2c57bdeed31c94a69a897884e42c79881ef532503`, release
adapter SHA-256 değeri
`a64d4e52a370582678d8e871fceddbd26c1423d644696830a7760fa0747e76d7` ile iki
karşılaştırma da **2921/2921** eşleşti.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

## Açık kapsam

Hareketli extent artık ayrı bir Java/Rust state modeli olarak doğrulandı.
Gerçek `ClientWorld` border packet/state uygulaması, block/entity/world-border
shape'lerinin tek `noCollision` sorgusunda birleşmesi ve border distance,
warning, damage, teleport ile player movement etkileri sonraki P09/P10
dilimleridir. Bu kayıt canlı sunucu kabulü değildir.
