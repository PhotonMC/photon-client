# V02 — Shader dosyaları, build-time SPIR-V, pipeline cache

**Amaç:** GLSL'i Rust dizgelerinden çıkarıp dosyalara almak, açılışta derlemeyi kaldırmak.
**Bağımlılık:** V01. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §5 (5.7)
- `vulkan.rs:95-905` (shader sabitleri), `:2657` (`shaderc`)

## Yapılacaklar
- `crates/voxel-render/shaders/*.{vert,frag}` (+ ortak `include`); `build.rs` SPIR-V üretir, `include_bytes!` ile gömülür; `shaderc` yalnız build bağımlılığı.
- Diskte kalıcı `VkPipelineCache` (kullanıcı önbellek dizini; bozuk/uyumsuzsa yok sayılır).
- Shader derleme testi build'e taşınır.

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit. Açılış süresi V00 aracıyla ölçülüp devir notuna yazılır.

## Kapsam dışı
Shader mantık değişikliği.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
