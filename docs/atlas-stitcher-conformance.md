# P16.a — Texture atlas paketleyici — [client-parity] birleşik kayıt

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik girdi pinned JAR'daki
`net.minecraft.client.renderer.texture.Stitcher` sınıfına ve üretim
kütüphanesindeki `voxel_client::render::atlas` modülüne verilir, çıktılar
birebir karşılaştırılır. GPU, asset veya pencere gerektirmez.

Bu, render bölümünün ilk **veri** dilimidir. Bitmiş istemcideki her sprite UV'si
bu tamsayılardan türediği için layout, tek texture yüklenmeden önce sabitlenir.
[P15.a.2 ölçümündeki](render-variability-measurement.md) piksel toleransı
sorunu bu dilimi etkilemez: burada karşılaştırılan şey rasterize edilmiş
görüntü değil, paketleyicinin ürettiği tamsayı yerleşimdir.

## Java kaynak incelemesi

`Stitcher`, sprite'ları önce azalan yüksekliğe, sonra azalan genişliğe, sonra
sprite adına göre sıralar. Her sprite `registerSprite` içinde iki yanına
`padding` eklenerek kaydedilir ve `smallestFittingMinTexel` ile mip seviyesinin
tam katına yuvarlanır. `padding` değeri `1 << mipLevel << clamp(anisotropyBit - 1, 0, 4)`'tür.

Yerleştirme, mevcut bölgeleri sırayla dener; hiçbirine sığmazsa atlas
`smallestEncompassingPowerOfTwo` ile bir eksende büyütülür. Hangi eksende
büyüyeceği, yalnızca birinin büyüyebildiği durumda o eksen, ikisi de veya
hiçbiri büyüyecekse mevcut boyutların karşılaştırmasıyla seçilir.

Bir bölge kendisinden küçük bir sprite alırsa bir kez alt bölgelere ayrılır.
Kesme yönü, artan kenarların uzun olanı ile özgün kenarların uzun olanı
karşılaştırılarak seçilir.

`Identifier.compareTo` **önce path'i, sonra namespace'i** karşılaştırır. Ters
sıra makul görünür ve yanlıştır: eşit boyutlu sprite'ların sırasını değiştirir
ve ardından gelen bütün UV'leri kaydırır. Bu kayıt o önceliği ayrı bir satırla
sabitler.

## Girdiler

`atlas/stitch` 19 parametrik satır taşır: tek sprite, eşit boyutlu sekizli,
yalnızca adın ayırt ettiği durumlar, alt bölmeye zorlayan karışık boyutlar,
mip seviyesiyle yukarı yuvarlanan tek sayılı boyutlar, mip ve anisotropy'nin
padding'i büyüttüğü durumlar, dar/kısa atlas sınırları ve hiç sığmayan setler.

## Sonuçlar

Java ve Rust her satırda aynı atlas boyutunu, aynı padding'i ve aynı sprite
yerleşimlerini aynı sırada verdi. Ayırt edici satırlar:

| Girdi | Atlas | Padding | Ne gösteriyor |
| --- | --- | ---: | --- |
| `single` (16x16) | `32x32` | `1` | Padding 18'e çıkarıyor, ikinin kuvveti 32'ye yuvarlıyor |
| `exact-fit` (64x64 sprite, 64x64 atlas) | yerleştirilemedi | — | Padding 66 yapıyor; tam sığan sprite sığmıyor |
| `anisotropy-1` | `256x128` | `1` | `clamp(0,0,4)` ile anisotropy 0 ile aynı |
| `anisotropy-clamped-9` | `256x128` | `16` | Clamp üst sınırı `4` doğrulandı |
| `mip-2-anisotropy-2` | `256x128` | `8` | İki kaydırmanın birlikte etkisi |
| `narrow-atlas` (max 64 geniş) | yerleştirilemedi | — | Tek eksende sınır, büyüme reddi |
| `path-before-namespace` | `64x64` | `1` | `zzz:block/a` `(0,0)`, `aaa:block/b` `(32,0)` |

Son satır önceliği kanıtlar: namespace önce karşılaştırılsaydı `aaa:block/b`
başa geçerdi.

Koşu: `python3 tools/behavior/run.py capture --family atlas` ardından
`compare --family atlas --adapter target/release/examples/behavior_adapter`
→ `parity_passed`.

## Açık kapsam

Bu kayıt yalnızca paketleme algoritmasıdır. `SpriteLoader`'ın gerçek asset
dosyalarından sprite içeriğini okuması, PNG çözme, mipmap üretimi, animasyonlu
sprite'lar, `MissingTextureAtlasSprite` yolu ve atlas'ın GPU'ya yüklenmesi
açıktır. Sprite adlarının gerçek atlas kaynaklarından toplanması `P04` asset
içerik hattına bağlıdır.
