# P08.p — [server-conformance] LocalPlayer effective attribute hesabı

Bu kayıt, local player attribute snapshot'ındaki base değer ve modifier'ların
Java operasyon sırasıyla etkili değere dönüştürülmesini kapsar. Canlı kanıt
sınıfı `server-conformance`'dır: Rust probe'u Mojang vanilla 26.2 dedicated
server'ından gelen attribute state'ini `ClientWorld` içine uygular ve
hesaplanan sonucu kontrol eder. Aynı girdide Java istemcisiyle oturum
karşılaştırması yapılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`AttributeInstance.java:139-170`, etkili değeri önce base değerden başlatır;
`ADD_VALUE` modifier'larını base'e ekler, `ADD_MULTIPLIED_BASE` modifier'larını
ilk adımdan sonraki base üzerinden sonuca ekler ve `ADD_MULTIPLIED_TOTAL`
modifier'larını o ana kadarki sonuçla çarpar. Son değer attribute'ın
`sanitizeValue` yolundan geçer.

`AttributeModifier.java:38-64`, operation ID'lerini `0=add_value`,
`1=add_multiplied_base`, `2=add_multiplied_total` olarak çözer. ID sınırları
dışındaki ham operation değeri Java'nın `BY_ID` lookup'ında zero operation'a
map edilir. `Attribute.java:43-45` temel sanitize yolunu tanımlar;
`RangedAttribute.java:26-37` ise NaN'ı minimum değere, diğer değerleri
attribute aralığına uygular.

`ClientPacketListener.java:2348-2372`, gelen snapshot için base değeri yazar,
eski modifier'ları temizler ve packet modifier'larını ekler. `Attributes.java:77-79`
hareket hızının (`movement_speed`) default değerini `0.7`, aralığını `0..1024`
ve sync edilebilir olduğunu tanımlar. Pinned Java oracle'da bu attribute'ın
registry ID'si `26` olarak gözlendi.

## Sözleşme

- `ClientAttributeState` ham base değerini ve modifier kayıtlarını saklar;
  etkili değer hesabında operation grupları Java sırasıyla işlenir.
- `ADD_VALUE` base'i değiştirir. Sonraki `ADD_MULTIPLIED_BASE` adımı bu yeni
  base'i kullanır. `ADD_MULTIPLIED_TOTAL` her modifier için çalışan sonucu
  `1 + amount` ile çarpar.
- Bilinmeyen ham operation ID'si Java decode davranışına uygun olarak
  `ADD_VALUE` kabul edilir.
- Sonuç NaN ise minimum değere, aksi halde verilen `[min,max]` aralığına
  sanitize edilir. Bu API aralığı çağıran sağlar; attribute registry adını ve
  aralığını bu dilim çözmez.
- Kontrollü senaryoda `movement_speed` base `0.25`, modifier'lar sırasıyla
  `0.1/add_value`, `0.2/add_multiplied_base` ve
  `0.5/add_multiplied_total` taşır. Beklenen etkili değer `0.63`, IEEE-754
  bitleri `0x3fe428f5c28f5c29`'dur.

## Rust doğrulaması

`ClientAttributeState::effective_value(min_value, max_value)`, üç Java
operation grubunu ve range sanitize davranışını uygular. Unit test aynı
senaryonun bitlerini, bilinmeyen operation fallback'ini, üst sınır clamp'ini
ve NaN davranışını kontrol eder.

`examples/world_player_attribute_effective_probe.rs`, mevcut custom dimension
respawn trigger'ını yeniden kullanır; yeni bir `ProbeAction` varyantı açmaz.
Probe, local player attribute ID `26`, base/modifier ham state'i ve etkili
değeri sorgulanabilir world state'inden doğrular.

## Java oracle

Başlangıç hesap formülü, referans envanterindeki Java Edition 26.2 client JAR'ı
(`2dc72797acbc1b63fc16a11c4ac393605f453754`) ile headless yerel oracle'da
çalıştırıldı. Oracle bootstrap sonrasında `movement_speed` ID'sini `26`,
`max_health` ID'sini `23`, `attack_speed` ID'sini `5` verdi; kontrollü
modifier setinin etkili sonucu `0.63` ve aynı bit dizisi
`0x3fe428f5c28f5c29` oldu. Oracle çıktısı Rust implementasyonundan türetilmedi;
orijinal sınıf ve asset içeriği repository'ye kopyalanmadı.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Custom dimension kayıtlarında kullanılan sentetik `p08_custom_dimension`
datapack'i `/tmp/photon-vanilla-26.2-world/world/datapacks` altında tutuldu;
repository'ye eklenmedi ve orijinal asset taşımıyor. Her probe login olduktan
sonra vanilla server konsolunda şu komutlar çalıştırıldı:

```text
attribute <probe-user> minecraft:movement_speed base set 0.25
attribute <probe-user> minecraft:movement_speed modifier add p08:oracle_add 0.1 add_value
attribute <probe-user> minecraft:movement_speed modifier add p08:oracle_base 0.2 add_multiplied_base
attribute <probe-user> minecraft:movement_speed modifier add p08:oracle_total 0.5 add_multiplied_total
execute in p08:test run tp <probe-user> 0 80 0
```

Rust probe:

```text
cargo run --locked --offline --example world_player_attribute_effective_probe -- \
  127.0.0.1:25568 <probe-user>
```

İki bağımsız vanilla oturumunda bütün kabul alanları aynı gözlendi:

| Gözlem | `P08pB` | `P08pC` |
| --- | ---: | ---: |
| bağlantı | `ok` | `ok` |
| respawn dimension | `p08:test` | `p08:test` |
| `data_to_keep` | `3` | `3` |
| attribute ID | `26` | `26` |
| base | `0.25` | `0.25` |
| modifier sayısı | `3` | `3` |
| etkili değer | `0.63` | `0.63` |
| etkili değer bitleri | `3fe428f5c28f5c29` | `3fe428f5c28f5c29` |
| `p08:oracle_add` | `0.1`, operation `0` | `0.1`, operation `0` |
| `p08:oracle_base` | `0.2`, operation `1` | `0.2`, operation `1` |
| `p08:oracle_total` | `0.5`, operation `2` | `0.5`, operation `2` |
| final world dimension | `p08:test` | `p08:test` |
| final chunk/section sayısı | `157 / 1884` | `157 / 1884` |
| local player attribute update sayısı | `2` | `2` |

Toplam alınan packet sayısı `P08pB` için `5698`, `P08pC` için `6305` oldu;
bu oturum zamanlamasına bağlı gözlem kabul ölçütü değildir. İki koşu da custom
dimension replacement sonrasında vanilla'nın gönderdiği local player
attribute state'ini, operation ID'lerini ve etkili değeri başarıyla doğruladı.

Bu sonuç Rust paket tüketimini ve vanilla sunucusunun attribute state'ini kabul
ettiğini gösterir; Java client eşitliği veya movement/health/combat gameplay
etkileri için kanıt değildir.

## Açık kapsam

Attribute registry adından otomatik range çözümü, bir operation grubundaki
birden çok modifier'ın Java map iteration semantiği, attribute state'inin
movement/health/combat hesaplarına bağlanması, equipment ve mob-effect
modifier'ları, remote living entity state'i ve `client-parity` karşılaştırması
açık kalır.
