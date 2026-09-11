# P06/P07 — Protocol phase ve packet flow metadata doğrulaması

Java oracle, `ConnectionProtocol.valueOf` ve `PacketFlow.valueOf` üzerinden enum kimliği, adını, ordinal'ini ve flow opposite sonucunu üretir. Rust adapter aynı yedi JSONL senaryosunda bu alanları karşılaştırır.

Beş connection phase ve iki packet flow vakası doğrulandı. Java capture kimliği `5ac68b2e452bc40a6b75ba67d95eaa46907c4a53563b4c48411073bc7c927d6d`; `verify-jwziq_96`, debug `compare-jymsi58c` ve release `compare-p8qrr9av` raporlarının tamamında fark yoktur.

Bu slice sonrası tam corpus 3755 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=459`. Tam capture kimliği `ec222d47eed118990410a510dcc46dccfcaa1da2e7ceabc16fb157259fbd7824`; debug `compare-eq24bcow` ve release `compare-23se7rk8` raporları geçti.

Bu sonuç phase/flow metadata eşitliğini doğrular. Packet id dispatch, compression/encryption pipeline, listener/state transition ve gerçek vanilla 26.2 bağlantısı sonraki P06/P07 entegrasyon kapsamındadır.
