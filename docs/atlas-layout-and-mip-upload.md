# P16.n — Atlas yerleşimi, dolgu ve mip yüklemesi — karışık kanıt kaydı

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Bu kayıt üç ayrı kanıt sınıfı taşır ve hangisinin hangi parçaya ait olduğunu
açıkça söyler:

- `client-parity`: `TextureAtlasSprite` yerleşimi ve UV'leri, sprite doku meta
  verisi, piksellerden saydamlık ölçümü. Pinned JAR'daki gerçek sınıflara
  verilip bit düzeyinde karşılaştırılır.
- **Türetilmiş**: dolgunun kenar texel'iyle doldurulması ve etkin mip
  seviyesinin kısıtlanması. Referans kodu okundu ve aritmetiği burada
  yeniden üretildi; ikisi de headless bir oracle'dan sürülemiyor (biri GPU
  çizim geçişi, diğeri `Minecraft.getInstance().options` istiyor).
- **Alıntı**: Vulkan sampler alanları. Referansın kendi
  `VulkanGpuSampler` yapıcısından okundu, karşılaştırılmadı.

## Bulunan hata: sprite bir texel kaymış yazılıyordu

Paketleyici `gatherSprites` ile **dolgulu yuvanın** sol üst köşesini verir,
sprite'ın kendi köşesini değil. `TextureAtlasSprite` yapıcısı bunu şöyle
düzeltir:

```java
this.u0 = (float)(x + padding) / atlasWidth;
this.u1 = (float)(x + padding + contents.width()) / atlasWidth;
```

Bizim `build_atlas` fonksiyonumuz dolguyu geri eklemiyordu: hem yazma hem UV
`placement.x`'i kullanıyordu. Kayma ikisinde de aynı olduğu için çizim doğru
görünüyordu ve bu yüzden gözden kaçmıştı. Yanlış olan şu: dolgu alanı boş
kalıyordu, yani kenar texel'i yayılmıyordu ve mip seviyeleri hiç kullanılamazdı
(paketleyicinin ayırdığı dolgu tam olarak en derin seviyenin bir texel'i
kadardır). `SpritePlacement` alanlarının belgesi de "padding already applied"
diyordu; o da yanlıştı ve düzeltildi.

`image/sprite-uv` bu aritmetiği yedi satırda pinliyor. Yapıcı `protected`
olduğu ve GPU istemediği için oracle onu açıyor, yaptığı dört bölmeyi yeniden
yazmıyor. `padded` satırı tek başına yeterli: `x=0`, `padding=1`, atlas `256`
için `u0 = 0.00390625 = 1/256`.

## Dolgu neyle dolar

Referans atlası seviye başına bir çizim geçişiyle doldurur. Her sprite için
dörtgen, dolgulu yuvayı kaplar (`translate(x >> level, y >> level)`,
`scale((w + 2p) >> level, (h + 2p) >> level)`) ve doku koordinatları
`-p/w`'den `1 + p/w`'ye gider (`animate_sprite.vsh`), en yakın texel ve
kenarda kenetleme ile örneklenir.

Aritmetiği açarsak, `t = (i + 0.5) / W'` ve `uv = t(1 + 2p/w) - p/w` için
`uv · w' = i + 0.5 - p'` çıkar (`w' = w >> N`, `p' = p >> N`,
`W' = w' + 2p'`). Yani atlas texel'i `i`, sprite texel'i `i - p'`'yi okur ve
aralığa kenetlenir: **dolgu kenar texel'inin tekrarıdır**. `blit_sprite` bunu
doğrudan tamsayı olarak uygular; kayan noktalı ara adım yoktur.

Bu türetme `crates/voxel-render/src/resolve.rs` içindeki üç testle korunuyor: sprite
dolgunun ötesine yazılıyor, sol/sağ dolgu ilk/son sütunu tekrarlıyor, köşe
iki kenarın harmanı değil köşe texel'inin kendisi.

## Etkin mip seviyesi

İstenen seviye bir tavandır, taban değil. `SpriteLoader.stitch` bütün sprite
kümesini tarar: en küçük kenar ve her kenarın en düşük set biti seviyeyi
düşürebilir, çünkü temiz bölünemeyen bir kenar yarıya inemez.
`effective_mip_level` bunu uygular; `Mth.log2` zaten `math` ailesinde pinli,
`Integer.lowestOneBit` ise `x & -x`. Java kaydırma sayacını beş bitle
maskelediği için saçma bir istek burada da sarıyor, taşmıyor.

Bileşim headless sürülemiyor (özel metot, `Minecraft.getInstance().options`
istiyor), bu yüzden türetilmiş sayılıyor ve birim testleriyle korunuyor.

## Sprite meta verisi ve saydamlık

`image/texture-metadata` yirmi satırda `texture` bölümünü pinliyor. Özü şu:
bütün alanlar isteğe bağlı, tanınmayan alan yok sayılıyor, ama **yanlış türde**
bir alan bütün bölümü reddediyor — tırnaklı bir sayı, büyük harfli bir strateji
adı ve sayı olarak verilmiş bir strateji hep reddediliyor. Varsayılan strateji
`auto`, varsayılan bias sıfır.

`image/compute-transparency` `AUTO`'yu çözen ölçümü pinliyor: yalnızca tam
sıfır alfa "saydam", yalnızca tam 255 "opak" sayılıyor, ikisi aynı anda
doğru olabiliyor ve `254` alfalı tek bir texel bütün sprite'ı yarı saydam
yapıyor.

İkisi de artık bağlı: `FolderAssets` sprite'ın yanındaki `.png.mcmeta`
dosyasını okuyor, `build_atlas` saydamlığı piksellerden ölçüyor ve
[mip üretimi](records/mipmap-conformance.md) gerçek stratejiyle çalışıyor.

## Sampler — beklenmedik bir bulgu

`LevelRenderer`'ın arazi sampler'ı şudur:

```java
createSampler(AddressMode.CLAMP_TO_EDGE, AddressMode.CLAMP_TO_EDGE,
              FilterMode.LINEAR, FilterMode.LINEAR, maxAnisotropy, OptionalDouble.empty())
```

Yani **büyütmede de doğrusal** süzme. Piksel sanatı bir oyun için şaşırtıcı ve
bizim eski kodumuz `NEAREST` varsayıyordu; o yorum yanlıştı ve düzeltildi.
Üç kontrol bunun okuma hatası olmadığını gösteriyor: `TextureFilteringMethod`
yalnızca `NONE`/`RGSS`/`ANISOTROPIC` taşıyor, yani süzme seçeneği en
yakın/doğrusal seçimi yapmıyor; `block.fsh` düz `texture()` çağırıyor, yani
gölgelendirici kendi texel yuvarlamasını yapmıyor; ve kenar dolgusunun
varlığı ancak doğrusal süzmeyle anlam kazanıyor — en yakın süzmede o dolgunun
yapacak işi olmazdı.

Bir texel bir piksele denk geldiğinde doğrusal süzme texel merkezine oturur,
yani oyun içi görünüm keskin kalır; yumuşama yalnızca büyütmede ortaya çıkar.
Aşağıdaki çıktıda küp ekranı doldurduğu için bu abartılı görünüyor.

`VulkanGpuSampler`'dan alınan diğer alanlar: `mipmapMode` doğrusal (maxLod
`0.25`'ten büyük olduğunda), `mipLodBias` sıfır, `maxLod` bin, `minLod` sıfır,
üçüncü eksen varsayılanda (tekrar) bırakılıyor ve hiç örneklenmiyor,
anizotropi `maxAnisotropy > 1` olmadıkça kapalı.

## Yükleme

`upload_texture` artık bütün seviyeleri alıyor: tek bir hazırlama tamponu,
seviye başına bir kopyalama bölgesi, `mip_levels` görüntü seviyesi sayısı
kadar. GPU tarafında hiçbir şey blit edilmiyor, çünkü seviyeler zaten
işlemcide üretilmiş durumda.

## Sonuçlar

`parity_passed`, 3895 senaryo / 12 aile. 197 birim testi, clippy sıfır uyarı.

`minecraft:block/glass` için atlas dökümü (`VOXEL_ATLAS_DUMP`):

| Seviye | Atlas | Dolgu | Sprite köşesi | Dolgudaki komşu |
| --- | --- | --- | --- | --- |
| 0 | 64x64 | 16 | `d0eae9` | `d0eae9` |
| 1 | 32x32 | 8 | `c0d6d5` | `c0d6d5` |
| 2 | 16x16 | 4 | `aebfbe` | `aebfbe` |
| 3 | 8x8 | 2 | `9ba7a6` | `9ba7a6` |
| 4 | 4x4 | 1 | `8f9c9e` | `8f9c9e` |

Her seviye bir öncekinin tam yarısı, sprite `dolgu >> seviye` konumunda ve
dolgu kenar texel'ini taşıyor. Renklerin seviyeden seviyeye değişmesi
harmanın doğrusal ışıkta yapıldığını gösteriyor.

## Açık kapsam

GPU'nun hangi mip seviyesini seçtiği doğrulanmadı. Sondanın kamerası sabit ve
küpü büyütüyor, yani her karede seviye sıfır baskın. Seçim ancak küçültmeli bir
görüntüyle görülebilir ve o da [piksel toleransı](render-variability-measurement.md)
engeline takılı.

Renk tonlaması (tint) hâlâ yok: akasya yaprakları gri çıkıyor, çünkü biyom
rengi uygulanmıyor.
