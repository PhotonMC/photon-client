# P09.g — Entity hareket çarpışma bayrakları — [client-parity] birleşik kayıt

Bu kayıt, çözülen tek bir entity hareket delta'sından Java'nın temel çarpışma
bayraklarını üretir. Kanıt sınıfı `client-parity`'dir: aynı sentetik hareket ve
shape girdileri Java 26.2 oracle'ında ve Rust adapter'ında çalıştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/entity/Entity.java:712-768`, `move` önce `collide` ile hareketi çözer;
  sonra X/Z için `Mth.equal`, Y için doğrudan karşılaştırma kullanarak
  çarpışma bayraklarını günceller.
- `world/entity/Entity.java:762-767`, yatay bayrağın X veya Z çarpışmasının
  birleşimi olduğunu, aşağı yönlü dikey çarpışmanın `onGround` sonucunu
  oluşturduğunu gösterir.
- `world/entity/Entity.java:1196-1207`, çözülen hareketin boş veya toplanmış
  shape listesi üzerinde eksen sırasıyla üretildiği temel primitive'i taşır.
- `util/Mth.java:158-164`, X/Z karşılaştırmasındaki eşik değerini kesin olarak
  `abs(b - a) < 1.0E-5` olarak tanımlar.

## Sözleşme

- Önce `Entity.collideWithShapes` sonucu alınır; hareket vektörü bu sonuçtur.
- X ve Z çarpışma bayrakları, istenen ve çözülen bileşenler arasındaki Java
  `Mth.equal(double, double)` karşılaştırmasının olumsuzudur.
- Y çarpışması doğrudan `movement.y != resolved.y` ile hesaplanır.
- `horizontalCollision`, X veya Z bayrağının OR birleşimidir.
- `verticalCollisionBelow`, dikey çarpışma ve istenen Y hareketinin negatif
  olmasıdır. Bu temel hareket diliminde `onGround` aynı sonuçtur.
- Bu kayıt step-up seçimini, LocalPlayer'ın minor-horizontal override'ını ve
  hareket delta'sı sıfırken mevcut state alanlarını koruma kapısını kapsamaz.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`shape/entity-movement-state/base-flags` girdisi altı sentetik satır taşır:
boş collider, X yönü, zemine düşme, tavana yükselme ve birleşik yatay/dikey
çarpışma. Shape listesi doğrudan Java `VoxelShape` kutularından kurulur.

Java oracle, `Entity.collideWithShapes(Vec3, AABB, List)` metodunu reflection
ile çağırır ve hemen ardından `Entity.move` içindeki bayrak hesap sırasını
uygular. Beklenen hareket vektörü ve tüm temel bayraklar Java çıktısından
üretilir.

## Rust doğrulaması

`src/world.rs` içindeki `collide_entity_movement_state`, mevcut Java uyumlu
hareket çözümünü kullanır; X/Z için `equal_f64`, Y için doğrudan karşılaştırma
uygular ve yatay/aşağı dikey/on-ground birleşimlerini döndürür. Adapter aynı
production helper'ı kullanır; beklenen sonuç Rust çıktısından türetilmez.

Java referans kimliği:
`60153852b65d823a8ef187f50938af367a54e8a76448ec97fa6b59e27dc14084`.

Math corpus'undaki 2916 kayıt, Java'nın iki ardışık capture çıktısında kararlı
kaldı. Java `verify` başarılı oldu; Rust adapter SHA-256 değeri
`16e10e03b8ed59852ce5e00ab3c1431bff8b64e4715d6d9931cb754a51c4afdd` ile
karşılaştırma **2916/2916** eşleşti.

```text
cargo fmt --all -- --check
cargo test --locked --offline --all-targets --quiet
cargo check --locked --offline --all-targets
cargo clippy --locked --offline --all-targets -- -D warnings
cargo build --locked --offline --examples
python3 tools/behavior/run.py verify --family math
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

## Açık kapsam

Step-up adayları arasından daha iyi yatay hareketin seçilmesi, final hareketten
sonra gerçek `ClientEntity` flag state'inin güncellenmesi, zero-Y hareketinde
Java'nın state retention kapısı, Player minor-horizontal hesabı, entity ve
world-border collider'ları, block-state context'i, pose dimensions yenilemesi
ve scale dalları sonraki P09 dilimleridir.

Bu kayıt canlı sunucu kabulü değildir; Java istemcisiyle sentetik hareket
sonuçlarının karşılaştırılmasıdır.
