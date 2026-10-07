# P16.u — Canlı sunucudan dünya çizimi — sonda kaydı

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Bu bir parity kaydı **değil**. Sonda ilk kez ağ tarafıyla çiziciyi birleştirdi;
kaydın amacı ne yapıldığını, hangi parçaların pinli olduğunu ve neyin hâlâ
eksik olduğunu yazmak.

## Ne yapıldı

`tools/render/scene.py serve` pinned sunucuyu `build` ile üretilmiş sahne
üzerinde çalıştırıp ayakta tutuyor. Dünya orada üretilmiyor; fikstür kopyalanıp
sunuluyor, yani sunulan dünya tam olarak `build`'in ürettiği.

Sonda `VOXEL_SERVER` ile o sunucuya katılıyor, yığınları bekliyor ve
oyuncunun çevresindeki bir kutuyu çiziyor. Köprünün tamamı şu zincir:

```
block_state_at(x,y,z) → durum numarası
  → block_registry::block_for_state  → blok + özellikler
  → blockstates/<blok>.json          → varyant
  → models/<model>.json              → çözülmüş model
  → build_atlas_for                  → tek atlas (mip'leriyle)
  → build_mesh                       → yüzler (cullface + UV kilidi + tonlama)
```

Zincirdeki her halka ayrı ayrı pinli:
[kayıt defteri](records/block-registry-conformance.md),
[blockstate](records/blockstate-conformance.md),
[model dosyası](records/model-file-parse-conformance.md),
[atlas](atlas-layout-and-mip-upload.md),
[mip](records/mipmap-conformance.md),
[UV kilidi](records/uv-lock-conformance.md),
[cullface](records/cullface-conformance.md),
[tonlama](records/block-tint-conformance.md).

## Ölçülen sonuç

Huş ormanı bir sahnede, 81x40x81 kutuda:

| Ölçüm | Değer |
| --- | --- |
| Sunucudan gelen yığın | 20 |
| Bölüm | 480 |
| Hava olmayan blok | 63834 |
| Kullanılan model | 27 |
| Atlas | 512x512, 5 mip seviyesi |
| Köşe | 56200 |
| Çözülemeyen blok | 0 |

Çim yeşil, huş gövdeleri doğru, kumsal ve şeker kamışı görünüyor.

## Eksikler — görüntüde doğrudan görünenler

- **Su çizilmiyor.** `minecraft:water` doğru çözülüyor ama modelinde eleman
  yok: referans sıvıları model yolundan değil ayrı bir sıvı çizicisinden
  çiziyor. Bu yüzden gölün yerinde delik kalıyor. Çözülemeyen blok sayısı
  sıfır olduğu hâlde bir delik olması bu yüzden.
- **Gökyüzü yok.** Arka plan sondanın temizleme rengi.
- **Işık yok.** Yüz başına sabit bir gölge var ve o bile referansın değil;
  blok ışığı, gökyüzü ışığı ve ambient occlusion hiç yok.
- **Yüz gizleme basit.** Komşu hava değilse yüz atılıyor. Gerçek karar
  `VoxelShape` örtüşmesine bakıyor, yani merdiven ya da camın yanındaki yüzler
  şu an yanlış atılabiliyor veya gereksiz çiziliyor.

## Eksikler — yapısal

- Kutu bir kez okunuyor; yığın güncellemesi, yeniden mesh, render mesafesi ve
  frustum ayıklama yok.
- Kamera sondanın kendi kamerası; referansın kamerası hâlâ pinli değil.
- Ağırlıklı varyant listelerinden ilki alınıyor; seçim referansın rastgele
  kaynağını istiyor.

## Koşma

```
python3 tools/render/scene.py build            # bir kez
python3 tools/render/scene.py serve --port 25599
VOXEL_SERVER=127.0.0.1:25599 \
VOXEL_ASSETS=.data/minecraft/assets/jar/assets \
VOXEL_REACH=40 VOXEL_HEIGHT=40 VOXEL_FLOOR=50 VOXEL_EYE=16 \
cargo run --offline -- window 854 480 3
```
