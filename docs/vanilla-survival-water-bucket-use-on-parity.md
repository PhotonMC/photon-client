# P12 — Vanilla 26.2 survival water bucket use-on doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. World spawn `(100,101,100)` üzerindeki kontrollü dirt platformu kullanıldı. Her oyuncu oyuna girdikten sonra vanilla server console üzerinden water bucket slot hazırlığı uygulandı; üst hücrenin air olduğu ayrıca sağlandı. Console başarı mesajı görülmeden hiçbir koşul geçerli sayılmadı.

```text
cargo run --offline --release --example survival_water_bucket_use_on_probe -- 127.0.0.1:25566 RustWaterUse4
cargo run --offline --release --example survival_water_bucket_use_on_probe -- 127.0.0.1:25566 RustWaterUse5
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustWaterUse4 | RustWaterUse5 |
| --- | ---: | ---: |
| player position | `(105.5,101.0,106.5)` | `(99.5,101.0,98.5)` |
| support target | `(105,100,106)` | `(99,100,98)` |
| water target | `(105,101,106)` | `(99,101,98)` |
| carried item packets sent | `1` | `1` |
| use item-on packets sent | `1` | `1` |
| use item fallback packets sent | `1` | `1` |
| container slot preparation | `state=2, slot=36, count=1, item=1041` | `state=2, slot=36, count=1, item=1041` |
| container content before use | `state=1, items=46, slot36=empty` | `state=1, items=46, slot36=empty` |
| container content after use | `state=3, items=46, slot36=count1,item1040` | `state=3, items=46, slot36=count1,item1040` |
| block changed ACK sequences | `[24]` | `[24]` |
| target block update | `(105,100,106,state=10)` | `(99,100,98,state=10)` |
| water block update | `(105,101,106,state=86)` | `(99,101,98,state=86)` |
| chunk packet / decoded / batch finished | `99 / 99 / 31` | `99 / 99 / 31` |
| received packet | `2514` | `1852` |

İlk `use_item_on` action'ı listener'ın destek bloğu ve üst hücre snapshot'ını oluşturdu; üst hücre için `state=0` gözlemi, ardından fallback `use_item` içindeki fluid yerleşiminden `state=86` gözlemi geldi. İkinci koşuda su oyuncunun bulunduğu üst katmanda akabildiği için iki ek fluid block update'i de kaydedildi; beklenen kaynak hücresi değişmedi. Server console, ikinci koşunun water hücresini `minecraft:water`, destek bloğunu `minecraft:dirt` olarak doğruladı. Her iki geçerli oturumda system chat gözlemi yok ve server logunda protocol error, action warning veya timeout oluşmadı.

`use_item_on` sequence `23` için ayrı ACK görünmemesi, hemen arkasından sequence `24` geldiğinde vanilla ACK kuyruğunun en yüksek sequence'i `[24]` olarak yayınlamasıyla uyumludur. Bu kayıt iki action'ın gönderildiğini ayrıca ölçer.

## Sonuç ve açık kapsam

Bu sonuç Java istemcisinin water bucket block-use fallback sırasını, survival slot dönüşümünü ve otoriter water block state'ini iki vanilla oturumunda doğrular. Yana yerleştirme, boş bucket pickup, tam fluid flow davranışı, component patch'leri, farklı stack count'ları, inventory click ve client prediction açık kalır.

Sözleşme ayrıntıları [survival water bucket use-on contract belgesindedir](survival-water-bucket-use-on-contract.md). Survival snowball tüketimi [ayrı kayıttadır](vanilla-survival-snowball-use-parity.md); boş el `use_item_on` [ayrı kayıttadır](vanilla-use-item-on-parity.md).
