# P12 — Survival snowball tüketimi sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun slot 0'daki tek snowball stack'ini seçip ana elle `use_item` göndermesini sınar. Amaç dolu stack item kullanımının action sequence ACK'ini, projectile spawn'ını ve tüketim sonrası clientbound container slot düzeltmesini gerçek Java server davranışıyla karşılaştırmaktır.

## Java referansı

`ServerGamePacketListenerImpl` `use_item` action'ında sequence'i ACK kuyruğuna alır ve eldeki boş olmayan stack'i game mode'a verir. `SnowballItem.use` server level'da rotation'dan projectile üretir, kullanım istatistiğini günceller ve stack'ten bir item tüketir. Tüketilen player inventory hotbar slotu `ClientboundContainerSetSlotPacket` ile boş stack olarak gönderilir. Item stack wire codec'i boş stack için count `0` taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: oyuncu oyuna girdikten sonra `item replace entity <player> hotbar.0 with minecraft:snowball`
- Rust hazırlığı: on iki saniyelik console komutu aralığından sonra `SET_CARRIED_ITEM`, slot `0`
- Action: `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `0.0`, sequence `22`
- Stack: slot `0`'da tek snowball

Rust probe container slot paketlerini, add entity paketlerini ve ACK'leri toplar. Beklenen sonuç carried item packet sayısı `1`, use item packet sayısı `1`, ACK `[22]`, snowball entity type `120` ile bir add-entity gözlemi ve hotbar slot `36` için boş optional item stack (`item_count=None`) gözlemidir. Hazırlık sırasında slot `36` için count `1` stack güncellemesi de görülebilir.

## Sınır

Bu dilim tek stack snowball tüketimini ve projectile spawn wire akışını kapsar. Item component patch'leri, farklı stack count'ları, projectile hareket/impact, iki el önceliği, inventory click ve client prediction sonraki dilimlerdir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. World spawn `(100,101,100)` üzerindeki kontrollü dirt platformu korunarak iki bağımsız oturum açıldı. Her oyuncu oyuna girdikten sonra vanilla server console üzerinden `item replace entity <player> hotbar.0 with minecraft:snowball` uygulandı; console başarı mesajı görülmeden hiçbir koşul geçerli sayılmadı.

```text
cargo run --offline --release --example survival_snowball_use_probe -- 127.0.0.1:25566 RustSnowSurv5
cargo run --offline --release --example survival_snowball_use_probe -- 127.0.0.1:25566 RustSnowSurv6
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustSnowSurv5 | RustSnowSurv6 |
| --- | ---: | ---: |
| carried item packets sent | `1` | `1` |
| use item packets sent | `1` | `1` |
| container slot preparation | `state=2, slot=36, count=1, item=1044` | `state=2, slot=36, count=1, item=1044` |
| container content before use | `state=1, items=46, slot36=empty, carried=empty` | `state=1, items=46, slot36=empty, carried=empty` |
| container content after use | `state=3, items=46, slot36=empty, carried=empty` | `state=3, items=46, slot36=empty, carried=empty` |
| block changed ACK sequences | `[22]` | `[22]` |
| snowball add-entity | `entity=390, type=120, owner=339` | `entity=417, type=120, owner=391` |
| chunk packet | `54` | `54` |
| decoded chunk | `54` | `54` |
| chunk batch finished | `16` | `16` |
| received packet | `1042` | `807` |

RustSnowSurv5 server logunda `(95.5,101.0,93.5)` konumunda entity ID `339`, RustSnowSurv6 için `(107.5,101.0,106.5)` konumunda entity ID `391` ile giriş görüldü; console her iki slot değişikliğini de `[Snowball]` olarak başarıyla bildirdi. Add-entity konumları oyuncu konumunun y ekseninde `102.52000000327826` değerine, owner data alanları ilgili oyuncu entity ID'sine uydu. Vanilla logunda geçerli koşular için protocol error, timeout veya action warning oluşmadı.

## Sonuç ve açık kapsam

Bu sonuç survival dolu snowball stack kullanımında `SET_CARRIED_ITEM`, `use_item` ACK'i, vanilla projectile spawn'ı ve kullanım sonrası inventory content state'inde hotbar slotunun boşalmasını iki vanilla oturumunda doğrular. Item component patch'leri, farklı stack count'ları, projectile hareket/impact, iki el önceliği, inventory click ve client prediction açık kalır.

Sözleşme ayrıntıları [survival snowball use contract belgesindedir](#sözleşme). Creative snowball kullanımı [ayrı kayıttadır](creative-snowball-use-conformance.md); boş el `use_item` [ayrı kayıttadır](use-item-conformance.md).
