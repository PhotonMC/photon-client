# P01.a — Mth binarySearch doğrulaması

`src/math.rs` içindeki `mth_binary_search`, Java `Mth.binarySearch(int, int, IntPredicate)` yolunu JSONL adapter'a bağlar. `ge` ve `gt` monoton predicate'leriyle ilk true konumu, üst uç ve signed i32 taşma sonuçları Java oracle'ın `value` sonucu ile karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Boş ve tek elemanlı aralıklar, normal ve negatif aralıklar, ters aralıklar, signed i32 sınırları ve wrapping uzunluk/orta değer yolları için 24 özgün senaryo hazırlandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2076 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2076/2076 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2578/2578 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `e84d285a8cd335bd1ba5cd88d076e7cb38315013b991057804c6f1b4598f1c80`. Birleşik referans kimliği: `9c94496c39504f2bd146d331630516f020e05259c67c0bca659d1f1ca92c74a8`.

Adapter hashleri: debug `6eb1e772a2e06c20b8d0c09ba0b2a11b5f1df7a671ee8d1513127bc318d143ef`, release `318c5ef22a8d56277aa19f9da9f5becca9f575904eba79756b959bdc8c993fe8`.

Yerel raporlar: `compare-k2v65l83` ve `compare-9uf1gox2` (math debug/release), `compare-ptlm9ptr` ve `compare-86z18x_k` (birleşik debug/release), `verify-9dtf6rea` (math) ve `verify-31k8bjcu` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki `Mth.binarySearch` yordamı, `ge`/`gt` predicate biçimleri ve bu dilimin 24 signed i32 girdisiyle sınırlıdır. Keyfi predicate bileşimleri, `rayIntersectsAABB`, `atan2`, ters karekök yardımcıları ve renk dönüşümleri ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
