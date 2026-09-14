# P09.l — ClientWorld birleşik collision sources — [client-parity] birleşik kayıt

Bu kayıt, Java `CollisionGetter.noCollision` sorgusunun block, entity ve
world-border kaynaklarını aynı çağrıda ve aynı kısa devre sırasıyla birleştirmesini
taşır. Kanıt sınıfı `client-parity`'dir: aynı yüklenmiş block fixture'ı, entity
adayları, border state'i ve query AABB Java oracle'ında ve Rust `ClientWorld`
adapter'ında çalıştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/level/CollisionGetter.java:39-88`, `noCollision` sırasıyla
  `noBlockCollision`, `noEntityCollision` ve `noBorderCollision` çağrılarını
  `&&` kısa devresiyle birleştirir. Block yolu collision iterable'ının boş
  olmasını, entity yolu entity shape listesinin boş olmasını bekler; border yolu
  null source için doğrudan true döner.
- `world/level/CollisionGetter.java:82-88`, `getCollisions` entity ve block
  shape'lerini birleştirir; world-border shape'i ayrı `noBorderCollision`
  kapısından sorgulanır.
- `world/level/EntityGetter.java:56-75`, entity shape listesinde küçük query
  alanını erken eler, query'yi `1.0E-7` genişletir ve source/spectator/
  collidable/aynı vehicle filtrelerini uygular.
- `world/level/border/WorldBorder.java:96-117`, entity konumunun border'a
  yakınlığını query AABB'sinin yatay boyutlarından türetilen en az `1.0` margin,
  strict distance eşiği ve margin'li bounds kontrolüyle sınar. Yakınsa border
  collision shape'i ile query shape'inin `AND` kesişimi collision sonucunu verir.

## Sözleşme

- `noCollision(source, query)` önce block, sonra entity, sonra border kaynağını
  sorgular; ilk false sonucu sonraki kaynağı çalıştırmadan false döndürür.
- Block sonucu yüklenmiş `ClientWorld` section state ID'lerinden toplanan
  collision shape'lerinin boşluğudur. Entity sonucu P09.h'deki epsilon broad-
  phase ve filtre sırasını korur.
- Source yoksa border sonucu true'dur. Source varsa border margin'i source
  kutusundan değil Java çağrısına verilen query AABB'sinin X/Z boyutlarından
  hesaplanır.
- Source konumu border'a margin'in iki katından daha yakın değilse veya margin'li
  border bounds dışındaysa border shape'i sorgulanmaz. Yakınlık kapısı geçerse
  sınırsız Y'li border shape ile query shape'in kesişimi boş olmalıdır.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik kayıt beş gözlem taşır:

- boş query ve null source;
- block shape tarafından reddedilen query;
- entity adayı tarafından reddedilen query;
- border shape tarafından reddedilen yakın query;
- border'ın margin'li bounds kapısından uzakta kalan source.

Java oracle gerçek `CollisionGetter.noBlockCollision`,
`noEntityCollision`, `noBorderCollision` ve `noCollision` default yollarını
çalıştırır. Rust adapter aynı block section, entity candidate ve
`ClientWorldBorder` state'ini gerçek production yordamlarına verir. Beklenen
sonuç Rust çıktısından türetilmez.

## Rust doğrulaması

`ClientWorld` artık ayrı block, entity ve border sorgularını ve bunların Java
sıralı birleşimini üretim API'sinde taşır. `ClientWorldCollisionSource` yalnızca
border yakınlık kapısının ihtiyaç duyduğu source konumunu taşır; border state'i
P09.k'deki packet uygulama yolundan okunur.

Java referans kimliği:
`d1e44ac39bd75e5af1907a213966e38295624e0882aedca48b759660337c5cec`.

Math corpus'undaki 2923 kayıt, Java'nın iki ardışık capture çıktısında kararlı
kaldı. Java `verify` başarılı oldu. Rust debug adapter SHA-256 değeri
`f169410e3cc5308a0994c27d1af652c9dd86f1c317090835329cbf16ed9443f5`, release
adapter SHA-256 değeri `92271409cabea1297c0f1e9db1d6d88229ebcff7045fa7ee91d29651ed75380a`
ile iki karşılaştırma da **2923/2923** eşleşti.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
cargo fmt --all -- --check
cargo test --locked --offline
cargo check --locked --offline --all-targets
cargo build --locked --offline --examples
cargo clippy --locked --offline --all-targets -- -D warnings
```

## Açık kapsam

Block/entity/world-border birleşiminin sorgu sırası ve border yakınlık kapısı
synthetic state üzerinden `client-parity` olarak doğrulandı. Gerçek block-state
registry/context çözümü, canlı `ClientWorld` entity index/state'i, bu state'lerin
aynı sorguya bağlanması, `refreshDimensions`, movement/step-up ve border'ın
damage/teleport/render etkileri sonraki P09/P10/P14 dilimleridir. Bu kayıt canlı
sunucu kabulü değildir.
