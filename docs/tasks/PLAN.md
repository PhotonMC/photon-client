# PLAN görevleri: bir kilometre taşını görevlere bölmek

PLAN-M4, PLAN-M7, PLAN-M8, PLAN-M9 (pano tablosu) kod yazmaz; yalnız sıradaki işi bağımsız, tek oturumluk görev dosyalarına böler. Uzak işler bu yüzden ayrıntısız bekler (bayatlamasın).

1. [`remaining-work.md`](../remaining-work.md) içinde ilgili bölümü (M4 §4, M7 §7, M8 §8, M9 §9) ve [roadmap](../roadmap.md) satırlarını oku. Başka belgeyi gerekmedikçe okuma.
2. Java kaynak ağacını `find`/`grep` ile tara (örn. M4 için `.data/minecraft/source/net/minecraft/client/renderer/entity/*Renderer.java`, `client/model/**`); aileleri grupla (aynı model/renderer yolunu paylaşanlar **tek görev**, tablo satırıdır — AGENTS.md "varyant değil dal").
3. Her grup için `docs/tasks/T<kimlik>-<ad>.md` yaz (şablon: `README.md`): kesin Java yolları, Rust çapaları (`grep` ile doğrula), oracle adımı, kabul. Görev ≤ bir oturum; değilse böl.
4. `README.md` tablosuna satırlar ekle (kimlik T27'den itibaren, bağımlılıklarıyla), PLAN satırını `bitti` yap.
5. PLAN-M7 (ses) yeni bağımlılık (ses çıkışı, Ogg/Vorbis çözücü) gerektirir: **kullanıcıya sormadan görev açma**; seçenekleri ve önerini `AskUserQuestion` ile sun, karar kaydını `docs/render-backend-decision.md` biçiminde yeni bir belgeye yaz.
6. Tek commit: `docs: <taş> görevlere bölündü`.
