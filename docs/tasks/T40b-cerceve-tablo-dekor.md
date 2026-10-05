# T40b — Eşya çerçevesi, tablo, bağ düğümü, kristal, TNT, düşen blok, yıldırım, display'ler

**Amaç:** Dünyaya yerleştirilen kalan entity'ler: eşya çerçevesi (içerik, dönüş, harita), tablo (`PaintingVariant`, atlas), bağ düğümü ve `SetEntityLink`, ender kristali, TNT (yanıp sönme), düşen blok, yıldırım, `display` entity'leri.
**Bağımlılık:** T40. **Boy:** L (dallara bölünebilir: çerçeve+tablo, bağ, kristal/TNT/düşen blok, display).

## Oku
- Java: `client/renderer/entity/{ItemFrameRenderer,PaintingRenderer,LeashKnotRenderer,EndCrystalRenderer,TntRenderer,FallingBlockRenderer,LightningBoltRenderer,DisplayRenderer}.java`, `world/entity/decoration/{ItemFrame,Painting,HangingEntity}.java`, `client/renderer/state/` ilgili durumlar.
- Rust: `crates/voxel/src/armor_stands.rs` (yeni entity çizim kalıbı), `crates/voxel-world/src/armor_stand.rs` (veri birleştirme kalıbı), `voxel-render/src/item_entity.rs` (eşya çizimi).

## Yapılacaklar
1. Çerçeve: yön verisi (`Direction`), içerik eşyası ve dönüşü; çerçeve bloğu modeli (blok modeli çizimi).
2. Tablo: varyant kaydı (registry), atlas dokusu.
3. Bağ, kristal, TNT, düşen blok, yıldırım, display entity'leri ayrı dilimler.

## Oracle
`armor_stand_model` kalıbı: gerçek model sınıfı/yöntemi + gerçek durum; veri indeksleri `variant-fields`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon`.

## Kapsam dışı
Blok entity çizicileri (PLAN-M8).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Bölündü: kristal T37h3'te yapıldı, TNT minecart'ı T38d'de; kalan dilimler T40c (TNT), T40d (düşen blok), T40e (yıldırım), T40f (bağ), T40g (çerçeve), T40h (tablo), T40i (display).
