//! Efeitos visuais limitados da interface nativa.
//!
//! O módulo implementa temporização de transições de ação, crossfade e
//! gradientes curtos de seleção. Não há animação ambiente nem temporização
//! periódica: cada efeito parte de um evento explícito da interface e termina
//! dentro de uma duração máxima. Quando o operador pede movimento reduzido,
//! a transição é resolvida de imediato, sem espera nem progressão intermediária.
//!
//! O consumo da preferência `reduced_motion` fica a cargo da camada de UI. Este
//! módulo recebe a decisão já resolvida para não depender do arquivo de
//! preferências e permanecer testável de forma determinística.
// A integração na navegação e nos painéis de evidência e incidente é
// incremental. Os tipos ainda não referenciados pela interface não são erro de
// compilação neste módulo.
#![allow(dead_code)]

use std::collections::HashMap;
use std::time::Duration;

use egui::{
    Color32, Context, Direction, Id, Pos2, Rect, Shape,
    epaint::{Mesh, Vertex},
};

/// Duração máxima de uma transição de ação, em milissegundos.
///
/// O limite é normativo para a interface: transições acima desse valor
/// atrasam a leitura e são normalizadas para o teto.
pub(crate) const MAX_TRANSITION_MS: u32 = 180;

/// Duração recomendada para transições de ação, em milissegundos.
///
/// O valor fica dentro de [`MAX_TRANSITION_MS`] e representa a faixa de 120 a
/// 180 ms adotada para as ações da interface.
pub(crate) const ACTION_TRANSITION_MS: u32 = 150;

/// Estado de uma transição de ação limitada.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Transition {
    duration: Duration,
    elapsed: Duration,
}

impl Transition {
    /// Cria uma transição com duração limitada a [`MAX_TRANSITION_MS`].
    ///
    /// Uma duração de zero e o modo de movimento reduzido resolvem a
    /// transição de imediato: a duração efetiva é zero e o progresso inicial
    /// já é o final.
    pub(crate) fn new(duration_ms: u32, reduced_motion: bool) -> Self {
        let duration_ms = duration_ms.min(MAX_TRANSITION_MS);
        if reduced_motion || duration_ms == 0 {
            return Self {
                duration: Duration::ZERO,
                elapsed: Duration::ZERO,
            };
        }
        Self {
            duration: Duration::from_millis(u64::from(duration_ms)),
            elapsed: Duration::ZERO,
        }
    }

    /// Cria uma transição já concluída.
    pub(crate) fn completed() -> Self {
        Self {
            duration: Duration::ZERO,
            elapsed: Duration::ZERO,
        }
    }

    /// Duração efetiva da transição, já normalizada e limitada.
    pub(crate) fn duration(&self) -> Duration {
        self.duration
    }

    /// Avança a transição em `delta`, sem ultrapassar a duração efetiva.
    ///
    /// O tempo acumulado é saturado, de modo que uma sequência de quadros
    /// lentos ou um salto grande de relógio não produzam progresso fora do
    /// intervalo `0.0..=1.0`.
    pub(crate) fn advance(&mut self, delta: Duration) {
        self.elapsed = self.elapsed.saturating_add(delta).min(self.duration);
    }

    /// Progresso linear da transição no intervalo `0.0..=1.0`.
    pub(crate) fn progress(&self) -> f32 {
        if self.duration.is_zero() {
            return 1.0;
        }
        let elapsed = self.elapsed.as_secs_f32();
        let duration = self.duration.as_secs_f32();
        (elapsed / duration).clamp(0.0, 1.0)
    }

    /// Progresso suavizado da transição no intervalo `0.0..=1.0`.
    ///
    /// A curva `smoothstep` mantém o início e o fim estáveis, o que evita
    /// saltos visíveis no primeiro e no último quadro.
    pub(crate) fn eased_progress(&self) -> f32 {
        let progress = self.progress();
        progress * progress * (3.0 - 2.0 * progress)
    }

    /// Indica se a transição já atingiu a duração efetiva.
    pub(crate) fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Conclui a transição imediatamente.
    pub(crate) fn finish(&mut self) {
        self.elapsed = self.duration;
    }
}

#[cfg(test)]
mod tests {
    use super::{ACTION_TRANSITION_MS, MAX_TRANSITION_MS, Transition};
    use std::time::Duration;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn recommended_duration_stays_within_the_maximum() {
        assert!(ACTION_TRANSITION_MS <= MAX_TRANSITION_MS);
        assert!((120..=180).contains(&ACTION_TRANSITION_MS));
    }

    #[test]
    fn duration_is_clamped_to_the_maximum() {
        let transition = Transition::new(5_000, false);
        assert_eq!(transition.duration(), Duration::from_millis(180));
        assert_eq!(
            transition.duration(),
            Duration::from_millis(u64::from(MAX_TRANSITION_MS))
        );
    }

    #[test]
    fn duration_below_the_maximum_is_preserved() {
        let transition = Transition::new(120, false);
        assert_eq!(transition.duration(), Duration::from_millis(120));
    }

    #[test]
    fn reduced_motion_resolves_immediately() {
        let transition = Transition::new(150, true);
        assert_eq!(transition.duration(), Duration::ZERO);
        assert_eq!(transition.progress(), 1.0);
        assert!(transition.is_complete());
    }

    #[test]
    fn zero_duration_resolves_immediately() {
        let transition = Transition::new(0, false);
        assert_eq!(transition.duration(), Duration::ZERO);
        assert_eq!(transition.progress(), 1.0);
        assert!(transition.is_complete());
    }

    #[test]
    fn progress_is_bounded_and_monotone() {
        let mut transition = Transition::new(150, false);
        assert_eq!(transition.progress(), 0.0);

        let mut previous = transition.progress();
        for _ in 0..10 {
            transition.advance(Duration::from_millis(20));
            let progress = transition.progress();
            assert!(
                (0.0..=1.0).contains(&progress),
                "progresso fora do intervalo: {progress}"
            );
            assert!(
                progress >= previous,
                "progresso não monotónico: {previous} -> {progress}"
            );
            previous = progress;
        }
        assert_eq!(transition.progress(), 1.0);
        assert!(transition.is_complete());
    }

    #[test]
    fn advance_saturates_at_the_effective_duration() {
        let mut transition = Transition::new(150, false);
        transition.advance(Duration::from_secs(30));
        assert_eq!(transition.progress(), 1.0);
        assert!(transition.is_complete());
        assert_eq!(transition.duration(), Duration::from_millis(150));
    }

    #[test]
    fn finish_completes_the_transition() {
        let mut transition = Transition::new(150, false);
        transition.advance(Duration::from_millis(50));
        transition.finish();
        assert_eq!(transition.progress(), 1.0);
        assert!(transition.is_complete());
    }

    #[test]
    fn eased_progress_keeps_the_endpoints_stable() {
        let mut transition = Transition::new(150, false);
        assert_eq!(transition.eased_progress(), 0.0);

        transition.advance(Duration::from_millis(75));
        let midpoint = transition.eased_progress();
        assert!(
            midpoint > 0.0 && midpoint < 1.0,
            "meio suavizado: {midpoint}"
        );
        assert!(midpoint > transition.progress() * 0.5);

        transition.finish();
        assert_eq!(transition.eased_progress(), 1.0);
    }

    #[test]
    fn completed_transition_is_final() {
        let transition = Transition::completed();
        assert_eq!(transition.progress(), 1.0);
        assert!(transition.is_complete());
    }
}

// ===== Crossfade: transição de seleção acionada por evento =====

/// Resultado de uma atualização de crossfade.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CrossfadeOut {
    /// Progresso atual no intervalo `0.0..=1.0`.
    pub progress: f32,
    /// Verdadeiro enquanto a transição ainda está em andamento.
    /// A UI deve pedir repaint apenas enquanto `active` for verdadeiro.
    pub active: bool,
}

/// Estado de crossfade por widget, com progresso persistido entre quadros.
///
/// Este tipo agora usa o temporizador interno do egui (`egui::animate_bool_with_time`)
/// em vez de `std::time::Instant`, o que evita repaints eternos no harness.
/// O progresso fica no próprio `Crossfade`, de forma testável sem display.
#[derive(Default)]
pub(crate) struct Crossfade {
    progress: HashMap<Id, f32>,
}

impl Crossfade {
    /// Cria um crossfade vazio.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Avança o crossfade do `id` com a duração dada.
    ///
    /// Quando `reduced_motion` ou duração zero, resolve de imediato
    /// (`progress == 1.0`, `active == false`). Caso contrário usa o temporizador
    /// interno do egui (`ctx.animate_bool_with_time`), que agenda repaint apenas
    /// durante a progressão e nunca em idle.
    ///
    /// `duration` é expressa em milissegundos e convertida para segundos
    /// (`Duration::as_secs_f32()`), que é a unidade do egui.
    pub(crate) fn update(
        &mut self,
        ctx: &Context,
        id: Id,
        duration: Duration,
        reduced_motion: bool,
    ) -> CrossfadeOut {
        let progress = if reduced_motion || duration.is_zero() {
            // Movimento reduzido ou duração zero: resolve imediatamente
            // sem iniciar animação nem scheduler do egui.
            1.0
        } else {
            if !self.progress.contains_key(&id) {
                // O primeiro `true` de um Id desconhecido retorna 1.0 no egui.
                // Semear `false` preserva a transição na primeira seleção.
                ctx.animate_bool_with_time(id, false, 0.0);
            }
            let seconds = duration.as_secs_f32();
            let animated = ctx.animate_bool_with_time(id, true, seconds);
            // animate_bool_with_time retorna o progresso animado no intervalo
            // 0.0..=1.0; quando atinge o alvo (true) o progresso é 1.0.
            animated
        };

        let active = progress < 1.0;
        self.progress.insert(id, progress);

        CrossfadeOut { progress, active }
    }

    /// Força o crossfade do `id` a concluir imediatamente.
    pub(crate) fn finish(&mut self, ctx: &Context, id: Id) {
        self.progress.insert(id, 1.0);
        ctx.animate_bool_with_time(id, true, 0.0);
    }

    /// Remove o estado do `id` e resetar o scheduler egui para esse id,
    /// evitando repaint infinito ao deselecionar.
    pub(crate) fn clear(&mut self, ctx: &Context, id: Id) {
        self.progress.remove(&id);
        // Duração zero com target false finaliza imediatamente qualquer
        // animação pendente do egui para este id, sem repaint contínuo.
        ctx.animate_bool_with_time(id, false, 0.0);
    }

    /// Devolve o progresso atual do `id` sem alterar o estado.
    pub(crate) fn progress_of(&self, id: Id) -> f32 {
        *self.progress.get(&id).unwrap_or(&0.0)
    }
}

// ===== Gradiente de seleção =====

/// Direção do gradiente de borda de seleção.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SelectionEdge {
    /// Gradiente da esquerda para a direita (borda esquerda da seleção).
    LeftToRight,
    /// Gradiente da direita para a esquerda (borda direita da seleção).
    RightToLeft,
}

/// Constrói um gradiente horizontal curto na borda de seleção, recortado pelo
/// mesmo raio do controle para não reintroduzir cantos quadrados.
///
/// `rect` é a área da seleção; `color` é a cor de seleção; `width` é a
/// largura do gradiente em pontos, limitada à largura do `rect`.
/// A malha tem 20 pontos de perímetro e não usa textura nem nova dependência.
/// Retorna [`Shape::Noop`] quando a largura é zero ou negativa.
pub(crate) fn selection_gradient(
    rect: Rect,
    color: Color32,
    width: f32,
    corner_radius: f32,
) -> Shape {
    let width = width.clamp(0.0, rect.width().max(0.0));
    if width <= 0.0 {
        return Shape::Noop;
    }
    let grad_rect = Rect::from_min_max(
        Pos2::new(rect.left(), rect.top()),
        Pos2::new(rect.left() + width, rect.bottom()),
    );
    if corner_radius <= 0.0 {
        return Shape::gradient_rect(
            grad_rect,
            Direction::LeftToRight,
            [color, Color32::TRANSPARENT],
        );
    }

    let radius = corner_radius
        .min(grad_rect.width() / 2.0)
        .min(grad_rect.height() / 2.0);
    let color_at = |point: Pos2| {
        let remaining = 1.0 - ((point.x - grad_rect.left()) / grad_rect.width()).clamp(0.0, 1.0);
        Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            (f32::from(color.a()) * remaining).round() as u8,
        )
    };
    let mut vertices = Vec::with_capacity(21);
    vertices.push(Vertex::untextured(
        grad_rect.center(),
        color_at(grad_rect.center()),
    ));
    let corners = [
        (
            Pos2::new(grad_rect.right() - radius, grad_rect.top() + radius),
            -90.0,
        ),
        (
            Pos2::new(grad_rect.right() - radius, grad_rect.bottom() - radius),
            0.0,
        ),
        (
            Pos2::new(grad_rect.left() + radius, grad_rect.bottom() - radius),
            90.0,
        ),
        (
            Pos2::new(grad_rect.left() + radius, grad_rect.top() + radius),
            180.0,
        ),
    ];
    for (center, start_degrees) in corners {
        for step in 0..=4 {
            let angle = (start_degrees + step as f32 * 22.5).to_radians();
            let point = Pos2::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            );
            vertices.push(Vertex::untextured(point, color_at(point)));
        }
    }
    let mut indices = Vec::with_capacity(60);
    for point in 0..20_u32 {
        indices.extend_from_slice(&[0, point + 1, (point + 1) % 20 + 1]);
    }
    Shape::from(Mesh {
        indices,
        vertices,
        ..Default::default()
    })
}

/// Constrói um masked edge discreto por gradiente alpha na borda de seleção.
///
/// `rect` é a área da seleção; `color` é a cor de seleção; `width` é a
/// largura do gradiente em pontos, limitada à largura do `rect`.
/// Usa [`Shape::gradient_rect`] com `Color32::TRANSPARENT` como segundo stop,
/// sem `gamma_multiply`/`linear_multiply` e sem nova dependência.
///
/// Retorna [`Shape::Noop`] quando a largura é zero ou negativa.
pub(crate) fn masked_edge(rect: Rect, color: Color32, width: f32) -> Shape {
    let width = width.clamp(0.0, rect.width());
    if width <= 0.0 {
        return Shape::Noop;
    }
    let grad_rect = Rect::from_min_max(
        Pos2::new(rect.right() - width, rect.top()),
        Pos2::new(rect.right(), rect.bottom()),
    );
    Shape::gradient_rect(
        grad_rect,
        Direction::RightToLeft,
        [color, Color32::TRANSPARENT],
    )
}

#[cfg(test)]
mod tests_crossfade {
    use super::*;
    use egui::{Context, Id};
    use egui_kittest::Harness;
    use std::time::Duration;

    #[test]
    fn crossfade_reduced_motion_resolves_immediately() {
        let ctx = Context::default();
        let mut crossfade = Crossfade::new();
        let id = Id::unique("selecao_reduced_motion");
        let out = crossfade.update(&ctx, id, Duration::from_millis(150), true);
        assert_eq!(out.progress, 1.0);
        assert!(!out.active);
        assert_eq!(crossfade.progress_of(id), 1.0);
    }

    #[test]
    fn crossfade_zero_duration_resolves_immediately() {
        let ctx = Context::default();
        let mut crossfade = Crossfade::new();
        let id = Id::unique("selecao_zero_duration");
        let out = crossfade.update(&ctx, id, Duration::ZERO, false);
        assert_eq!(out.progress, 1.0);
        assert!(!out.active);
    }

    #[test]
    fn crossfade_progress_is_bounded_and_advances() {
        let mut harness = egui_kittest::HarnessBuilder::default()
            .with_step_dt(1.0 / 60.0)
            .with_max_steps(32)
            .build_ui(|_ui| {});
        harness.step();
        let ctx = harness.ctx.clone();
        let mut crossfade = Crossfade::new();
        let id = Id::unique("selecao_bounded");
        let duration = Duration::from_millis(100);

        // Primeiro quadro: animação iniciada, progresso ainda pode não ser 1.0.
        let out = crossfade.update(&ctx, id, duration, false);
        assert!((0.0..=1.0).contains(&out.progress));
        assert!(out.active && out.progress < 1.0);

        // Avança frames com a atualização chamada em cada um deles.
        for _ in 0..20 {
            let out = crossfade.update(&ctx, id, duration, false);
            harness.step();
            if !out.active {
                break;
            }
        }
        let out = crossfade.update(&ctx, id, duration, false);
        assert_eq!(out.progress, 1.0);
        assert!(!out.active);
        assert_eq!(crossfade.progress_of(id), 1.0);
    }

    #[test]
    fn crossfade_progress_never_exceeds_one() {
        let mut harness = egui_kittest::HarnessBuilder::default()
            .with_step_dt(1.0 / 60.0)
            .with_max_steps(32)
            .build_ui(|_ui| {});
        harness.step();
        let ctx = harness.ctx.clone();
        let mut crossfade = Crossfade::new();
        let id = Id::unique("selecao_exceeds");
        let duration = Duration::from_millis(100);

        for _ in 0..10 {
            let out = crossfade.update(&ctx, id, duration, false);
            assert!(
                (0.0..=1.0).contains(&out.progress),
                "progresso fora do intervalo: {}",
                out.progress
            );
            if out.progress == 1.0 {
                break;
            }
            harness.step();
        }
        assert_eq!(crossfade.progress_of(id), 1.0);
    }

    #[test]
    fn crossfade_finish_and_clear() {
        let harness = Harness::new_ui(|_ui| {});
        let ctx = harness.ctx.clone();
        let mut crossfade = Crossfade::new();
        let id = Id::unique("selecao_finish_clear");
        crossfade.update(&ctx, id, Duration::from_millis(150), false);
        crossfade.finish(&ctx, id);
        assert_eq!(crossfade.progress_of(id), 1.0);
        crossfade.clear(&ctx, id);
        assert_eq!(crossfade.progress_of(id), 0.0);
    }

    #[test]
    fn selection_gradient_zero_width_is_noop() {
        let rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(100.0, 50.0));
        let shape = selection_gradient(rect, Color32::WHITE, 0.0, 8.0);
        assert!(matches!(shape, Shape::Noop));
    }

    #[test]
    fn selection_gradient_width_is_clamped_to_rect() {
        let rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(100.0, 50.0));
        let shape = selection_gradient(rect, Color32::WHITE, 10_000.0, 8.0);
        assert!(!matches!(shape, Shape::Noop));
    }

    #[test]
    fn selection_gradient_respects_rounded_corners_and_bounded_geometry() {
        let rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(100.0, 50.0));
        let Shape::Mesh(mesh) = selection_gradient(rect, Color32::WHITE, 54.0, 8.0) else {
            panic!("seleção arredondada deveria usar malha");
        };
        assert_eq!(mesh.vertices.len(), 21);
        assert_eq!(mesh.indices.len(), 60);
        assert!(
            mesh.indices
                .iter()
                .all(|index| (*index as usize) < mesh.vertices.len())
        );
        assert!(mesh.vertices.iter().all(|vertex| vertex.pos.is_finite()));
        assert!(
            !mesh.vertices.iter().any(|vertex| vertex.pos
                == Rect::from_min_max(Pos2::ZERO, Pos2::new(54.0, 50.0)).left_top())
        );
    }

    #[test]
    fn masked_edge_zero_width_is_noop() {
        let rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(100.0, 50.0));
        let shape = masked_edge(rect, Color32::WHITE, 0.0);
        assert!(matches!(shape, Shape::Noop));
    }

    #[test]
    fn masked_edge_right_edge_places_gradient_on_right_edge() {
        let rect = Rect::from_min_max(Pos2::new(10.0, 10.0), Pos2::new(110.0, 60.0));
        let shape = masked_edge(rect, Color32::WHITE, 20.0);
        assert!(!matches!(shape, Shape::Noop));
    }

    #[test]
    fn selection_edge_variants_are_distinct() {
        assert_ne!(SelectionEdge::LeftToRight, SelectionEdge::RightToLeft);
    }

    #[test]
    fn crossfade_reuses_id_after_clear() {
        let mut harness = egui_kittest::HarnessBuilder::default()
            .with_step_dt(1.0 / 60.0)
            .with_max_steps(32)
            .build_ui(|_ui| {});
        harness.step();
        let ctx = harness.ctx.clone();
        let mut crossfade = Crossfade::new();
        let id = Id::unique("reused_id_after_clear");
        let duration = Duration::from_millis(100);

        let initial = crossfade.update(&ctx, id, duration, false);
        assert!(initial.active && initial.progress < 1.0);

        crossfade.clear(&ctx, id);
        assert_eq!(crossfade.progress_of(id), 0.0);

        let restarted = crossfade.update(&ctx, id, duration, false);
        assert!(restarted.active && restarted.progress < 1.0);

        harness.step();
        let advanced = crossfade.update(&ctx, id, duration, false);
        assert!((0.0..=1.0).contains(&advanced.progress));
        assert!(advanced.progress >= restarted.progress);
    }
}
