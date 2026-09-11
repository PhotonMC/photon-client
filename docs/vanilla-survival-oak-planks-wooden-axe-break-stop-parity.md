# P12 — Vanilla 26.2 survival oak planks wooden axe tool progress doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. Kontrollü y=`100` oak planks platformu ve `(100,101,100)` world spawn'ı kullanılarak iki bağımsız oturum açıldı. Her oyuncu oyuna girdikten sonra vanilla server console üzerinden `item replace entity <player> hotbar.0 with minecraft:wooden_axe` uygulandı; geçerli koşular yalnızca bu komutun başarıyla loglandığı oturumlar oldu.

```text
cargo run --offline --release --example survival_oak_planks_wooden_axe_break_stop_probe -- 127.0.0.1:25566 RustOakAxe1
cargo run --offline --release --example survival_oak_planks_wooden_axe_break_stop_probe -- 127.0.0.1:25566 RustOakAxe2
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustOakAxe1 | RustOakAxe2 |
| --- | ---: | ---: |
| carried item packets sent | `1` | `1` |
| player action packets sent | `2` | `2` |
| container preparation | `state=2, slot=36, count=1, item=942, components=[]` | `state=2, slot=36, count=1, item=942, components=[]` |
| container break update | `state=3, slot=36, count=1, item=942, components=[3]` | `state=3, slot=36, count=1, item=942, components=[3]` |
| block changed ACK sequences | `[32,33]` | `[32,33]` |
| block update | `(98,100,102,state=0)` | `(91,100,95,state=0)` |
| chunk packet | `54` | `54` |
| decoded chunk | `54` | `54` |
| chunk batch finished | `16` | `16` |
| received packet | `1106` | `1773` |

Server logu RustOakAxe1 için `(98.5,101.0,102.5)`, RustOakAxe2 için `(91.5,101.0,95.5)` konumlarını ve her iki oyuncuya wooden axe slot değişikliğinin başarıyla uygulandığını gösterdi. Java floor hedefleri sırasıyla `(98,100,102)` ve `(91,100,95)` oldu. İki probe da hedef için state `0` authoritative block update'i aldı; server console kontrolleri hedeflerin air olduğunu ayrıca doğruladı. Vanilla logunda geçerli koşular için protocol error, timeout veya action warning oluşmadı.

## Sonuç ve açık kapsam

Bu sonuç gerçek vanilla server üzerinde wooden axe seçiminin oak planks hedefinde 1.2 saniyelik kontrollü aralıkta kırma eşiğini geçtiğini, carried item paketini, `DAMAGE` component'li slot yayını, action ACK sırasını ve authoritative air update'ini iki geçerli oturumda doğrular. Farklı hardness/tool kombinasyonları, çoklu dayanıklılık, enchantment, erken stop, client prediction ve inventory click açık kalır.

Sözleşme ayrıntıları [survival oak planks wooden axe break stop contract belgesindedir](survival-oak-planks-wooden-axe-break-stop-contract.md). Wooden shovel sonucu [ayrı kayıttadır](vanilla-survival-dirt-wooden-shovel-break-stop-parity.md); çıplak el oak planks sonucu [ayrı kayıttadır](vanilla-survival-oak-planks-break-stop-parity.md).
