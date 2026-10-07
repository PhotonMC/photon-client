# V05 — Indexed quad ve kompakt köşe formatı

**Amaç:** Köşe sayısını 6→4'e, köşe boyutunu 48 B'tan küçüğe indirmek.
**Bağımlılık:** V03. **Boy:** L.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §5 (5.8)
- `crates/voxel-render/src/scene.rs:25`, `world_mesh.rs`, Java `DefaultVertexFormat`

## Yapılacaklar
- Paylaşılan quad indeks tamponu; bütün quad çizimleri `cmd_draw_indexed`.
- Bölüm köşesi: konum (bölüm yerel, kesirli tamsayı veya f32), paketli renk (unorm8×4), UV (unorm16 veya f32), ışık (u8×2). Java formatıyla hizalı.
- Shader'lar yeni formata uyarlanır.

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit. Bench: GPU bölüm pass süresi ve bellek kullanımı düşer.

## Kapsam dışı
Indirect çizim.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
- **Ölçüldü, uygulanmadı.** `bench` raporuna `section_memory` eklendi (`VulkanBackend::section_memory`): Apple M1, görüş 12, 637 chunk, 1345 dolu bölüm: köşe 156.4 MB, indeks 34.6 MB (%18), arena 201 MB.
- Bölüm köşesi zaten Java'nın `BLOCK` biçiminde (konum f32×3, renk u8×4, uv f32×2, ışık u8×2 + 2 bayt dolgu = 28 B) ve **zaten indeksli** (`cmd_draw_indexed`); raporun 5.8 bulgusu (48 B, indekssiz) yalnız entity/GUI köşeleri için doğru ve bunlar küçük (~60 entity, GUI birkaç yüz köşe).
- Köşeyi küçültmek (konumu/uv'yi 16 bite nicemlemek) kayıplıdır ve "piksel eşit" kabulüyle çelişir; dolgu baytını atmak (26 B) vertex stride hizasını bozar. Paylaşımlı quad indeks tamponu kayıpsızdır ama opaque ve yarı saydam quad'lar köşe dizisinde iç içe olduğundan opaque indeksler `4k + {0,1,2,0,2,3}` kalıbında değildir: önce köşeleri opaque/yarı saydam diye yeniden sıralamak (yükleme sırasında kopya) gerekir. Kazanç yalnız bellek (~%17 ≈ 34 MB bu sahnede), GPU süresi değil (M1'de pass dökümü yok, sahne ~5.7 ms ve parça gölgelendirici baskın görünüyor).
- → V05b (düşük öncelik): paylaşımlı opaque quad indeks tamponu + yükleme sırasında köşe yeniden sıralaması; entity/GUI quad'ları için indeksli çizim yalnız emitter'lar ortak bir quad API'sine geçince anlamlı.
