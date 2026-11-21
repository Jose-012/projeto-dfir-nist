//! Catálogo local de ícones vetoriais para a interface nativa.
//!
//! A geometria é produzida sem `Context`, frames ou fontes externas, para que
//! possa ser testada de forma determinística. O texto canônico continua
//! disponível como fallback visível e nome acessível.

// O catálogo é integrado de forma incremental na GUI. Variantes e helpers ainda
// não referenciados pela interface não são erro de compilação neste módulo.
#![allow(dead_code)]

use egui::{Color32, Painter, Pos2, Rect, Response, Role, Sense, Shape, Stroke, Ui, Vec2, pos2};

/// Menor lado, em pontos, em que um ícone vetorial permanece legível.
pub(crate) const MIN_ICON_SIDE: f32 = 16.0;

/// Número máximo de primitivas em um ícone.
///
/// O limite protege a pintura e a inspeção geométrica contra catálogos que
/// crescem sem uma revisão explícita.
pub(crate) const MAX_SHAPES_PER_ICON: usize = 40;

const STROKE_MIN: f32 = 1.25;
const STROKE_MAX: f32 = 2.25;
const STROKE_RATIO: f32 = 0.09;

/// Taxonomia semântica dos ícones propostos para a GUI.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum AppIcon {
    /// Pasta de evidências.
    Evidence,
    /// Linha do tempo.
    Timeline,
    /// Candidatos de detecção.
    Detections,
    /// Incidentes.
    Incidents,
    /// Entregáveis.
    Deliverables,
    /// Exportação.
    Export,
    /// Trilha de auditoria.
    Audit,
    /// Diagnóstico.
    Diagnostics,
    /// Trabalho em execução.
    JobRunning,
    /// Trabalho concluído com sucesso.
    JobSucceeded,
    /// Trabalho com falha.
    JobFailed,
    /// Importação.
    Import,
    /// Achados confirmados.
    Findings,
    /// Plano de trabalho.
    Plan,
}

impl AppIcon {
    /// Todos os ícones do catálogo, em ordem estável.
    pub(crate) const ALL: [Self; 14] = [
        Self::Evidence,
        Self::Timeline,
        Self::Detections,
        Self::Incidents,
        Self::Deliverables,
        Self::Export,
        Self::Audit,
        Self::Diagnostics,
        Self::JobRunning,
        Self::JobSucceeded,
        Self::JobFailed,
        Self::Import,
        Self::Findings,
        Self::Plan,
    ];

    /// Nome textual canônico, também usado como fallback acessível.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Evidence => "Evidência",
            Self::Timeline => "Timeline",
            Self::Detections => "Detecções",
            Self::Incidents => "Incidente",
            Self::Deliverables => "Entregáveis",
            Self::Export => "Exportar",
            Self::Audit => "Auditoria",
            Self::Diagnostics => "Diagnóstico",
            Self::JobRunning => "Executando",
            Self::JobSucceeded => "Concluído",
            Self::JobFailed => "Falhou",
            Self::Import => "Importar",
            Self::Findings => "Achados",
            Self::Plan => "Plano",
        }
    }

    /// Rótulo de navegação acessível (plural para listas, singular para botões individuais).
    /// Separado do catálogo para permitir rótulos de navegação consistentes
    /// sem alterar o catálogo de ícones.
    pub(crate) const fn nav_label(self) -> &'static str {
        match self {
            Self::Evidence => "Evidências",
            Self::Timeline => "Timeline",
            Self::Detections => "Detecções",
            Self::Incidents => "Incidentes",
            Self::Deliverables => "Entregáveis",
            Self::Export => "Exportar",
            Self::Audit => "Auditoria",
            Self::Diagnostics => "Diagnóstico",
            Self::JobRunning => "Executando",
            Self::JobSucceeded => "Concluído",
            Self::JobFailed => "Falhou",
            Self::Import => "Importar",
            Self::Findings => "Achados",
            Self::Plan => "Plano",
        }
    }
}

/// Espessura do traço para um lado de ícone.
///
/// O valor é proporcional ao lado e limitado ao intervalo visual do catálogo.
pub(crate) fn stroke_width(side: f32) -> f32 {
    (side * STROKE_RATIO).clamp(STROKE_MIN, STROKE_MAX)
}

/// Constrói a geometria de um ícone dentro do `rect` fornecido.
///
/// O retorno é limitado por [`MAX_SHAPES_PER_ICON`] e não cria `Context` nem
/// depende de uma frame. O desenho é feito por [`paint_icon`].
pub(crate) fn icon_shapes(icon: AppIcon, rect: Rect) -> Vec<Shape> {
    let stroke = Stroke::new(
        stroke_width(rect.width().min(rect.height())),
        Color32::WHITE,
    );
    build_shapes(icon, rect, stroke)
}

/// Pinta um ícone no `rect` usando as primitivas de [`icon_shapes`].
///
/// O painter é limitado ao `rect`, de modo que qualquer futura forma fora da
/// área reservada não invade o conteúdo adjacente.
pub(crate) fn paint_icon(painter: &Painter, rect: Rect, icon: AppIcon, color: Color32) {
    let stroke = Stroke::new(stroke_width(rect.width().min(rect.height())), color);
    let shapes = build_shapes(icon, rect, stroke);
    let painter = painter.with_clip_rect(rect);
    for shape in shapes {
        painter.add(shape);
    }
}

fn build_shapes(icon: AppIcon, rect: Rect, stroke: Stroke) -> Vec<Shape> {
    let mut shapes = Vec::new();

    match icon {
        AppIcon::Evidence => evidence(&mut shapes, rect, stroke),
        AppIcon::Timeline => timeline(&mut shapes, rect, stroke),
        AppIcon::Detections => detections(&mut shapes, rect, stroke),
        AppIcon::Incidents => incidents(&mut shapes, rect, stroke),
        AppIcon::Deliverables => deliverables(&mut shapes, rect, stroke),
        AppIcon::Export => export(&mut shapes, rect, stroke),
        AppIcon::Audit => audit(&mut shapes, rect, stroke),
        AppIcon::Diagnostics => diagnostics(&mut shapes, rect, stroke),
        AppIcon::JobRunning => job_running(&mut shapes, rect, stroke),
        AppIcon::JobSucceeded => job_succeeded(&mut shapes, rect, stroke),
        AppIcon::JobFailed => job_failed(&mut shapes, rect, stroke),
        AppIcon::Import => import(&mut shapes, rect, stroke),
        AppIcon::Findings => findings(&mut shapes, rect, stroke),
        AppIcon::Plan => plan(&mut shapes, rect, stroke),
    }

    shapes.truncate(MAX_SHAPES_PER_ICON);
    shapes
}

/// Widget de ícone com nome acessível e hover.
///
/// O widget reserva pelo menos [`MIN_ICON_SIDE`] pontos. O chamador deve manter
/// o rótulo visível ou associar este widget a um texto por [`Response::labelled_by`].
pub(crate) fn icon_widget(ui: &mut Ui, icon: AppIcon, side: f32) -> Response {
    let requested_side = side.max(MIN_ICON_SIDE);
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(requested_side), Sense::hover());

    paint_icon(ui.painter(), rect, icon, ui.visuals().text_color());
    response
        .widget_info(|| egui::WidgetInfo::labeled(Role::Image, response.enabled(), icon.label()));
    response.on_hover_text(icon.label())
}

/// Widget com ícone e rótulo visível, padrão para navegação e ações.
pub(crate) fn icon_label(ui: &mut Ui, icon: AppIcon) -> Response {
    ui.horizontal(|ui| {
        let response = icon_widget(ui, icon, MIN_ICON_SIDE);
        let label = ui.label(icon.label());
        response.labelled_by(label.id)
    })
    .inner
}

fn evidence(shapes: &mut Vec<Shape>, rect: Rect, stroke: Stroke) {
    let side = rect.width().min(rect.height());
    let left = rect.left() + side * 0.16;
    let right = rect.right() - side * 0.14;
    let top = rect.top() + side * 0.20;
    let bottom = rect.bottom() - side * 0.14;
    let tab_width = side * 0.26;

    shapes.push(Shape::closed_line(
        vec![
            pos2(left, top),
            pos2(left, rect.top() + side * 0.10),
            pos2(left + tab_width, rect.top() + side * 0.10),
            pos2(left + tab_width + side * 0.08, top),
            pos2(right, top),
            pos2(right, bottom),
            pos2(left, bottom),
        ],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [
            pos2(left + side * 0.08, bottom),
            pos2(left + side * 0.08, top),
        ],
        stroke,
    ));
}

fn timeline(shapes: &mut Vec<Shape>, rect: Rect, stroke: Stroke) {
    let side = rect.width().min(rect.height());
    let x1 = rect.left() + side * 0.18;
    let x2 = rect.left() + side * 0.52;
    let x3 = rect.right() - side * 0.18;
    let axis_y = rect.center().y + side * 0.13;

    shapes.push(Shape::line_segment(
        [pos2(x1, axis_y), pos2(x3, axis_y)],
        stroke,
    ));
    for (x, top_fraction) in [(x1, 0.20), (x2, 0.34), (x3, 0.24)] {
        let top = rect.top() + side * top_fraction;
        shapes.push(Shape::line_segment([pos2(x, top), pos2(x, axis_y)], stroke));
        shapes.push(Shape::circle_filled(
            pos2(x, top),
            side * 0.055,
            stroke.color,
        ));
    }
}

fn detections(shapes: &mut Vec<Shape>, rect: Rect, stroke: Stroke) {
    let side = rect.width().min(rect.height());
    let center = rect.center();
    let outer = side * 0.30;
    let inner = side * 0.17;

    shapes.push(Shape::circle_stroke(center, outer, stroke));
    shapes.push(Shape::circle_stroke(center, inner, stroke));
    shapes.push(Shape::circle_filled(center, side * 0.055, stroke.color));
    shapes.push(Shape::line_segment(
        [
            pos2(center.x, center.y - outer),
            pos2(center.x, center.y - inner),
        ],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [
            pos2(center.x, center.y + inner),
            pos2(center.x, center.y + outer),
        ],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [
            pos2(center.x - outer, center.y),
            pos2(center.x - inner, center.y),
        ],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [
            pos2(center.x + inner, center.y),
            pos2(center.x + outer, center.y),
        ],
        stroke,
    ));
}

fn incidents(shapes: &mut Vec<Shape>, rect: Rect, stroke: Stroke) {
    let side = rect.width().min(rect.height());
    let center = rect.center();
    let half = side * 0.29;
    let points = vec![
        pos2(center.x, center.y - half),
        pos2(center.x + half * 0.78, center.y),
        pos2(center.x, center.y + half),
        pos2(center.x - half * 0.78, center.y),
    ];

    shapes.push(Shape::closed_line(points, stroke));
    shapes.push(Shape::line_segment(
        [
            pos2(center.x, center.y - side * 0.10),
            pos2(center.x, center.y + side * 0.04),
        ],
        stroke,
    ));
    shapes.push(Shape::circle_filled(
        pos2(center.x, center.y + side * 0.14),
        side * 0.035,
        stroke.color,
    ));
}

fn deliverables(shapes: &mut Vec<Shape>, rect: Rect, stroke: Stroke) {
    let side = rect.width().min(rect.height());
    let left = rect.left() + side * 0.22;
    let right = rect.right() - side * 0.20;
    let top = rect.top() + side * 0.13;
    let bottom = rect.bottom() - side * 0.12;
    let fold = side * 0.18;

    shapes.push(Shape::closed_line(
        vec![
            pos2(left, top),
            pos2(right - fold, top),
            pos2(right, top + fold),
            pos2(right, bottom),
            pos2(left, bottom),
        ],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [pos2(right - fold, top), pos2(right - fold, top + fold)],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [pos2(right - fold, top + fold), pos2(right, top + fold)],
        stroke,
    ));
    for row in 0..3 {
        let y = top + side * (0.40 + row as f32 * 0.13);
        shapes.push(Shape::line_segment(
            [pos2(left + side * 0.10, y), pos2(right - side * 0.10, y)],
            stroke,
        ));
    }
}

fn export(shapes: &mut Vec<Shape>, rect: Rect, stroke: Stroke) {
    let side = rect.width().min(rect.height());
    let left = rect.left() + side * 0.18;
    let right = rect.right() - side * 0.18;
    let top = rect.top() + side * 0.34;
    let bottom = rect.bottom() - side * 0.15;
    let center_x = rect.center().x;
    let arrow_top = rect.top() + side * 0.10;

    shapes.push(Shape::closed_line(
        vec![
            pos2(left, top),
            pos2(right, top),
            pos2(right, bottom),
            pos2(left, bottom),
        ],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [pos2(center_x, arrow_top), pos2(center_x, top + side * 0.13)],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [
            pos2(center_x - side * 0.11, arrow_top + side * 0.10),
            pos2(center_x, arrow_top),
        ],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [
            pos2(center_x, arrow_top),
            pos2(center_x + side * 0.11, arrow_top + side * 0.10),
        ],
        stroke,
    ));
}

fn audit(shapes: &mut Vec<Shape>, rect: Rect, stroke: Stroke) {
    let side = rect.width().min(rect.height());
    let left = rect.left() + side * 0.18;
    let right = rect.right() - side * 0.17;
    let top = rect.top() + side * 0.14;
    let bottom = rect.bottom() - side * 0.14;

    shapes.push(Shape::closed_line(
        vec![
            pos2(left, top),
            pos2(right, top),
            pos2(right, bottom),
            pos2(left, bottom),
        ],
        stroke,
    ));
    for row in 0..3 {
        let y = top + side * (0.13 + row as f32 * 0.14);
        shapes.push(Shape::circle_filled(
            pos2(left + side * 0.12, y),
            side * 0.045,
            stroke.color,
        ));
        shapes.push(Shape::line_segment(
            [pos2(left + side * 0.22, y), pos2(right - side * 0.10, y)],
            stroke,
        ));
    }
}

fn diagnostics(shapes: &mut Vec<Shape>, rect: Rect, stroke: Stroke) {
    let side = rect.width().min(rect.height());
    let center = rect.center();
    let radius = side * 0.16;

    shapes.push(Shape::circle_stroke(center, radius, stroke));
    shapes.push(Shape::line_segment(
        [
            pos2(center.x + radius * 0.72, center.y + radius * 0.72),
            pos2(rect.right() - side * 0.10, rect.bottom() - side * 0.10),
        ],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [
            pos2(center.x - side * 0.07, center.y),
            pos2(center.x, center.y + side * 0.08),
        ],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [
            pos2(center.x, center.y + side * 0.08),
            pos2(center.x + side * 0.07, center.y - side * 0.06),
        ],
        stroke,
    ));
    shapes.push(Shape::circle_filled(
        pos2(center.x + side * 0.07, center.y - side * 0.06),
        side * 0.025,
        stroke.color,
    ));
}

fn job_running(shapes: &mut Vec<Shape>, rect: Rect, stroke: Stroke) {
    let side = rect.width().min(rect.height());
    let center = rect.center();
    let radius = side * 0.29;
    let points = arc_points(center, radius, -35.0, 225.0, 12);

    let end = points[points.len() - 1];
    shapes.push(Shape::line(points, stroke));
    shapes.push(Shape::circle_filled(end, side * 0.055, stroke.color));
}

fn job_succeeded(shapes: &mut Vec<Shape>, rect: Rect, stroke: Stroke) {
    let side = rect.width().min(rect.height());
    let center = rect.center();
    let radius = side * 0.29;

    shapes.push(Shape::circle_stroke(center, radius, stroke));
    shapes.push(Shape::line_segment(
        [
            pos2(center.x - radius * 0.50, center.y),
            pos2(center.x - radius * 0.10, center.y + radius * 0.38),
        ],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [
            pos2(center.x - radius * 0.10, center.y + radius * 0.38),
            pos2(center.x + radius * 0.56, center.y - radius * 0.42),
        ],
        stroke,
    ));
}

fn job_failed(shapes: &mut Vec<Shape>, rect: Rect, stroke: Stroke) {
    let side = rect.width().min(rect.height());
    let center = rect.center();
    let radius = side * 0.29;
    let gap = 24.0_f32;

    shapes.push(Shape::line(
        arc_points(center, radius, gap, 360.0 - gap, 12),
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [
            pos2(center.x - radius * 0.50, center.y - radius * 0.50),
            pos2(center.x + radius * 0.50, center.y + radius * 0.50),
        ],
        stroke,
    ));
}

fn import(shapes: &mut Vec<Shape>, rect: Rect, stroke: Stroke) {
    let side = rect.width().min(rect.height());
    let left = rect.left() + side * 0.18;
    let right = rect.right() - side * 0.18;
    let top = rect.top() + side * 0.34;
    let bottom = rect.bottom() - side * 0.15;
    let center_x = rect.center().x;
    let arrow_bottom = top + side * 0.13;

    shapes.push(Shape::closed_line(
        vec![
            pos2(left, top),
            pos2(right, top),
            pos2(right, bottom),
            pos2(left, bottom),
        ],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [
            pos2(center_x, rect.top() + side * 0.10),
            pos2(center_x, arrow_bottom),
        ],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [
            pos2(center_x - side * 0.11, arrow_bottom - side * 0.10),
            pos2(center_x, arrow_bottom),
        ],
        stroke,
    ));
    shapes.push(Shape::line_segment(
        [
            pos2(center_x, arrow_bottom),
            pos2(center_x + side * 0.11, arrow_bottom - side * 0.10),
        ],
        stroke,
    ));
}

fn findings(shapes: &mut Vec<Shape>, rect: Rect, stroke: Stroke) {
    let side = rect.width().min(rect.height());
    let center = rect.center();
    let half = side * 0.30;
    let points = vec![
        pos2(center.x, center.y - half),
        pos2(center.x + half, center.y),
        pos2(center.x, center.y + half),
        pos2(center.x - half, center.y),
    ];

    shapes.push(Shape::closed_line(points, stroke));
    shapes.push(Shape::line_segment(
        [
            pos2(center.x, center.y - side * 0.13),
            pos2(center.x, center.y + side * 0.05),
        ],
        stroke,
    ));
    shapes.push(Shape::circle_filled(
        pos2(center.x, center.y + side * 0.16),
        side * 0.035,
        stroke.color,
    ));
}

fn plan(shapes: &mut Vec<Shape>, rect: Rect, stroke: Stroke) {
    let side = rect.width().min(rect.height());
    let left = rect.left() + side * 0.18;
    let right = rect.right() - side * 0.17;
    let top = rect.top() + side * 0.14;
    let bottom = rect.bottom() - side * 0.14;

    shapes.push(Shape::closed_line(
        vec![
            pos2(left, top),
            pos2(right, top),
            pos2(right, bottom),
            pos2(left, bottom),
        ],
        stroke,
    ));
    for row in 0..3 {
        let y = top + side * (0.15 + row as f32 * 0.18);
        shapes.push(Shape::circle_stroke(
            pos2(left + side * 0.12, y),
            side * 0.045,
            stroke,
        ));
        shapes.push(Shape::line_segment(
            [pos2(left + side * 0.24, y), pos2(right - side * 0.10, y)],
            stroke,
        ));
    }
}

fn arc_points(
    center: Pos2,
    radius: f32,
    start_degrees: f32,
    end_degrees: f32,
    count: usize,
) -> Vec<Pos2> {
    let count = count.max(2);
    let start = start_degrees.to_radians();
    let end = end_degrees.to_radians();
    (0..count)
        .map(|index| {
            let t = index as f32 / (count - 1) as f32;
            let angle = start + (end - start) * t;
            pos2(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Role;
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };
    use std::collections::HashSet;

    const TEST_RECT: Rect = Rect::from_min_max(pos2(4.0, 7.0), pos2(20.0, 23.0));

    #[test]
    fn stroke_width_is_clamped_and_proportional() {
        assert_eq!(stroke_width(0.0), STROKE_MIN);
        assert_eq!(stroke_width(10.0), STROKE_MIN);
        assert!((stroke_width(20.0) - 1.8).abs() < 1e-5);
        assert_eq!(stroke_width(30.0), STROKE_MAX);
        assert_eq!(stroke_width(100.0), STROKE_MAX);
    }

    #[test]
    fn all_icons_have_distinct_nonempty_geometry() {
        let mut signatures = HashSet::new();
        for icon in AppIcon::ALL {
            let shapes = icon_shapes(icon, TEST_RECT);
            assert!(!shapes.is_empty(), "{} must not be empty", icon.label());
            assert!(
                shapes.len() <= MAX_SHAPES_PER_ICON,
                "{} has {} shapes",
                icon.label(),
                shapes.len()
            );
            assert!(
                signatures.insert(geometry_signature(&shapes)),
                "{} repeats another icon's geometry",
                icon.label()
            );
        }
    }

    #[test]
    fn all_labels_are_unique_and_nonempty() {
        let labels: HashSet<_> = AppIcon::ALL.iter().map(|icon| icon.label()).collect();
        assert_eq!(labels.len(), AppIcon::ALL.len());
        assert!(labels.iter().all(|label| !label.is_empty()));
    }

    #[test]
    fn geometry_is_finite_and_contained_in_allocated_rect() {
        for icon in AppIcon::ALL {
            let shapes = icon_shapes(icon, TEST_RECT);
            for shape in &shapes {
                assert_finite_shape(shape, icon);
                assert_shape_contained(shape, TEST_RECT, icon);
            }
        }
    }

    #[test]
    fn icons_are_usable_at_the_minimum_side_and_labels_are_nonempty() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), Vec2::splat(MIN_ICON_SIDE));
        for icon in AppIcon::ALL {
            assert!(!icon.label().trim().is_empty());
            let shapes = icon_shapes(icon, rect);
            assert!(!shapes.is_empty(), "{} must not be empty", icon.label());
            assert!(
                shapes.len() <= MAX_SHAPES_PER_ICON,
                "{} has {} shapes",
                icon.label(),
                shapes.len()
            );
        }
    }

    #[test]
    fn all_icons_are_finite_at_multiple_sizes() {
        for side in [16.0, 24.0, 64.0] {
            let rect = Rect::from_min_size(pos2(3.0, 5.0), Vec2::splat(side));
            for icon in AppIcon::ALL {
                for shape in icon_shapes(icon, rect) {
                    assert_finite_shape(&shape, icon);
                }
            }
        }
    }

    fn geometry_signature(shapes: &[Shape]) -> String {
        shapes
            .iter()
            .map(|shape| format!("{shape:?}"))
            .collect::<Vec<_>>()
            .join("|")
    }

    fn assert_finite_shape(shape: &Shape, icon: AppIcon) {
        match shape {
            Shape::Circle(circle) => {
                assert!(
                    circle.center.is_finite(),
                    "{} has non-finite center",
                    icon.label()
                );
                assert!(
                    circle.radius.is_finite(),
                    "{} has non-finite radius",
                    icon.label()
                );
            }
            Shape::LineSegment { points, .. } => {
                assert!(
                    points.iter().all(|point| point.is_finite()),
                    "{} has non-finite line",
                    icon.label()
                );
            }
            Shape::Path(path) => {
                assert!(
                    path.points.iter().all(|point| point.is_finite()),
                    "{} has non-finite path",
                    icon.label()
                );
            }
            Shape::Rect(rect) => {
                assert!(
                    rect.rect.is_finite(),
                    "{} has non-finite rect",
                    icon.label()
                );
            }
            other => panic!("unexpected shape variant in {}: {other:?}", icon.label()),
        }
    }

    fn assert_shape_contained(shape: &Shape, rect: Rect, icon: AppIcon) {
        let tolerance = stroke_width(rect.width().min(rect.height())) * 0.51;
        let bounds = shape.visual_bounding_rect();
        assert!(
            bounds.min.x >= rect.min.x - tolerance && bounds.max.x <= rect.max.x + tolerance,
            "{} exceeds horizontal bounds: {:?} not in {:?}",
            icon.label(),
            bounds,
            rect
        );
        assert!(
            bounds.min.y >= rect.min.y - tolerance && bounds.max.y <= rect.max.y + tolerance,
            "{} exceeds vertical bounds: {:?} not in {:?}",
            icon.label(),
            bounds,
            rect
        );
    }

    /// Harness sem janela para executar os widgets de ícone e consultar a árvore de acessibilidade.
    /// O `MissingGlyphPolicy` padrão é `Panic`, de modo que qualquer caractere não renderizável
    /// (tofu) faz o teste falhar imediatamente.
    fn icon_widget_harness(icon: AppIcon, side: f32) -> Harness<'static> {
        Harness::new_ui(move |ui| {
            icon_widget(ui, icon, side);
        })
    }

    fn icon_label_harness(icon: AppIcon) -> Harness<'static> {
        Harness::new_ui(move |ui| {
            icon_label(ui, icon);
        })
    }

    #[test]
    fn icon_widget_has_accessible_name_image_role_and_min_side() {
        let harness = icon_widget_harness(AppIcon::Evidence, MIN_ICON_SIDE);
        let node = harness.get_by_role_and_label(Role::Image, AppIcon::Evidence.label());
        let rect = node.rect();
        assert!(rect.width() >= MIN_ICON_SIDE);
        assert!(rect.height() >= MIN_ICON_SIDE);
    }

    #[test]
    fn icon_widget_label_falls_back_to_textual_name() {
        let harness = icon_widget_harness(AppIcon::Timeline, MIN_ICON_SIDE);
        // Se não há rótulo visível associado, o nome acessível deve ser o texto canônico do ícone.
        let node = harness.get_by_role_and_label(Role::Image, AppIcon::Timeline.label());
        assert_eq!(
            node.accesskit_node().label().unwrap(),
            AppIcon::Timeline.label()
        );
    }

    #[test]
    fn icon_label_has_accessible_name_image_role_and_min_side() {
        let harness = icon_label_harness(AppIcon::Diagnostics);
        let node = harness.get_by_role_and_label(Role::Image, AppIcon::Diagnostics.label());
        let rect = node.rect();
        assert!(rect.width() >= MIN_ICON_SIDE);
        assert!(rect.height() >= MIN_ICON_SIDE);
    }

    #[test]
    fn icon_label_textual_fallback_is_visible() {
        let harness = icon_label_harness(AppIcon::Plan);
        // O rótulo textual deve estar presente na árvore de acessibilidade.
        let label_node = harness.get_by_label(AppIcon::Plan.label());
        assert!(!label_node.accesskit_node().label().unwrap().is_empty());
    }

    #[test]
    fn all_icons_via_icon_widget_have_accessible_name_image_role_and_min_side() {
        for icon in AppIcon::ALL {
            let harness = icon_widget_harness(icon, MIN_ICON_SIDE);
            let node = harness.get_by_role_and_label(Role::Image, icon.label());
            let rect = node.rect();
            assert!(
                rect.width() >= MIN_ICON_SIDE && rect.height() >= MIN_ICON_SIDE,
                "{} icon_widget must reserve at least {MIN_ICON_SIDE} points per side",
                icon.label()
            );
        }
    }

    #[test]
    fn all_icons_via_icon_label_have_accessible_name_image_role_and_min_side() {
        for icon in AppIcon::ALL {
            let harness = icon_label_harness(icon);
            let node = harness.get_by_role_and_label(Role::Image, icon.label());
            let rect = node.rect();
            assert!(
                rect.width() >= MIN_ICON_SIDE && rect.height() >= MIN_ICON_SIDE,
                "{} icon_label must reserve at least {MIN_ICON_SIDE} points per side",
                icon.label()
            );
        }
    }
}
