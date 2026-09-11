# P12 — Vanilla 26.2 survival snowball tüketimi doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. World spawn `(100,101,100)` üzerindeki kontrollü dirt platformu korunarak iki bağımsız oturum açıldı. Her oyuncu oyuna girdikten sonra vanilla server console üzerinden `item replace entity <player> hotbar.0 with minecraft:snowball` uygulandı; console başarı mesajı görülmeden hiçbir koşul geçerli sayılmadı.

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

Sözleşme ayrıntıları [survival snowball use contract belgesindedir](survival-snowball-use-contract.md). Creative snowball kullanımı [ayrı kayıttadır](vanilla-creative-snowball-use-parity.md); boş el `use_item` [ayrı kayıttadır](vanilla-use-item-parity.md).
