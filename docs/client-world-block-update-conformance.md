# P08.c — [server-conformance] ClientWorld block update uygulaması

Bu kayıt, vanilla sunucusundan gelen tekil block update paketinin zaten yüklü
bir `ClientWorld` section'ındaki sorgulanabilir block-state hücresini
değiştirmesini kapsar. Kanıt sınıfı `server-conformance`'dır: Rust paket
codec'i vanilla sunucusundan gelen güncellemeyi kabul eder ve world kopyasına
uygular. Java istemcisi ile Rust istemcisinin aynı girdideki state sonucunu
karşılaştırmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundBlockUpdatePacket.java` tek bir `BlockPos` ve block-state registry
ID'si taşır. `ClientPacketListener.java:935-938` bu paketi
`ClientLevel.setServerVerifiedBlockState` yoluna verir. `ClientLevel.java:199-203`
prediction handler güncellemeyi tüketmediğinde state'i temel level kopyasına
yazar; bu dilimdeki Rust uygulaması yüklü section için aynı gözlenebilir
değişikliği world query katmanında tutar.

## Sözleşme

- Başlangıç dünya kopyası, P08.a/b kaydındaki vanilla chunk teslimiyle
  `ClientWorld` içine yazılır.
- Güncelleme koordinatı yüklü chunk ve section içindeyse o hücrenin global
  block-state ID'si değişir ve `block_state_at` yeni ID'yi döndürür.
- Yüklü olmayan chunk veya section güncellemesi bu dar dilimde uygulanmış
  sayılmaz; ışık, block entity, chunk unload ve prediction yaşam döngüsü ayrı
  işlerdir.

## Rust doğrulaması

`ClientWorld::update_block_state` için yüklü, negatif koordinatlı section,
başarılı sorgu ve yüklü olmayan hedef reddi birim testinde kontrol edildi.

```text
cargo test --locked --offline --all-targets
53 passed; 0 failed
```

`ClientboundBlockUpdatePacket` decode edildikten sonra bağlantı probe'u
`ClientWorld`'ü günceller. Water bucket probe'u aynı oturumda hem packet
observation'ını hem de world query sonucunu yazdırır.

## Vanilla kanıtı

**Tarih:** 2026-09-12

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** SHA-1'i `823e2250d24b3ddac457a60c92a6a941943fcd6a` olan Mojang vanilla
26.2 dedicated server, offline ve seed `776`

```text
cargo run --locked --offline --release --example survival_water_bucket_use_on_probe -- 127.0.0.1:25569 WaterWorldC26

item replace entity WaterWorldC26 hotbar.0 with minecraft:water_bucket
```

Geçerli koşuda server console item replacement başarısını bildirdi; probe
`use_item_on` ve `use_item` fallback paketlerini birer kez gönderdi, ACK `[24]`
aldı ve 99 chunk'ı decode etti. Vanilla'nın gönderdiği block update ile Rust
world query sonuçları aynıydı:

| Gözlem | Sonuç |
| --- | --- |
| destek hedefi packet update | `(24,70,-38,state=9)` |
| destek hedefi `ClientWorld` query | `Some(9)` |
| water komşusu packet update | `(24,71,-38,state=86)` |
| water komşusu `ClientWorld` query | `Some(86)` |
| container sonucu | `state=3, slot=36, item=1040` |
| block changed ACK | `[24]` |
| chunk packet / decoded / batch finished | `99 / 99 / 31` |

Server logunda protocol error, action warning veya timeout görülmedi. Bu
sonuç, sunucunun block update paketinin kabul edildiğini ve yüklü world
state'inin güncellendiğini gösterir; Java istemcisiyle parity iddiası değildir.

## Açık kapsam

Dinamik dimension/respawn, ışık nibble state'i, block entity kopyası, chunk
radius/unload sırası, entity deposu, prediction handler ile server-verified
state'in çakışması ve render invalidation açık kalır.
