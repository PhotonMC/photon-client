# P12 — Vanilla 26.2 survival dirt wooden shovel tool progress doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Kontrollü y=`100` dirt platformu ve `(100,101,100)` world spawn'ı kullanılarak iki bağımsız oturum açıldı. Her oyuncu oyuna girdikten sonra vanilla server console üzerinden `item replace entity <player> hotbar.0 with minecraft:wooden_shovel` uygulandı; geçerli koşular yalnızca bu komutun başarıyla loglandığı oturumlar oldu.

```text
cargo run --offline --release --example survival_dirt_wooden_shovel_break_stop_probe -- 127.0.0.1:25566 RustDirtShovel3
cargo run --offline --release --example survival_dirt_wooden_shovel_break_stop_probe -- 127.0.0.1:25566 RustDirtShovel4
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustDirtShovel3 | RustDirtShovel4 |
| --- | ---: | ---: |
| carried item packets sent | `1` | `1` |
| player action packets sent | `2` | `2` |
| container preparation | `state=2, slot=36, count=1, item=940, components=[]` | `state=2, slot=36, count=1, item=940, components=[]` |
| container break update | `state=3, slot=36, count=1, item=940, components=[3]` | `state=3, slot=36, count=1, item=940, components=[3]` |
| block changed ACK sequences | `[30,31]` | `[30,31]` |
| block update | `(102,100,110,state=0)` | `(100,100,92,state=0)` |
| chunk packet | `54` | `54` |
| decoded chunk | `54` | `54` |
| chunk batch finished | `16` | `16` |
| received packet | `448` | `1278` |

Server logu RustDirtShovel3 için `(102.5,101.0,110.5)`, RustDirtShovel4 için `(100.5,101.0,92.5)` konumlarını ve her iki oyuncuya wooden shovel slot değişikliğinin başarıyla uygulandığını gösterdi. Java floor hedefleri sırasıyla `(102,100,110)` ve `(100,100,92)` oldu. İki probe da hedef için state `0` authoritative block update'i aldı; ikinci hedef server console'da `air` olarak ayrıca doğrulandı. Vanilla logunda geçerli koşular için protocol error, timeout veya action warning oluşmadı.

## Sonuç ve açık kapsam

Bu sonuç gerçek vanilla server üzerinde wooden shovel seçiminin dirt hedefinde 650 ms'lik kontrollü aralıkta kırma eşiğini geçtiğini, carried item paketini, `DAMAGE` component'li slot yayını, action ACK sırasını ve authoritative air update'ini iki geçerli oturumda doğrular. Farklı hardness/tool kombinasyonları, çoklu dayanıklılık ilerlemesi, enchantment, erken stop, client prediction ve inventory click açık kalır.

Sözleşme ayrıntıları [survival dirt wooden shovel break stop contract belgesindedir](survival-dirt-wooden-shovel-break-stop-contract.md). Wooden pickaxe sonucu [ayrı kayıttadır](vanilla-survival-stone-wooden-pickaxe-break-stop-parity.md); kısa hardness sonuçları [stone](vanilla-survival-stone-break-stop-parity.md) ve [oak planks](vanilla-survival-oak-planks-break-stop-parity.md) belgelerindedir.
