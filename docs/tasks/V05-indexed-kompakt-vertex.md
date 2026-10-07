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
(boş)
