use super::enum_combo;
use egui_kittest::kittest::{NodeT, Queryable};

#[test]
fn repeated_labels_keep_distinct_ids_and_independent_values() {
    let mut harness = egui_kittest::HarnessBuilder::<[String; 2]>::default()
        .with_size(egui::vec2(800.0, 300.0))
        .build_ui_state(
            |ui, values: &mut [String; 2]| {
                enum_combo(
                    ui,
                    "finding_confidence",
                    "Confiança",
                    &mut values[0],
                    &["LOW", "HIGH"],
                );
                enum_combo(
                    ui,
                    "cause_confidence",
                    "Confiança",
                    &mut values[1],
                    &["LOW", "HIGH"],
                );
            },
            [String::new(), String::new()],
        );
    harness.run();

    let combos: Vec<_> = harness
        .get_all_by_role_and_label(egui::accesskit::Role::ComboBox, "Confiança")
        .collect();
    assert_eq!(
        combos.len(),
        2,
        "os dois seletores devem manter o mesmo nome acessível"
    );
    assert_ne!(
        combos[0].accesskit_node().id(),
        combos[1].accesskit_node().id(),
        "salts distintos não podem compartilhar o ID persistente"
    );

    combos[1].click();
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "HIGH")
        .click();
    harness.run();

    let values = harness.state();
    assert_eq!(values[0], "", "o primeiro seletor não pode ser alterado");
    assert_eq!(values[1], "HIGH", "o segundo seletor deve responder");
}
