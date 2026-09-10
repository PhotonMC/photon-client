# P05.d — ScrollWheelHandler sözleşmesi

Bu dilim, mouse scroll offset'lerinin tam wheel adımlarına çevrilmesini ve hotbar seçim helper'ını kapsar. GLFW callback sırası, pencere odağı, GUI/player/spectator yönlendirmesi ve MouseHandler'ın frame içi hareket işleme akışı sonraki input/GUI/world dilimlerinde kalır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/ScrollWheelHandler.java:5-45` incelendi. Oracle, gerçek Java `ScrollWheelHandler` nesnesi ve static selection helper'ı üzerinde kontrollü f64 girdileri çalıştırır.

- Birikmiş eksen sıfır değilken yeni offset'in Java `Math.signum` sonucu farklıysa o eksenin birikimi sıfırlanır. Sonra offset eklenir; double değer Java int dönüşümüyle sıfıra doğru kesilir.
- Her iki tam wheel çıktısı da sıfırsa artık double değerler korunur. En az bir eksen tam adıma ulaştığında tam adımlar döndürülür ve ilgili artık değerlerden çıkarılır; diğer eksenin kesirli artığı aynı çağrıda korunur.
- Selection helper, wheel işaretini Java `Math.signum` ile int adıma çevirip mevcut seçimi bu adımdan çıkarır. Sonuç önce en az `-1` yapılır, sonra pozitif limit ile döngüsel aralığa taşınır.

Rust karşılığı `src/scroll.rs` içindeki `ScrollWheelHandler` tipidir. Java double-to-int dönüşümünün NaN/Infinity sonuçları ve i32 taşmalı selection arithmetic'i korunur; production callback ve player state entegrasyonu bu veri dilimine dahil değildir.

## JSONL sınırı

İstek `family: "math"`, `operation: "scroll_wheel"`, sıralı x/y scroll f64 bitleri ve wheel/current/limit selection sorgularını taşır. Sonuç her scroll çağrısının x/y tam adımını ve her selection sorgusunun giriş bitleri ile seçilen slotu taşır.
