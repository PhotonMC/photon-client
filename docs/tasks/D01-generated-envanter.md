# D01 — Üretilmiş tabloların envanteri ve sınıflandırması

**Amaç:** Her `generated/` dosyasını Java kaynağına ve hedef biçimine eşlemek.
**Bağımlılık:** –. **Boy:** S.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §6
- `crates/voxel-data/src/generated/`, `tools/generate/*.py`

## Yapılacaklar
- Tablo: dosya → Java sınıfı/JSON yolu → sınıf (a) elle Rust modülü, (b) çalışma anında JAR `data/` veya `RegistryData`, (c) hesaplanır → hedef Rust yolu → hangi parity karşılaştırması doğrular.
- Elle modül düzeni kararı (ör. `voxel-data/src/model/<java_sınıfı>.rs::create_body_layer`).
- Tablo `docs/records/generated-inventory.md`'e yazılır.

## Kabul
Envanter 28 dosyanın hepsini kapsar.

## Kapsam dışı
Kod taşıma.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
