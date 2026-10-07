# V00 — Performans ölçümü ve bench

**Amaç:** Optimizasyonlardan önce tekrarlanabilir kare süresi ve GPU pass süreleri elde etmek.
**Bağımlılık:** –. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §5 (5.6)
- `crates/voxel/src/{play,script,profile}.rs`, `crates/voxel-render/src/vulkan.rs` (`render_frame`)

## Yapılacaklar
- Kare süresi istatistiği: p50/p95/p99, ortalama FPS; CPU tarafında tik, mesh, extract, kayıt, submit süreleri.
- GPU zaman damgası sorguları (`vkCmdWriteTimestamp`) pass başına: gökyüzü, bölümler, entity, şeffaf, GUI.
- `python3 tools/voxel.py bench`: sabit betik, sabit görüş mesafesi, sabit konum; JSON rapor yerelde.
- Mevcut değerler (temel çizgi) bu görevin devir notuna yazılır; sonraki V görevleri karşılaştırır.

## Kabul
Aynı makinede ardışık iki koşu arasında p50 farkı < %5; temel çizgi kayıtlı.

## Kapsam dışı
Herhangi bir optimizasyon.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
