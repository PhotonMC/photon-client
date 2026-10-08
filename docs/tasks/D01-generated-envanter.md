# D01 — Üretilmiş tabloların envanteri ve sınıflandırması

**Amaç:** Her `generated/` dosyasını Java kaynağına ve hedef biçimine eşlemek.
**Bağımlılık:** –. **Boy:** S.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §6
- `crates/photon-data/src/generated/`, `tools/generate/*.py`

## Yapılacaklar
- Tablo: dosya → Java sınıfı/JSON yolu → sınıf (a) elle Rust modülü, (b) çalışma anında JAR `data/` veya `RegistryData`, (c) hesaplanır → hedef Rust yolu → hangi parity karşılaştırması doğrular.
- Elle modül düzeni kararı (ör. `photon-data/src/model/<java_sınıfı>.rs::create_body_layer`).
- Tablo `docs/records/generated-inventory.md`'e yazılır.

## Kabul
Envanter 28 dosyanın hepsini kapsar.

## Kapsam dışı
Kod taşıma.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
- Envanter `docs/records/generated-inventory.md`'de: 28 dosyanın 28'i kapsandı (22 elle modül (a), 4 çalışma anı verisi (b), 1 karma (a+b), üç `mod.rs` D07'de birleşir; (c) hesaplanan tablo yok). Her dosya Java kaynağına, üreticiye ve hedef yola eşlendi; doğrulama her parity koşusundaki tam karşılaştırma.
- **Karar bekleyen nokta (D02–D07'yi kilitliyor):** inceleme §8 ve AGENTS.md'deki "tablo oracle'dan yakalanır, üretilmiş dosyaya yazılır" kuralı üretilmiş tabloları şart koşuyor; D07 üreticiyi ve kural metnini kaldırır. Proje sahibinin açık kararı olmadan D02–D07 başlatılmadı.
