# P12 — Survival water bucket use-on sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun slot 0'daki tek water bucket'ı destek bloğunun üst yüzünde kullanmasını sınar. Amaç Java istemcisinin `use_item_on` → `use_item` fallback sırasını, sequence ACK davranışını, suyun otoriter blok güncellemesini ve water bucket'ın bucket'a dönüşmesini karşılaştırmaktır.

## Java referansı

Java istemcisi block hit üzerinde önce `use_item_on` action'ını başlatır. Water bucket'ın item sınıfı block-use override'ı taşımadığı için bu action item tarafından tüketilmez; istemci aynı input işleminin devamında oyuncunun bakış rotasyonuyla `use_item` yollar. Server ikinci action'da sequence'i ACK kuyruğuna alır, oyuncunun POV raycast'ini kullanarak bucket içeriğini yerleştirir ve survival eldeki sonucu boş bucket'a dönüştürür. İlk listener snapshot'ı ile fluid yerleşiminin block update'i ayrı packet'ler olarak görülebilir.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: oyuncu oyuna girdikten sonra `item replace entity <player> hotbar.0 with minecraft:water_bucket`
- Ortam: oyuncunun altındaki y=`100` dirt destek bloğu; üst hücre başlangıçta air
- Rust hazırlığı: console komutları için yirmi saniyelik aralık, ardından `SET_CARRIED_ITEM`, slot `0`
- İlk action: destek bloğunun üst yüzü, `use_item_on`, `MAIN_HAND`, sequence `23`
- Fallback action: `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `90.0`, sequence `24`

Rust probe container content, slot, block update ve ACK packet'lerini toplar. Beklenen sonuç carried item packet sayısı `1`, `use_item_on` ve `use_item` sayıları `1`, ACK `[24]`, hazırlıkta slot `36` için water bucket registry `1041`, kullanım sonrası content state `3` içinde slot `36` için bucket registry `1040` ve üst hücrede water source state `86` gözlemidir.

## Sınır

Bu dilim üst yüze su kaynağı koyan tek water bucket senaryosunu kapsar. Yana yerleştirme, boş bucket ile fluid pickup, akışın bütün komşulara yayılması, lava/powder-snow bucket'ları, component patch'leri, farklı stack sayıları ve inventory click sonraki dilimlerdir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. World spawn `(100,101,100)` üzerindeki kontrollü dirt platformu kullanıldı. Her oyuncu oyuna girdikten sonra vanilla server console üzerinden water bucket slot hazırlığı uygulandı; üst hücrenin air olduğu ayrıca sağlandı. Console başarı mesajı görülmeden hiçbir koşul geçerli sayılmadı.

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

Sözleşme ayrıntıları [survival water bucket use-on contract belgesindedir](#sözleşme). Survival snowball tüketimi [ayrı kayıttadır](survival-snowball-use-conformance.md); boş el `use_item_on` [ayrı kayıttadır](use-item-on-conformance.md).
