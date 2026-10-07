# T84b — Dünya sınırının çizimi

**Amaç:** `WorldBorderRenderer`: sınırın dört yüzünün `forcefield.png` ile çizilmesi (uzaklığa göre alfa, durum rengi: durağan/küçülüyor/büyüyor, doku kayması), `BorderStatus` renkleri ve uyarı kırmızı örtüsü (`Hud`'un sınır uyarısı).
**Bağımlılık:** T84. **Boy:** M.

## Oku
- Java: `client/renderer/WorldBorderRenderer.java`, `client/renderer/rendertype/RenderTypes.java` (sınır boru hattı), `world/level/border/{WorldBorder,BorderStatus}.java`.
- Rust: `crates/voxel-world/src/border.rs`, `crates/voxel-render/src/{vulkan,weather}.rs` (şeffaf boru hattı örneği), `crates/voxel/src/play.rs`.

## Oracle
Gerçek `WorldBorderRenderer` köşe/UV/alfa kaydı (`border_quads` operation'ı).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`; canlıda sınıra bakılır (`tools/voxel.py shot`) bir kez.

## Belgeler
`docs/records/client-world-border-live-parity-conformance.md`.

## Devir notu
(boş)
