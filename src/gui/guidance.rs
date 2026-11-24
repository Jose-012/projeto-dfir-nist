//! Orientação compartilhada para estados vazios, com uma única revelação curta.

use super::icons::{AppIcon, paint_icon};
use std::time::Duration;

pub(crate) struct GuidanceAction {
    pub label: &'static str,
    pub hover: &'static str,
}

/// Revela um painel uma única vez, usando a duração já resolvida pelo tema.
/// O seed evita o primeiro `true` imediato do egui; duração zero não inicia
/// nenhuma animação e implementa Reduced Motion sem estado intermediário.
fn reveal_progress(context: &egui::Context, id: egui::Id, duration: Duration) -> f32 {
    if duration.is_zero() {
        return 1.0;
    }
    let initialized = context.data(|data| data.get_temp::<bool>(id).unwrap_or(false));
    if !initialized {
        context.animate_bool_with_time(id, false, 0.0);
        context.data_mut(|data| data.insert_temp(id, true));
    }
    context
        .animate_bool_with_time(id, true, duration.as_secs_f32())
        .clamp(0.0, 1.0)
}

/// Painel de próximos passos para um estado vazio. Retorna `true` quando a
/// ação opcional foi clicada; dados e comandos continuam pertencendo à tela.
pub(crate) fn guidance_panel(
    ui: &mut egui::Ui,
    id_salt: &'static str,
    icon: AppIcon,
    title: &'static str,
    body: &'static str,
    steps: &[(&'static str, &'static str)],
    action: Option<GuidanceAction>,
) -> bool {
    let duration = Duration::from_secs_f32(ui.style().animation_time.max(0.0));
    let reveal_id = ui.scope_id().with((id_salt, "first-reveal"));
    let progress = reveal_progress(ui.ctx(), reveal_id, duration);
    let mut clicked = false;
    ui.scope(|ui| {
        ui.set_opacity(progress);
        egui::Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .corner_radius(ui.visuals().widgets.noninteractive.corner_radius)
            .inner_margin(egui::Margin::symmetric(16, 14))
            .outer_margin(egui::Margin::symmetric(0, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::hover());
                    paint_icon(ui.painter(), rect, icon, ui.visuals().hyperlink_color);
                    ui.heading(title);
                });
                ui.add_space(6.0);
                ui.label(body);
                ui.add_space(10.0);
                for (index, (step_title, description)) in steps.iter().enumerate() {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{}. {step_title}", index + 1))
                                .strong()
                                .color(ui.visuals().hyperlink_color),
                        );
                        ui.label(egui::RichText::new(*description).weak());
                    });
                    ui.add_space(4.0);
                }
                if let Some(action) = action {
                    ui.add_space(8.0);
                    let button = egui::Button::new(
                        egui::RichText::new(action.label)
                            .strong()
                            .color(ui.visuals().selection.stroke.color),
                    )
                    .fill(ui.visuals().selection.bg_fill)
                    .min_size(egui::vec2(148.0, 32.0));
                    clicked = ui.add(button).on_hover_text(action.hover).clicked();
                }
            });
    });
    clicked
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::Queryable;
    use std::{cell::Cell, rc::Rc};

    #[test]
    fn guidance_fits_narrow_width_and_returns_the_action_click() {
        let clicked = Rc::new(Cell::new(false));
        let clicked_in_ui = Rc::clone(&clicked);
        let mut harness = crate::gui::test_harness::builder()
            .with_size(egui::vec2(420.0, 560.0))
            .with_step_dt(1.0 / 60.0)
            .with_max_steps(32)
            .build_ui(move |ui| {
                let action_clicked = guidance_panel(
                    ui,
                    "narrow-guidance-test",
                    AppIcon::Evidence,
                    "Comece com uma evidência",
                    "Nenhuma evidência importada ainda.",
                    &[
                        (
                            "Escolha um arquivo",
                            "O processamento acontece neste computador.",
                        ),
                        (
                            "Execute a análise",
                            "Abra os detalhes para ver as análises disponíveis.",
                        ),
                        (
                            "Revise os resultados",
                            "Revise eventos e candidatos antes de confirmar achados.",
                        ),
                    ],
                    Some(GuidanceAction {
                        label: "Escolher arquivo…",
                        hover: "Abre o seletor nativo de arquivo",
                    }),
                );
                clicked_in_ui.set(action_clicked);
            });
        harness.run();
        harness.get_by_label("Comece com uma evidência");
        harness.get_by_label("Nenhuma evidência importada ainda.");
        let final_step = harness
            .get_by_label("Revise eventos e candidatos antes de confirmar achados.")
            .rect();
        assert!(final_step.left() >= 0.0 && final_step.right() <= 420.0);
        let action_rect = harness.get_by_label("Escolher arquivo…").rect();
        assert!(action_rect.left() >= 0.0 && action_rect.right() <= 420.0);
        assert!(!clicked.get());
        harness.get_by_label("Escolher arquivo…").click();
        harness.step();
        assert!(clicked.get());
    }

    #[test]
    fn reveal_starts_below_one_and_finishes_without_ambient_motion() {
        let mut harness = crate::gui::test_harness::builder()
            .with_step_dt(1.0 / 60.0)
            .with_max_steps(32)
            .build_ui(|_ui| {});
        harness.step();
        let context = harness.ctx.clone();
        let id = egui::Id::unique("guidance-reveal-test");
        let duration = Duration::from_millis(100);
        let initial = reveal_progress(&context, id, duration);
        assert!(initial < 1.0);
        for _ in 0..20 {
            let progress = reveal_progress(&context, id, duration);
            assert!((0.0..=1.0).contains(&progress));
            harness.step();
            if progress == 1.0 {
                break;
            }
        }
        assert_eq!(reveal_progress(&context, id, duration), 1.0);
    }

    #[test]
    fn zero_duration_resolves_reduced_motion_immediately() {
        let harness = crate::gui::test_harness::builder().build_ui(|_ui| {});
        let id = egui::Id::unique("guidance-reduced-motion-test");
        assert_eq!(reveal_progress(&harness.ctx, id, Duration::ZERO), 1.0);
    }

    #[test]
    fn repeated_reveal_keeps_the_final_opacity() {
        let mut harness = crate::gui::test_harness::builder()
            .with_step_dt(1.0 / 60.0)
            .with_max_steps(32)
            .build_ui(|_ui| {});
        harness.step();
        let id = egui::Id::unique("guidance-repeated-reveal-test");
        let duration = Duration::from_millis(20);
        assert!(reveal_progress(&harness.ctx, id, duration) < 1.0);
        for _ in 0..4 {
            harness.step();
        }
        assert_eq!(reveal_progress(&harness.ctx, id, duration), 1.0);
        harness.step();
        assert_eq!(reveal_progress(&harness.ctx, id, duration), 1.0);
    }
}
