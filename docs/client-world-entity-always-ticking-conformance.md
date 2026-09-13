# P08.y — [server-conformance] Player entity always-ticking state'i

Bu dosya, vanilla client world entity storage'ındaki oyuncu entity'sinin chunk
tick görünürlüğünden bağımsız tick uygunluğunu kaydeder. Canlı kanıt sınıfı
`server-conformance`'dır: iki Rust probe oturumu Mojang vanilla 26.2 dedicated
server'ına bağlanır ve birbirini oyuncu entity'si olarak görür. Java client ile
aynı girdinin karşılaştırılması yapılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`Entity.java:3970-3972`, genel entity'lerde `isAlwaysTicking()` sonucunu
`false` döndürür. `Player.java:1932-1934` bu metodu `true` ile override eder.
`TransientEntitySectionManager.java:28-45,62,83-121,131-137`, chunk status'ı
tracked olduğunda sıradan entity'lerin ticking listesinden çıkarılmasını,
oyuncuların ise `isAlwaysTicking()` nedeniyle listede kalmasını sağlar.
`ClientLevel.java:450-483`, tick listesinde olan oyuncu entity'sinin her tick'te
`setOldPosAndRot`, tick counter ve entity tick gövdesi yolundan geçirilmesini
belirler.

`EntityTypes.java:1163-1173` içindeki built-in kayıt sırası, `player` için
vanilla protocol entity type ID `156` ile eşleşir. `ClientPacketListener.java:565-590`
oyuncu entity'sini add-entity paketinden dünya entity storage'ına ekler; bu
state daha sonra chunk status'ından bağımsız saklanır.

## Sözleşme

- `player` entity type ID `156`, `always_ticking` state'iyle oluşturulur.
- Oyuncu entity'si bulunduğu chunk unload edildiğinde global entity map'inde
  kalır ve `ticking` uygunluğunu korur.
- Chunk dışında kalan oyuncu entity'si `ClientWorld::tick` tarafından bir sonraki
  tick'te yürütülür.
- Sıradan entity'ler için mevcut chunk'a bağlı ticking davranışı korunur.
- Bu dilim entity'nin tam `tick` gövdesini, fizik/motion integration'ını veya
  body/head rotation hesabını uygulamaz; yalnızca Java'nın entity tick listesi
  üyeliğini world kopyasında temsil eder.

## Rust doğrulaması

`ClientEntity` artık `always_ticking` state'ini taşır. `ClientWorld` bunu
oyuncu entity type ID'sinden kurar; chunk unload, entity position ve move
yolları bu state'i ordinary entity chunk uygunluğundan ayrı değerlendirir.
`is_always_ticking()` sorgusu iki state'in ayrımını gözlenebilir kılar.

`player_entities_keep_ticking_after_their_chunk_unloads` unit testi oyuncu
entity'sini yüklü chunk'a ekler, chunk'ı unload eder ve entity'nin hem map'te
hem ticking listesinde kaldığını; sonraki world tick'inde counter'ın arttığını
kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
`view-distance=5`, `simulation-distance=5`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

İki eşzamanlı Rust probe oturumu birbirini görürken ilk oturumun world
kopyasında remote player entity'si aranıp bulunduğu chunk unload edildi:

```text
cargo run --locked --offline --example world_entity_always_ticking_probe -- 127.0.0.1:25568 P08AlwaysE P08AlwaysF
```

İlgili canlı sonuç:

| Gözlem | Sonuç |
| --- | ---: |
| iki oturumda play login | `true / true` |
| ilk / ikinci add-entity paket sayısı | `49 / 24` |
| remote player entity type ID | `156` |
| remote player chunk | `x:1,z:-3` |
| chunk removal | `true` |
| unload sonrası always-ticking | `true` |
| unload sonrası ticking | `true` |
| tick count önce / sonra | `0 / 1` |
| ilk world chunk / entity sayısı | `156 / 43` |
| ikinci world entity sayısı | `23` |
| ilk / ikinci alınan paket sayısı | `9801 / 5624` |

Probe, vanilla server'ın gerçek ikinci oyuncu paketini ve type ID'sini
kullanır; chunk unload sonrası kontrol Rust world kopyasında yapılır. Java
client parity karşılaştırması, gerçek entity tick gövdesi ve oyuncu movement
physics'i açık kalır.

## Açık kapsam

Oyuncu entity'sinin AI/movement tick gövdesi, collision, network interpolation,
passenger ilişkileri, body/head rotation semantic'i ve diğer özel entity
type'larının tam registry metadata'sı sonraki dilimlerdir.
