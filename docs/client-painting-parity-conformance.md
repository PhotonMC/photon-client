# T40h — Tablo — [client-parity]

Bu kayıt `painting` entity'sinin (`Painting`, `PaintingRenderer`) çizimini kapsar. Kanıt sınıfı `client-parity`'dir: tablo çeşitleri (ad, doku, boyut) gerçek kayıttan tablo olarak alınır ve her koşuda karşılaştırılır; geometri gerçek `renderPainting`'in vertex tüketicisine verdiklerine karşı karşılaştırılır. Asılma konumu ve blok ışığı oracle'sızdır (aşağıda). Canlı gözlem `server-conformance`'tır.

## Java kaynak incelemesi

- `PaintingRenderer.submit`: `Y(180 − yön₂ᴅ·90)`; her blok için bir dilim: ön yüz (çeşidin dokusu, `1/genişlik`, `1/yükseklik` dilimleri), arka yüz (`back` dokusu), üst/alt kenar `v` boyunca `1/16`, yan kenar `u` boyunca `1/16`; derinlik `±0,03125`. Her blok kendi ışığıyla çizilir (önündeki bloğun ışığı: yöne göre `x` veya `z` kayar).
- Çeşit: `Painting.DATA_PAINTING_VARIANT_ID` (9; `HangingEntity` yönü 8 almıştır), `Holder` olarak `kayıt kimliği + 1` (0 = tam gönderilen). Yön: `AddEntity` verisi (`get3DDataValue`), konum: sunucunun yolladığı blok.
- `BlockAttachedEntity.setPos` + `Painting.calculateBoundingBox`: entity duvara `0,46875` yaklaşır; çift boyutlu tablo yarım blok sola/yukarı kayar; merkez `AABB.ofSize(...)`'ın merkezidir.

## Kapsam

- `entity/painting-variants` 51 satır (kayıttan; `tools/generate/painting_variants.py` ile üretilir, sürüm değişince yenilenir); `entity/painting-geometry` 10 satır (1×1 dört yön, geniş, uzun, 2×2, 4×3, 3×4, uzak konum; sprite köşeleri kasten farklı).
- Rust: `voxel-render::painting` (`painting_center`, `painting_pose`, `painting_vertices`), `voxel-data` çeşit tablosu, oyunda `paintings.rs`.
- Yan düzeltme: bağ düğümünün konumu da artık sunucunun yolladığı bloktan türetilir (`block + (0,5; 0,375; 0,5)`), önceki T40f kaydında ham paket konumu kullanılıyordu.

## Sapmalar ve açık kalanlar

- Asılma konumu (`painting_center`) tablo boyutundan **render anında** türetilir; entity'nin kendi konumu ve vuruş kutusu hâlâ blok köşesidir (seçim/hedefleme tablo için doğru değil). Konum için oracle yok.
- Blok ışığı kuralı kaynaktan yazıldı, oracle'sız; ışık yayan bloklar (`getLightEmission`) hesaba katılmaz.
- `entitySolidZOffsetForward` derinlik kaydırması yok (duvarla eşdüzlem yüzlerde titreşim olabilir).
- Boyutu/dokusu bilinmeyen çeşit çizilmez; tam gönderilen (kayıtta olmayan) çeşit okunmaz.
