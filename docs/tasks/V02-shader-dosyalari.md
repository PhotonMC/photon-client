# V02 — Shader dosyaları, build-time SPIR-V, pipeline cache

**Amaç:** GLSL'i Rust dizgelerinden çıkarıp dosyalara almak, açılışta derlemeyi kaldırmak.
**Bağımlılık:** V01. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §5 (5.7)
- `vulkan.rs:95-905` (shader sabitleri), `:2657` (`shaderc`)

## Yapılacaklar
- `crates/photon-render/shaders/*.{vert,frag}` (+ ortak `include`); `build.rs` SPIR-V üretir, `include_bytes!` ile gömülür; `shaderc` yalnız build bağımlılığı.
- Diskte kalıcı `VkPipelineCache` (kullanıcı önbellek dizini; bozuk/uyumsuzsa yok sayılır).
- Shader derleme testi build'e taşınır.

## Kabul
Davranış değişmez: `python3 tools/photon.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit. Açılış süresi V00 aracıyla ölçülüp devir notuna yazılır.

## Kapsam dışı
Shader mantık değişikliği.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
- GLSL `crates/photon-render/shaders/` altında 31 dosya (`*.vert`/`*.frag`) + iki ortak `#include` (`push.glsl`, `lightmap.glsl`; on köşe gölgelendiricisinin tekrarlayan başı). Dosyalar eski sabitlerin makro açılmış hâlinden üretildi.
- `build.rs` shaderc ile SPIR-V üretir (`OUT_DIR`), `shaders.rs` `include_bytes!` ile gömer (`Shader::words`). `shaderc` yalnız `[build-dependencies]`; çalışma zamanında derleme yok. Derleme testi build'e taşındı (bozuk GLSL build'i düşürür); `Cargo.toml` `include` listesi `build.rs` ve `shaders/**` ile genişledi.
- **Kanıt:** 31 gömülü SPIR-V'nin hepsi eski sabit metninin shaderc derlemesiyle bayt bayt eşit (geçici test, silindi). Gölgelendirici mantığı değişmedi.
- Pipeline cache: `VulkanBackend::use_pipeline_cache(yol)` / `save_pipeline_cache()`; dosya `<oyun dizini>/pipeline.cache` (`PHOTON_GAME_DIR`, varsayılan `.data/client`). Başlık (vendor, aygıt, UUID) bu aygıta uymuyorsa veya dosya kısaysa yok sayılır (birim testli); yazma geçici dosya + yeniden adlandırma, hata `warn!`.
- **Açılış (22 boru hattı, Apple M1, 3 koşu):** önce 1.13–1.16 sn (her boru hattı için çalışma zamanı shaderc); şimdi ilk koşu (boş önbellek) 35 ms, sonraki 19–24 ms (önbellek 101 KB). Ölçüm `log::info!("pipelines built in …")` satırı ile (`PHOTON_LOG=info`, `shot` günlüğünde).
- Sapma: iki kare piksel piksel karşılaştırılmadı; SPIR-V eşitliği aynı gölgelendiriciyi kanıtlar, ayrı çizim karşılaştırması anlamsız kalırdı.
