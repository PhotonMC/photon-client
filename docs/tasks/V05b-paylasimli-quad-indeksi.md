# V05b — Paylaşımlı opaque quad indeks tamponu

**Amaç:** Bölüm indeks belleğinin opaque payını (V05 ölçümüne göre ~%17) kaldırmak.
**Bağımlılık:** V05. **Boy:** M. **Öncelik:** düşük (yalnız bellek kazancı).

## Oku
- [V05 devir notu](V05-indexed-kompakt-vertex.md)
- `crates/photon-render/src/vulkan/{sections,memory,frame}.rs`, `world_mesh.rs` (indeks üretimi)

## Yapılacaklar
- `upload_section`: köşeleri opaque quad'lar önce, yarı saydamlar sonra olacak şekilde yeniden sırala (çizim sırası indeks sırasıyla belirlendiğinden görüntü değişmez); opaque indeksler `4k + {0,1,2,0,2,3}` olur.
- Paylaşımlı, büyüyen tek indeks tamponu; opaque çizim bunu bağlar. Yarı saydam indeksler bölüm başına kalır (kare başına sıralanır).
- Kalıp varsayımı yükleme sırasında doğrulanır; uymayan mesh için eski yol (kendi indeksleri) tutulur.

## Kabul
`python3 tools/photon.py check` → `CHECK OK`; `bench` `section_memory.index_bytes` düşer, GPU sahne süresi aynı aralıkta; bir `shot` karesi doğru; `--validation` 0 hata.

## Kapsam dışı
Köşe nicemleme (kayıplı), indirect çizim.

## Belgeler
Bu görev dosyasının devir notu.

## Devir notu
(boş)
