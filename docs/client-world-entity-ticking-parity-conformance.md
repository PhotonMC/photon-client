# P08.t.1 — ClientWorld entity ticking eligibility — [client-parity]

Bu kayıt, local Player ve chunk'a bağlı remote entity için world tick uygunluğunu
Java 26.2 ile Rust `ClientWorld` kopyasında karşılaştırır. Kanıt sınıfı
`client-parity`'dir: aynı özgün sentetik zaman çizelgesi Java oracle'ına ve
production Rust adapter'a verilir.

[P08.t server-conformance kaydı](client-world-tick-conformance.md) canlı
vanilla bağlantısında local player tick sayacını gösterir. O kayıt wire ve
entegrasyon kanıtıdır; bu kayıt local/remote ticking state geçişini aynı
girdide Java ile karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `client/multiplayer/ClientLevel.java:450-477`, world tick içinde yalnız
  ticking entity listesindeki uygun kök entity'lerin tick edildiğini ve
  local Player ile remote entity için aynı tick sınırının kullanıldığını
  gösterir.
- `world/entity/Entity.java:247,1816-1835,3955-3972`, `tickCount` ve eski
  transform güncellemesini tanımlar; genel entity'nin `isAlwaysTicking`
  sonucu `false`'tur.
- `world/entity/player/Player.java:1931-1934`, Player override'ının
  `isAlwaysTicking=true` dalını açtığını gösterir. Local Player section
  uygunluğundan bağımsız tick edilir.
- `world/level/entity/TransientEntitySectionManager.java:28-65,95-140`,
  chunk section tracked durumuna göre sıradan entity'nin ticking listesinden
  çıkmasını ve chunk geri geldiğinde yeniden eklenmesini tanımlar.

Oracle fixture bu kaynak sırasını kontrollü state makinesiyle yürütür:
başlangıçta remote entity'nin chunk'ı yüklüdür; unload yalnız remote uygunluğu
değiştirir; reload remote uygunluğu geri getirir; her world tick'inde game time
ve local Player sayacı ilerler.

## Yeni dal ve varyant kararı

Bu dilim, P08.t server-conformance kaydındaki ticking eligibility dalını ilk
kez `client-parity` sınıfında çalıştırır. Farklı gözlem; chunk unload ile
remote entity tick sayacının durması, local Player sayacının devam etmesi,
reload sonrası remote sayacın kaldığı yerden sürmesi ve bu geçişlerin game
time ile sırasıdır. Aynı sonucu veren mevcut `client-parity` kayıtlarının
sayısı **0**'dır; P08.e local-player tick kaydı entity listesinin chunk
uygunluğunu karşılaştırmaz.

Yüklü, unload edilmiş ve yeniden yüklenmiş tek remote chunk tek parametrik
corpus kaydında kapsandı. Sabit veya aynı branch'i kopyalayan yeni entity
varyantı açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776` ve Java `25`'tir. Başlangıç world'ü
  dimension min Y `0` olan, chunk `(0,-3)` içinde boş ve yüklü tek section'dır.
- Local Player ID `42`, `(10,64,-2)` konumunda ve `always_ticking=true` olarak
  başlar. Remote entity ID `37`, type ID `71`, `(1.25,80,-35.5)` konumunda
  başlar; bu konum chunk `(0,-3)` içindedir ve `always_ticking=false`'tur.
- Başlangıçta iki entity'nin de tick uygunluğu true'dur. İlk world tick'i
  game time'ı, local Player sayacını ve remote sayacını bir artırır.
- Remote chunk unload edildiğinde game time ve sayaçlar değişmez; remote
  `ticking=false` olur. Sonraki world tick'i yalnız game time ve local Player
  sayacını artırır.
- Chunk reload edildiğinde remote `ticking=true` olur. Sonraki world tick'i
  remote sayacını kaldığı değerden bir artırır; local Player ve game time da
  birer artar.
- Entity tick gövdeleri, eski transform/interpolation, removed/frozen/passenger
  filtreleri, gerçek section manager callback zamanlaması, network görünürlük,
  physics, render ve oynanabilir istemci bu kaydın dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/entity-ticking-eligibility` kaydı başlangıç, loaded tick,
unload, unloaded tick, reload ve reloaded tick durumlarını taşır. Java oracle
iki çalıştırmada kararlı kaldı; tekrar farkı `null`'dır.

Java referans kimliği:
`69929208bfe8dd62ac7538deecd153ead1908a2e6c987397e7c3c9b4aa53ed15`

Math corpus'u **2999** kayıttır. Ticking state özeti:

| Adım | game time | local Player tick | remote tick | remote ticking |
| --- | ---: | ---: | ---: | --- |
| initial | 0 | 0 | 0 | `true` |
| loaded-first-tick | 1 | 1 | 1 | `true` |
| remote-chunk-unloaded | 1 | 1 | 1 | `false` |
| unloaded-tick | 2 | 2 | 1 | `false` |
| remote-chunk-reloaded | 2 | 2 | 1 | `true` |
| reloaded-tick | 3 | 3 | 2 | `true` |

## Rust doğrulaması

Adapter production `ClientWorld::tick`, `set_local_player_id`, local Player
state binding, loaded chunk replacement ve chunk unload yollarını kullanır.
Remote entity type ID `71` ile gerçek entity map'ine eklenir; state serializer
game time, local Player counter, remote counter, `always_ticking` ve `ticking`
alanlarını sorgular.

Debug adapter SHA-256:
`32249a8c4e700ccabebde8f5cf5707dde47e4970767934a783e2f7d95ef15052`

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=2999
difference=null
status=parity_passed
```

Release adapter SHA-256:
`7a7d72e17430080e7f7f6bee770b91daf1c25ba33ed2d282cb48a8bcd1bf596d`

Release adapter aynı capture üzerinde **2999/2999** kayıt ve
`difference=null` ile `parity_passed` oldu.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/release/examples/behavior_adapter
case_count=2999
difference=null
status=parity_passed
```

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve
Rust adapter'a verilir. Vanilla sunucusuna canlı bağlantı [P08.t
server-conformance kaydında](client-world-tick-conformance.md) tutulur;
sunucu kabulü bu local/remote ticking state karşılaştırmasının yerine geçmez.

## Açık kapsam

Gerçek `ClientLevel.tickEntities` ve `TransientEntitySectionManager` callback
zamanlaması, genel entity tick gövdeleri, eski transform/interpolation,
removed/frozen/passenger filtreleri, network entity tracker görünürlüğü,
physics, collision, dimension/respawn, render ve oynanabilir istemci
bağlantısı açık kalır.
