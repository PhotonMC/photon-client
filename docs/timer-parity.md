# P02.a — Rust zamanlayıcısı

`src/timer.rs`, Java 26.2 istemcisinin tick birikimi ve kısmi tick davranışını uygular. `examples/behavior_adapter.rs`, mevcut `timer/trace` girdilerini bu kütüphaneye bağlar. Üretim modülüne bağımlılık veya gerçek sistem saati erişimi eklenmedi.

## Kaynak ve kapsam

Referans kaynak: `.data/minecraft/source/net/minecraft/client/DeltaTracker.java`, `Timer` sınıfı; constructor, zaman ilerletme, pause/freeze ve getter metotları yeniden incelendi. Orijinal client SHA-1: `2dc72797acbc1b63fc16a11c4ac393605f453754`. Beklenen sonuçlar, daha önce yazılmış `TimerOracle.java` üzerinden orijinal JAR'dan alınır. Senaryolar ve Java adaptörü bu Rust diliminde değiştirilmedi.

Korunan gözlenebilir davranışlar:

- Oyun zamanı ve gerçek zaman ayrı i64 milisaniye saatleri kullanır. Fark önce tamsayı olarak hesaplanır, sonra f32'ye çevrilir; büyük mutlak saat değerleri ayrı ayrı float'a dönüştürülmez.
- Oyun tick süresi değişebilir; gerçek zamanın tick süresi constructor'daki nominal değere bağlı kalır. Tick sayısı kesilerek i32'ye dönüşür; artan kesir sonraki çağrıya taşınır.
- Pause ilk geçişte artan kesri saklar. Tekrarlı pause bu kaydı değiştirmez; unpause saklanan kesri geri yükler. Pause/freeze, zaman ilerletme metotlarına erken dönüş eklemez.
- Freeze durumunda, freeze'i dikkate alan kısmi tick sorgusu 1 verir. Freeze'i yok sayan sorgu pause kaydını veya güncel kesri kullanır.
- Gerçek zaman deltasının 7 tick'i aşması getter'da 0,5 sonucuna yol açar. Eşitlik sınırı ayrıca senaryoda bulunur; genel bir clamp uygulanmaz.

Rust `Timer::new(ticks_per_second, current_ms)` constructor'ını ve `advance_game_time(current_ms, target_ms_per_tick)` çağrısını sunar. Java'nın sakladığı sağlayıcının döndürdüğü hedef süre Rust'ta her çağrıda açık girdi olarak verilir. Callback saklama/çağırma yan etkileri bu API'nin kapsamı değildir; sunucu tick-rate sağlayıcısının bağlanması sonraki entegrasyon işidir. Adaptörün `target` adımı yalnızca sonraki çağrılarda verilecek girdiyi değiştirir.

Diğer metotlar: `advance_real_time`, `update_pause_state`, `update_frozen_state`, `game_time_delta_ticks`, `game_time_delta_partial_tick`, `realtime_delta_ticks`. İç durum dışarıdan değiştirilemez. Pause kaydı Rust'ta `Option<f32>` ile temsil edilir.

Bu dilim oyun döngüsü, frame başına 10 tick sınırı, paket/input sıralaması veya karakter hareketini uygulamaz. Timer'ın ürettiği tick sayısına oyun döngüsü sınırını burada uygulamak referans davranışını değiştirirdi. Sıfır/geçersiz tick hızları ve i64 taşma uçları mevcut 22 senaryonun doğrulama kapsamı değildir; bu girdiler için test sonucu iddiası yoktur.

## Doğrulama sonucu

P02.a dilimi tamamlandığında Java 25.0.4.1 ve Rust 1.98.1, Linux x86_64 üzerinde alınan sonuçlar:

| Kontrol | Sonuç |
| --- | --- |
| 22 timer + 144 wire Java senaryosu | İki bağımsız JVM çalıştırması eşleşti |
| Rust debug adaptörü | 166/166 eşleşti |
| Rust release adaptörü | 166/166 eşleşti |
| Timer karşılaştırması | Başlangıçta ve her adımda tüm delta/kısmi tick alanlarının f32 bitleri; oyun adımlarında dönen tick sayısı |
| Adaptör hata yolları | O aşamada desteklenmeyen math ailesi, bozuk float bitleri, i64 dışı saat, yanlış steps/boolean türü ve bilinmeyen action reddedildi |
| Biçim ve Clippy | `cargo fmt --all --check`; `cargo clippy --locked --offline --all-targets -- -D warnings` geçti |
| Paket listesi | `.data/`, orijinal içerikler ve test adaptörü dışarıda |

Birleşik referans kimliği: `8164dedd5ef47b3fef92986abeb9c6aa24acbbdb458bbf5acf4467f3e7fe31001`.

Yerel kanıt dosyaları:

- `.data/runs/behavior/capture-z5s08vpz/report.json`
- `.data/runs/behavior/compare-vm1fv7ai/report.json` (debug)
- `.data/runs/behavior/compare-20w25b3h/report.json` (release)

Karşılaştırılan kapsam P02.a'nın mevcut 22 senaryosu ve önceki P03.a'nın 144 senaryosudur. Bu sonuç P02 bölümünün tamamlandığı veya karakter hareketinin eşleştiği anlamına gelmez. Sonraki P01.a dilimiyle 180 matematik senaryosu da Rust ile eşleşti; [güncel 346 senaryoluk doğrulama](math-parity.md).

## Tekrar çalıştırma

Depo kökünden, yerel referans dosyaları ve Cargo bağımlılık önbelleği mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py capture --family timer --family wire
python3 tools/behavior/run.py compare --family timer --family wire --adapter target/debug/examples/behavior_adapter
```

Mevcut capture değiştirilmez. Yalnızca Java referansını yeniden doğrulamak için aynı aile seçimiyle `verify` kullanılabilir. Release için derlemeye `--release` eklenir ve adaptör yolu `target/release/examples/behavior_adapter` olur. Yukarıdaki aile seçimi timer/wire kapsamını çalıştırır.

P01.a Vec2 ve sayısal yardımcıları tamamlandı. P03.b UTF-8 de 141 senaryoda doğrulandı; [kanıt](utf8-parity.md). Kullanıcının hareket sorusu bir öncelik değişikliği değildir; kaynak ve bağımlılık sırasına göre ilerlenir.
