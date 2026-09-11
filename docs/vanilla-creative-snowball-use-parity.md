# P12 — Vanilla 26.2 creative snowball use doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=creative`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Kontrollü taş platformu ve world spawn `(100,101,100)` korunarak iki bağımsız oturum açıldı.

```text
cargo run --offline --release --example creative_snowball_use_probe -- 127.0.0.1:25566 RustSnowball1
cargo run --offline --release --example creative_snowball_use_probe -- 127.0.0.1:25566 RustSnowball2
```

İki bağımsız koşunun sonuçları:

| Gözlem | RustSnowball1 | RustSnowball2 |
| --- | ---: | ---: |
| snowball item registry ID | `1044` | `1044` |
| creative slot packets sent | `1` | `1` |
| carried item packets sent | `1` | `1` |
| use item packets sent | `1` | `1` |
| add entity count | `1` | `1` |
| entity type ID | `120` | `120` |
| add entity data (owner ID) | `269` | `289` |
| add entity position | `(109.5,102.52000000327826,105.5)` | `(98.5,102.52000000327826,97.5)` |
| block changed ACK sequences | `[11]` | `[11]` |
| chunk packet | `24` | `24` |
| decoded chunk | `24` | `24` |
| chunk batch finished | `6` | `6` |
| received packet | `111` | `111` |

Server logundaki oyuncu entity ID'leri sırasıyla `269` ve `289`, oyuncu konumları ise `(109.5,101.0,105.5)` ve `(98.5,101.0,97.5)` idi. Her add-entity data alanı, ilgili oyuncu ID'siyle eşleşti; spawn y koordinatı oyuncu y'sinin yaklaşık `1.52000000327826` üstündeydi. Vanilla logunda protocol error, timeout veya creative slot/use warning oluşmadı.

## Sonuç ve açık kapsam

Bu sonuç creative item stack kodlamasını, snowball item registry ID'sini, `use_item` ACK'ini, vanilla snowball entity spawn'ını ve owner data alanını iki vanilla oturumunda doğrular. Creative tüketim yayını, projectile hareket/impact, dolu consumable/component item'ler, inventory ve görsel/ses feedback'i açık kalır.

Sözleşme ayrıntıları [creative snowball use contract belgesindedir](creative-snowball-use-contract.md). Boş el `use_item` [ayrı kayıttadır](vanilla-use-item-parity.md); creative placement [ayrı kayıttadır](vanilla-creative-placement-parity.md).
