# P08.c-light — [server-conformance] ClientWorld ışık state'i

Bu kayıt, vanilla sunucusundan gelen chunk ışık listelerinin ve bağımsız
`LIGHT_UPDATE` paketlerinin Rust `ClientWorld` içindeki sorgulanabilir sky/block
light state'ine uygulanmasını kapsar. Kanıt sınıfı `server-conformance`'dır:
Rust packet akışını Mojang vanilla 26.2 sunucusuna bağlayıp tüketir. Java
istemcisiyle aynı girdide state karşılaştırılmadığı için bu kayıt
`client-parity` değildir.

## Java kaynak incelemesi

`ClientboundLevelChunkWithLightPacket.java:35-39` chunk koordinatlarını ve
chunk gövdesinden sonra gelen ışık verisini taşır. Bağımsız
`ClientboundLightUpdatePacket.java:31-40` chunk X/Z'sini VarInt olarak okuyup
aynı ışık veri yapısını kullanır. `ClientboundLightUpdatePacketData.java:17-112`
dört bitset'i (sky, block, boş sky, boş block) ve ardından sky/block byte
listelerini kaynak sırasıyla okur; her dolu `DataLayer` 2048 byte'tır.

`ClientPacketListener.java:850-861,2385-2405,2537-2558` chunk ile gelen ışığı
chunk kopyası hazırlandıktan sonra uygular, bağımsız ışık paketini de aynı
section-list yoluna verir. `LevelLightEngine.java:157-166` ışık section
aralığını level section sayısına iki sınır section ekleyerek ve minimumu bir
section aşağı alarak tanımlar. `DataLayer.java:22-70` 4096 nibble hücresini
`y << 8 | z << 4 | x` sırasına koyar; çift hücre byte'ın düşük, tek hücre
yüksek nibble'ını kullanır.

## Sözleşme

- Chunk packet'i önce `ClientWorld` chunk kopyasını oluşturur; aynı packet'in
  ışık verisi bu yüklü chunk'a uygulanır.
- Light section bit `i`, `min_section_y - 1 + i` dünya section Y'sine karşılık
  gelir. Geçerli ışık section sayısı level section sayısına iki eklenerek
  hesaplanır.
- Maskedeki her bit, ilgili byte listesinde sıradaki 2048 byte'ı tüketir.
  Empty mask bit'i 2048 sıfır byte'tan oluşan bir layer oluşturur. Aynı konumda
  data maskesi varsa data layer önceliklidir.
- `ClientWorld::sky_light_at` ve `block_light_at`, Java nibble sırasına göre
  0–15 arası değeri döndürür. Yüklü olmayan chunk veya light section için
  sonuç `None`'dır.
- Bağımsız light update yalnızca mevcut yüklü chunk kopyasına uygulanır. Chunk
  kaldırıldığında ham sky/block layer'ları da chunk ile birlikte kaldırılır.
- 2048 byte olmayan bir data layer başarı sayılmaz; probe bu durumda hata
  verir. Maskenin dışındaki fazla section bitleri bu dar aralıkta uygulanmaz.

## Rust doğrulaması

`parse_light_update_packet`, play packet ID `48` için chunk X/Z'sini, dört
bitset'i, iki byte listesini ve trailing payload kontrolünü yapar. Birim testi
negatif X, mask bit sırası, 2048 byte layer uzunluğu ve trailing byte hatasını
kontrol eder.

`ClientWorld` birim testi, packed nibble düzenini, bit ile section Y eşlemesini,
empty layer sıfırlarını, sky/block ayrımını ve chunk unload sonrasındaki ışık
sorgusunu kontrol eder. `world_light_probe`, ilk chunk packet'lerinin ışık
state'ine yazıldığını; `world_light_update_probe` ise bağlantı açıkken gelen
bağımsız ışık paketlerinin world query'ye işlendiğini ölçer.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-12

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, yerel
`127.0.0.1:25568` koşusu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Başlangıç chunk ışığı için:

```text
cargo run --locked --offline --release --example world_light_probe -- 127.0.0.1:25568 LightR2
```

Bağımsız update için probe bağlantısı açık tutuldu; oyuncu yüklendikten sonra
sunucu konsolunda aynı hedefte önce `minecraft:air`, sonra
`minecraft:glowstone` kullanıldı:

```text
cargo run --locked --offline --release --example world_light_update_probe -- 127.0.0.1:25568 LightUpdateR7
setblock 25 80 -36 minecraft:air
setblock 25 80 -36 minecraft:glowstone
```

Sonuçlar:

| Gözlem | Başlangıç chunk ışığı | Bağımsız `LIGHT_UPDATE` |
| --- | ---: | ---: |
| ışık update/list kayıtları | `201` initial data layer | `6` packet |
| uygulanan light section | chunk state'inin parçası | `8` |
| yüklü light section | `279` | `1846` |
| sky sample | `(16,-64,-48)=0` | — |
| block light sample | `(23,80,-48)=1` | `(25,80,-36)=15` |
| world chunk / section | `24 / 576` | `157 / 3768` |
| alınan packet | `482` | `9403` |

Bağımsız koşuda ilk gözlenen ışık paketi `(x=5,z=-4)` için sky listesi boş,
block listesi tek bir `2048` byte layer ve uygulama sonucu iki block section'dı.
Server konsolu iki `setblock` değişikliğini başarıyla bildirdi; Rust probe
sonunda hedef hücreyi `Some(15)` olarak sorguladı. Bu sonuç vanilla'nın ışık
paketlerinin Rust world kopyasına uygulandığını gösterir; Java istemcisiyle
parity iddiası değildir.

## Açık kapsam

Java istemcisinin henüz yüklenmemiş chunk için bağımsız light update'ı kuyruğa
alma davranışı, tam light propagation/relight engine'i, render/AO bağlantısı,
heightmap semantiği, dinamik dimension/respawn, block entity ve entity yaşam
döngüsü açık kalır. Ham server light state'inin sorgulanması oynanabilir
render veya tam dünya eşitliği kanıtı değildir.
