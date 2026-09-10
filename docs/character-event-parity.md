# P05.b — CharacterEvent doğrulaması

`src/input_event.rs`, Java Edition 26.2 / protokol 776 / Java 25 kaynak davranışına göre `CharacterEvent` codepoint dönüşümünü ve chat karakteri filtresini uygular. Java oracle gerçek `CharacterEvent` metodlarını çalıştırır; beklenen sonuç Rust adapter'ından türetilmez.

Tam corpus referansı `cee0b0cfe4a88ad295837f974a16309b02fdc6bc8a591837f26ddd183fc553e0` olarak kaydedildi. Debug adapter karşılaştırması `compare-qg2y8ke2`, release adapter karşılaştırması `compare-4k16jioa` raporunda fark üretmedi.

Senaryolar NUL, boşluk, ASCII, satır sonu, DEL, section sign, BMP accented karakter, surrogate codepoint ve supplementary karakter dönüşümlerini; izin filtresinin sınırlarını kapsar. Geçersiz codepoint exception yolu, callback'in gerçek GLFW kaynakları ve metin widget'ı davranışı bu dilimin kapsamı dışındadır.

Tekrarlanabilir doğrulama komutları:

```text
python3 tools/behavior/run.py verify
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py compare --adapter target/release/examples/behavior_adapter
python3 tools/behavior/test_runner.py
cargo test --locked --offline --all-targets
cargo clippy --locked --offline --all-targets --all-features -- -D warnings
```
