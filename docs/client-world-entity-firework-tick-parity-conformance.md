# P08.bp.1 — ClientWorld Firework Rocket free-flight tick — [client-parity]

Bu kayıt, yüklü ve boş bir world section içindeki normal Firework Rocket
entity'sinin iki ardışık `ClientWorld` tick'inde Java 26.2 ile Rust world
kopyasında aynı konum, hareket, rotation ve tick state'ini üretmesini kapsar.
Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve
production Rust adapter'a verilir.

[P08.bp server-conformance kaydı](client-world-entity-firework-tick-conformance.md)
vanilla 26.2 dedicated server'dan alınan Firework Rocket AddEntity state'inin
Rust world tick'inde kabul edilmesini gösterir. O kayıt canlı sunucu wire ve
kabul kanıtıdır; bu kayıt Java istemci davranışını aynı girdide karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `world/entity/EntityTypes.java:460-467`, Firework Rocket entity type ID'sini
  `53` olarak kaydeder. `client/multiplayer/ClientLevel.java:467-472`, uygun
  entity için eski transform'u kopyalar, `tickCount` değerini artırır ve
  entity tick gövdesini yürütür.
- `world/entity/projectile/FireworkRocketEntity.java:114-162`, bağlı olmayan
  ve `shotAtAngle=false` dalında mevcut hareketin X/Z bileşenlerini `1.15` ile
  çarpar, Y'ye `0.04` ekler, hit test sonrasında konum alır ve hareketi korur.
  Bu dalda FallingBlock ve Primed TNT'deki gibi air drag veya gravity yoktur.
- `world/entity/projectile/FireworkRocketEntity.java:164-186`, hit, rotation,
  launch sound, life ve particle/lifecycle yan etkilerini; bu kayıt ise yalnız
  MISS free-flight state'ini izole eder.
- `world/entity/projectile/Projectile.java:329-346`, hareketten pitch/yaw
  hedefi üretilmesini ve eski rotation'a `0.2F` ile yaklaşan wrapped
  `lerpRotation` döngüsünü tanımlar. Pinned Java bytecode'unda dönüşüm
  `57.2957763671875` derece/radyan çarpanı olarak gerçekleşir.

Firework Rocket'un ayırt edici Java dalı, gravity veya drag kullanmadan
`horizontalAcceleration → verticalAcceleration → position → rotation`
sırasını yürütmesidir. Oracle, `FireworkRocketEntity` tabanlı özgün fixture ile
gerekli inherited `updateRotation` yolunu çağırır; bağlı hedef, hit/deflection,
particle, sound, item ve lifecycle yan etkilerini kontrollü olarak dışarıda
bırakır.

## Yeni dal ve varyant kararı

Bu dilim, P08.bp server-conformance kaydındaki Firework Rocket client tick
dalını ilk kez `client-parity` sınıfında çalıştırır. Farklı gözlem; mevcut
movement'in X/Z bileşenlerinin `1.15` ile hızlandırılması, Y'ye `0.04`
eklenmesi, hızlandırılmış movement ile position alınması ve Java'nın wrapped
projectile rotation yaklaşımının uygulanmasıdır. Aynı sonucu veren mevcut
`client-parity` kayıtlarının sayısı **0**'dır; Snowball, Arrow, Experience Orb
ve Primed TNT ayrı entity tick gövdelerini ölçer.

Normal, bağlı olmayan ve `shotAtAngle=false` dalı tek parametrik corpus
kaydında iki ardışık tick ile kapsandı. Başlangıç yaw'ı `355°` seçilerek aynı
dalın 180° sınırını aşan `Projectile.lerpRotation` davranışı da ayrı varyant
oluşturmadan ölçüldü.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25` ve Firework Rocket
  entity type ID `53`'tür. Başlangıç world'ü dimension min Y `0` olan,
  chunk `(0,0)` içinde boş ve yüklü tek section'dır.
- Entity `37`, `(2,10,2)` konumunda `(0.25,0.5,0.125)` hareketi ve
  `(y=355,x=-20)` rotation'ı ile başlar. Entity bağlı değildir, horizontal
  collision yoktur, `shotAtAngle=false`'dır ve hit sonucu MISS'tir.
- Her world tick'inde ClientLevel eski position/rotation state'ini saklar,
  `tickCount` değerini artırır. X/Z movement bileşenleri `1.15` ile çarpılır;
  Y movement'ine `0.04` eklenir.
- Entity hızlandırılmış movement ile konum alır ve aynı movement'i korur.
  Rotation hedefi movement'ten Java `atan2` ve
  `Projectile.lerpRotation` wrapped lerp sırasıyla üretilir; Z rotation
  değişmez.
- Bağlı entity/fall-flying, `shotAtAngle=true`, hit/deflection, block/entity
  collision, fluid/block effects, life/lifetime, particle, launch sound,
  explosion/removal, firework item/component metadata, gerçek packet/thread
  dispatch'i, renderer ve oynanabilir istemci bu kaydın dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/entity-firework-tick` kaydı bir Firework Rocket satırını ve iki
tick işlemini taşır. Java oracle iki çalıştırmada kararlı kaldı; tekrar farkı
`null`'dır.

Java referans kimliği:
`d089d3e73bba57e843e782784c2a5683d0b29846185d95cc7c24cd9ff498db36`

Math corpus'u **2996** kayıttır. Yeni kaydın seçili raw state bitleri:

| Adım | Entity | position X/Y/Z | motion X/Y/Z | rotation Y | rotation X | tick |
| --- | ---: | --- | --- | --- | --- | ---: |
| initial | 37 | `4000000000000000/4024000000000000/4000000000000000` | `3fd0000000000000/3fe0000000000000/3fc0000000000000` | `43b18000` | `c1a00000` | 0 |
| first-air-tick | 37 | `40024ccccccccccd/4025147ae147ae14/4001266666666666` | `3fd2666666666666/3fe147ae147ae148/3fc2666666666666` | `410afe10` | `c084e27a` | 1 |
| second-air-tick | 37 | `4004f1eb851eb852/40263d70a3d70a3d/400278f5c28f5c28` | `3fd528f5c28f5c28/3fe28f5c28f5c290/3fc528f5c28f5c28` | `419d17d3` | `4102d01d` | 2 |

## Rust doğrulaması

`ClientWorld::tick` production entity tick yolunu kullanır. Yüklü air-cell
kontrolü type ID `53` için geçildiğinde `ClientEntity::tick_firework`,
horizontal acceleration → vertical acceleration → position → rotation sırasını
uygular ve movement'i korur. Rotation hesabı Java'nın derlenmiş derece/radyan
çarpanı ile ortak `Projectile.lerpRotation` helper'ını kullanır; özellikle
`355°` wrapped başlangıç state'i Java ile aynı raw bitlerde üretilir.

Debug adapter SHA-256:
`5b4adf4d08e880b41517846c544e2b0ecc9f05ede3e0585d07743a85d7912058`

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=2996
difference=null
status=parity_passed
```

Release adapter SHA-256:
`08b5d129efe369b790708c49c40571d8a2c303f047bed0e64b26defb6add998f`

Release adapter aynı capture üzerinde **2996/2996** kayıt ve
`difference=null` ile `parity_passed` oldu.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/release/examples/behavior_adapter
case_count=2996
difference=null
status=parity_passed
```

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci sınıflarını
çalıştıran oracle'a ve Rust adapter'a verilir. Vanilla sunucusu üzerindeki
Firework Rocket AddEntity ve wire kabul sonucu [P08.bp
server-conformance kaydında](client-world-entity-firework-tick-conformance.md)
tutulur; canlı sunucu kabulü bu Java istemci state karşılaştırmasının yerine
geçmez.

## Açık kapsam

Firework Rocket item/component ve `shotAtAngle` metadata'sı, attached target ve
elytra boost dalı, collision/hit/deflection, life/lifetime, particle/sound/
explosion lifecycle'ı, removal, network entity dispatch, renderer, genel
motion integration ve oynanabilir istemci bağlantısı açık kalır.
