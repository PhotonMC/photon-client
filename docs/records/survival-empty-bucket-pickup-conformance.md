# P12 — Survival empty bucket pickup sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun slot 0'daki boş bucket'ı ayak hücresindeki water source üzerinde kullanmasını sınar. Amaç Java istemcisinin `use_item_on` → `use_item` fallback sırasını, source-only raycast'i, water source'un kaldırılmasını ve bucket'ın water bucket'a dönüşmesini karşılaştırmaktır.

## Java referansı

Empty bucket'ın item sınıfı block-use override'ı taşımadığı için Java istemcisi block hit üzerinde önce `use_item_on` yollar ve PASS sonucundan sonra `use_item` fallback'ine geçer. Server fallback action'ında bucket'ın source-only raycast'i water block'ı bulur. Empty-content bucket'ın doğrudan yerleştirme yolu başarısız olduktan sonra `BucketPickup` source'u kaldırır ve `ItemUtils` sonucu water bucket'a dönüştürür.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: oyuncunun ayak hücresine `minecraft:water`, slot 0'a `minecraft:bucket`
- İlk action: source block'ın üst yüzü, `use_item_on`, `MAIN_HAND`, sequence `25`
- Fallback action: `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `90.0`, sequence `26`

Rust probe container content, slot, block update ve ACK packet'lerini toplar. Beklenen sonuç carried item packet sayısı `1`, her iki action packet sayısı `1`, ACK `[26]`, hazırlıkta slot `36` için bucket registry `1040`, kullanım sonrası content state `3` içinde slot `36` için water bucket registry `1041` ve source block için `state=86`dan `state=0`a geçiştir.

## Sınır

Bu dilim ayak hücresindeki source water pickup senaryosunu kapsar. Yana pickup, flowing water, lava/powder-snow bucket'ları, boş olmayan stack'ler, component patch'leri ve inventory click sonraki dilimlerdir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` dizininde `online-mode=false`, seed `776`, view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`, peaceful ve spawn protection `0` ayarlarıyla çalıştırıldı. World spawn `(100,101,100)` üzerindeki kontrollü dirt platformu kullanıldı. Her oyuncu oyuna girdikten sonra ayak hücresine source water ve slot 0'a empty bucket console üzerinden verildi. Console başarı mesajı görülmeden hiçbir koşul geçerli sayılmadı.

```text
cargo run --offline --release --example survival_empty_bucket_pickup_probe -- 127.0.0.1:25566 RustBPickup2
cargo run --offline --release --example survival_empty_bucket_pickup_probe -- 127.0.0.1:25566 RustBPickup3
```

İki bağımsız geçerli koşunun sonuçları:

| Gözlem | RustBPickup2 | RustBPickup3 |
| --- | ---: | ---: |
| player position | `(94.5,101.0,106.5)` | `(110.5,101.0,99.5)` |
| source position | `(94,101,106)` | `(110,101,99)` |
| carried item packets sent | `1` | `1` |
| use item-on packets sent | `1` | `1` |
| use item fallback packets sent | `1` | `1` |
| container slot preparation | `state=2, slot=36, count=1, item=1040` | `state=2, slot=36, count=1, item=1040` |
| container content before use | `state=1, items=46, slot36=empty` | `state=1, items=46, slot36=empty` |
| container content after use | `state=3, items=46, slot36=count1,item1041` | `state=3, items=46, slot36=count1,item1041` |
| block changed ACK sequences | `[26]` | `[26]` |
| source block transition | `state=86 → state=0` | `state=86 → state=0` |
| chunk packet / decoded / batch finished | `99 / 99 / 31` | `99 / 99 / 31` |
| received packet | `1954` | `2019` |

İlk `use_item_on` action'ı source ve üst komşusunun mevcut durumunu snapshot olarak yayınladı; fallback `use_item` source water block'ını kaldırdı. Ayak hücresindeki source water, server tick'leri boyunca çevreye flow update'leri de üretti; bu ek update'ler ham runner çıktısında korunur, source transition ve inventory sonucu değişmez. Her iki koşuda server console source koordinatının `air` olduğunu doğruladı. System chat gözlemi yoktur; geçerli oturumlarda server logunda protocol error, action warning veya timeout oluşmamıştır.

`use_item_on` sequence `25` için ayrı ACK görünmemesi, hemen arkasından sequence `26` geldiğinde vanilla ACK kuyruğunun en yüksek sequence'i `[26]` olarak yayınlamasıyla uyumludur. Runner iki clientbound action sonucunu ve gönderilen iki serverbound action'ı ayrı ayrı kaydeder.

## Sonuç ve açık kapsam

Bu sonuç Java istemcisinin empty bucket fallback akışını, source-only pickup sonucunu, survival inventory dönüşümünü ve source block'in air update'ini iki vanilla oturumunda doğrular. Yana pickup, flowing water ayrıntıları, diğer bucket türleri, component patch'leri, farklı stack count'ları, inventory click ve client prediction açık kalır.

Sözleşme ayrıntıları [survival empty bucket pickup contract belgesindedir](#sözleşme). Water bucket yerleştirme [ayrı kayıttadır](survival-water-bucket-use-on-conformance.md); survival snowball tüketimi [ayrı kayıttadır](survival-snowball-use-conformance.md).
