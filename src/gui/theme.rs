//! Tokens visuais e presets da interface nativa, sem dependências externas.
//!
//! A customização altera somente matiz/croma do acento. Cores semânticas de
//! erro, aviso e sucesso permanecem estáveis e independentes dessa escolha.

use std::time::Duration;

use egui::{Color32, Context, CornerRadius, Id, Shadow, Stroke, Theme, style::ScrollAnimation};
use serde::{Deserialize, Serialize};

pub(crate) const MOTION_DURATION: Duration = Duration::from_millis(160);

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreset {
    Light,
    Dark,
    #[default]
    GoldenWind,
    Custom,
}

impl ThemePreset {
    #[allow(dead_code)]
    pub(crate) const ALL: [Self; 4] = [Self::Light, Self::Dark, Self::GoldenWind, Self::Custom];

    #[allow(dead_code)]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Light => "Claro",
            Self::Dark => "Escuro",
            Self::GoldenWind => "Vento Áureo",
            Self::Custom => "Customizado",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BaseTheme {
    Light,
    Dark,
    #[default]
    GoldenWind,
}

impl BaseTheme {
    #[allow(dead_code)]
    pub(crate) const ALL: [Self; 3] = [Self::Light, Self::Dark, Self::GoldenWind];

    #[allow(dead_code)]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Light => "Claro",
            Self::Dark => "Escuro",
            Self::GoldenWind => "Vento Áureo",
        }
    }

    const fn dark(self) -> bool {
        !matches!(self, Self::Light)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct CustomAccent {
    pub base: BaseTheme,
    pub hue_degrees: f32,
    pub chroma: f32,
}

impl Default for CustomAccent {
    fn default() -> Self {
        Self {
            base: BaseTheme::GoldenWind,
            hue_degrees: 85.0,
            chroma: 0.14,
        }
    }
}

impl CustomAccent {
    pub fn validate(&self) -> Result<(), String> {
        if !self.hue_degrees.is_finite() || !(0.0..=360.0).contains(&self.hue_degrees) {
            return Err("matiz deve estar entre 0 e 360 graus".into());
        }
        if !self.chroma.is_finite() || !(0.0..=0.32).contains(&self.chroma) {
            return Err("croma deve estar entre 0 e 0,32".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RadiusTokens {
    pub window: u8,
    pub control: u8,
    pub inset: u8,
    pub padding: u8,
}

impl Default for RadiusTokens {
    fn default() -> Self {
        Self {
            window: 12,
            control: 8,
            inset: 4,
            padding: 4,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ThemeTokens {
    pub dark_mode: bool,
    pub canvas: Color32,
    pub surface: Color32,
    pub raised: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub border: Color32,
    pub accent: Color32,
    pub accent_text: Color32,
    pub selection: Color32,
    pub selection_text: Color32,
    pub gradient_end: Color32,
    pub error: Color32,
    pub warning: Color32,
    pub success: Color32,
    pub radius: RadiusTokens,
    pub motion: Duration,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct Palette {
    canvas: Color32,
    surface: Color32,
    raised: Color32,
    text: Color32,
    muted: Color32,
    border: Color32,
    accent_lightness: f32,
    error: Color32,
    warning: Color32,
    success: Color32,
}

impl ThemeTokens {
    #[allow(dead_code)]
    pub fn new(preset: ThemePreset, custom: CustomAccent, reduced_motion: bool) -> Self {
        let base = match preset {
            ThemePreset::Light => BaseTheme::Light,
            ThemePreset::Dark => BaseTheme::Dark,
            ThemePreset::GoldenWind => BaseTheme::GoldenWind,
            ThemePreset::Custom => custom.base,
        };
        let palette = palette(base);
        let (hue, chroma) = match (preset, custom.validate()) {
            (ThemePreset::Custom, Ok(())) => (custom.hue_degrees, custom.chroma),
            (ThemePreset::Light | ThemePreset::Dark, _) => (250.0, 0.12),
            _ => (85.0, 0.14),
        };
        let accent = oklch(palette.accent_lightness, chroma, hue);
        let accent_text = readable_accent(
            palette.accent_lightness,
            chroma,
            hue,
            palette.surface,
            base.dark(),
        );
        let selection = mix(
            palette.surface,
            accent,
            if base.dark() { 0.28 } else { 0.18 },
        );
        let gradient_end = mix(
            palette.surface,
            accent,
            if base.dark() { 0.52 } else { 0.36 },
        );
        Self {
            dark_mode: base.dark(),
            canvas: palette.canvas,
            surface: palette.surface,
            raised: palette.raised,
            text: palette.text,
            muted: palette.muted,
            border: palette.border,
            accent,
            accent_text,
            selection,
            selection_text: palette.text,
            gradient_end,
            error: palette.error,
            warning: palette.warning,
            success: palette.success,
            radius: RadiusTokens::default(),
            motion: if reduced_motion {
                Duration::ZERO
            } else {
                MOTION_DURATION
            },
        }
    }

    fn interpolate(self, target: Self, progress: f32) -> Self {
        let progress = progress.clamp(0.0, 1.0);
        Self {
            // O slot ativo só muda no fim; as cores já progridem no slot atual.
            dark_mode: if progress >= 1.0 {
                target.dark_mode
            } else {
                self.dark_mode
            },
            canvas: mix(self.canvas, target.canvas, progress),
            surface: mix(self.surface, target.surface, progress),
            raised: mix(self.raised, target.raised, progress),
            text: mix(self.text, target.text, progress),
            muted: mix(self.muted, target.muted, progress),
            border: mix(self.border, target.border, progress),
            accent: mix(self.accent, target.accent, progress),
            accent_text: mix(self.accent_text, target.accent_text, progress),
            selection: mix(self.selection, target.selection, progress),
            selection_text: mix(self.selection_text, target.selection_text, progress),
            gradient_end: mix(self.gradient_end, target.gradient_end, progress),
            error: mix(self.error, target.error, progress),
            warning: mix(self.warning, target.warning, progress),
            success: mix(self.success, target.success, progress),
            radius: target.radius,
            motion: target.motion,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn apply(&self, context: &Context) {
        let theme = if self.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        };
        context.set_theme(theme);
        context.style_mut_of(theme, |style| {
            let mut visuals = if self.dark_mode {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            };
            visuals.dark_mode = self.dark_mode;
            visuals.panel_fill = self.canvas;
            visuals.window_fill = self.surface;
            visuals.extreme_bg_color = self.canvas;
            visuals.faint_bg_color = self.raised;
            visuals.code_bg_color = self.canvas;
            visuals.text_edit_bg_color = Some(self.canvas);
            visuals.weak_text_color = Some(self.muted);
            visuals.hyperlink_color = self.accent_text;
            visuals.warn_fg_color = self.warning;
            visuals.error_fg_color = self.error;
            visuals.selection = egui::style::Selection {
                bg_fill: self.selection,
                stroke: Stroke::new(1.0, self.selection_text),
            };
            let border = alpha(self.border, if self.dark_mode { 150 } else { 190 });
            visuals.window_stroke = Stroke::new(1.0, border);
            visuals.window_corner_radius = CornerRadius::same(self.radius.window);
            visuals.menu_corner_radius = CornerRadius::same(self.radius.control);
            let shadow = if self.dark_mode {
                Shadow {
                    offset: [0, 2],
                    blur: 8,
                    spread: 0,
                    color: Color32::from_black_alpha(70),
                }
            } else {
                Shadow {
                    offset: [0, 1],
                    blur: 4,
                    spread: 0,
                    color: Color32::from_black_alpha(24),
                }
            };
            visuals.window_shadow = shadow;
            visuals.popup_shadow = shadow;
            widget(
                &mut visuals.widgets.noninteractive,
                self.surface,
                self.text,
                border,
                self.radius.control,
            );
            widget(
                &mut visuals.widgets.inactive,
                self.surface,
                self.text,
                border,
                self.radius.control,
            );
            widget(
                &mut visuals.widgets.hovered,
                self.raised,
                self.accent_text,
                alpha(self.accent, 180),
                self.radius.control,
            );
            widget(
                &mut visuals.widgets.active,
                self.selection,
                self.selection_text,
                self.accent_text,
                self.radius.control,
            );
            widget(
                &mut visuals.widgets.open,
                self.raised,
                self.accent_text,
                self.accent_text,
                self.radius.control,
            );
            style.visuals = visuals;
            style.animation_time = self.motion.as_secs_f32();
            if self.motion.is_zero() {
                style.scroll_animation = ScrollAnimation::none();
            }
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(12.0, 6.0);
        });
    }
}

/// Transição limitada entre dois conjuntos de tokens, dirigida pelo relógio
/// do egui. O chamador decide se movimento reduzido deve ignorar a animação.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ThemeTransition {
    from: ThemeTokens,
    target: ThemeTokens,
    id: Id,
}

impl ThemeTransition {
    pub(crate) fn new(context: &Context, from: ThemeTokens, target: ThemeTokens) -> Self {
        let id = Id::unique("dfir-theme-transition");
        // Na primeira chamada com `true`, o egui retorna o alvo imediatamente.
        // Semear `false` com duração zero cria o estado inicial da transição.
        context.animate_bool_with_time(id, false, 0.0);
        Self { from, target, id }
    }

    pub(crate) fn target(&self) -> ThemeTokens {
        self.target
    }

    /// Retorna os tokens do quadro e se a transição continua ativa.
    pub(crate) fn apply(&mut self, context: &Context) -> (ThemeTokens, bool) {
        let progress = context
            .animate_bool_with_time(self.id, true, MOTION_DURATION.as_secs_f32())
            .clamp(0.0, 1.0);
        let current = self.from.interpolate(self.target, progress);
        current.apply(context);
        let active = progress < 1.0;
        if active {
            // Cobre também o quadro inicial com progresso zero; o último quadro
            // não agenda nada, portanto não há timer ou repaint residual.
            context.request_repaint();
        }
        (current, active)
    }
}

#[allow(dead_code)]
fn widget(
    visuals: &mut egui::style::WidgetVisuals,
    fill: Color32,
    text: Color32,
    border: Color32,
    radius: u8,
) {
    visuals.bg_fill = fill;
    visuals.weak_bg_fill = fill;
    visuals.fg_stroke = Stroke::new(1.0, text);
    visuals.bg_stroke = Stroke::new(1.0, border);
    visuals.corner_radius = CornerRadius::same(radius);
}

#[allow(dead_code)]
fn palette(base: BaseTheme) -> Palette {
    match base {
        BaseTheme::Light => Palette {
            canvas: rgb(244, 243, 239),
            surface: rgb(252, 251, 248),
            raised: rgb(255, 255, 255),
            text: rgb(34, 37, 42),
            muted: rgb(83, 88, 96),
            border: rgb(181, 178, 168),
            accent_lightness: 0.48,
            error: rgb(180, 35, 24),
            warning: rgb(133, 76, 0),
            success: rgb(35, 116, 67),
        },
        BaseTheme::Dark => Palette {
            canvas: rgb(20, 23, 27),
            surface: rgb(27, 31, 36),
            raised: rgb(37, 43, 50),
            text: rgb(237, 239, 242),
            muted: rgb(174, 181, 190),
            border: rgb(91, 102, 114),
            accent_lightness: 0.76,
            error: rgb(242, 118, 107),
            warning: rgb(229, 185, 92),
            success: rgb(111, 207, 151),
        },
        BaseTheme::GoldenWind => Palette {
            canvas: rgb(22, 19, 15),
            surface: rgb(32, 27, 20),
            raised: rgb(44, 37, 27),
            text: rgb(245, 239, 223),
            muted: rgb(198, 187, 164),
            border: rgb(111, 91, 56),
            accent_lightness: 0.80,
            error: rgb(242, 118, 107),
            warning: rgb(229, 185, 92),
            success: rgb(111, 207, 151),
        },
    }
}

#[allow(dead_code)]
fn readable_accent(
    lightness: f32,
    chroma: f32,
    hue: f32,
    background: Color32,
    dark: bool,
) -> Color32 {
    let mut lightness = lightness;
    for _ in 0..80 {
        let color = oklch(lightness, chroma, hue);
        if contrast_ratio(color, background) >= 4.5 {
            return color;
        }
        lightness += if dark { 0.01 } else { -0.01 };
        if !(0.0..=1.0).contains(&lightness) {
            break;
        }
    }
    if dark { Color32::WHITE } else { Color32::BLACK }
}

#[allow(dead_code)]
fn oklch(lightness: f32, chroma: f32, hue_degrees: f32) -> Color32 {
    let hue = hue_degrees.to_radians();
    let a = chroma * hue.cos();
    let b = chroma * hue.sin();
    let l = (lightness + 0.396_337_77 * a + 0.215_803_76 * b).powi(3);
    let m = (lightness - 0.105_561_346 * a - 0.063_854_17 * b).powi(3);
    let s = (lightness - 0.089_484_18 * a - 1.291_485_5 * b).powi(3);
    let red = 4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s;
    let green = -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s;
    let blue = -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s;
    rgb(gamma(red), gamma(green), gamma(blue))
}

#[allow(dead_code)]
fn gamma(value: f32) -> u8 {
    let value = value.clamp(0.0, 1.0);
    let encoded = if value <= 0.003_130_8 {
        12.92 * value
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round() as u8
}

#[allow(dead_code)]
fn linear(channel: u8) -> f32 {
    let value = channel as f32 / 255.0;
    if value <= 0.040_45 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

#[allow(dead_code)]
pub(crate) fn contrast_ratio(first: Color32, second: Color32) -> f32 {
    fn luminance(color: Color32) -> f32 {
        let [r, g, b, _] = color.to_array();
        0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
    }
    let a = luminance(first);
    let b = luminance(second);
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[allow(dead_code)]
fn mix(start: Color32, end: Color32, amount: f32) -> Color32 {
    let a = start.to_array();
    let b = end.to_array();
    Color32::from_rgba_unmultiplied(
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * amount).round() as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * amount).round() as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * amount).round() as u8,
        255,
    )
}

#[allow(dead_code)]
fn alpha(color: Color32, opacity: u8) -> Color32 {
    let [r, g, b, _] = color.to_array();
    Color32::from_rgba_unmultiplied(r, g, b, opacity)
}

#[allow(dead_code)]
const fn rgb(red: u8, green: u8, blue: u8) -> Color32 {
    Color32::from_rgb(red, green, blue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oklch_matches_reference_srgb_colors() {
        assert_eq!(oklch(0.0, 0.0, 0.0), Color32::BLACK);
        assert_eq!(oklch(1.0, 0.0, 0.0), Color32::WHITE);
        let red = oklch(0.627_96, 0.257_68, 29.233_9).to_array();
        assert!(red[0] >= 254 && red[1] <= 1 && red[2] <= 1, "{red:?}");
    }

    #[test]
    fn presets_keep_text_and_semantics_readable() {
        for preset in [
            ThemePreset::Light,
            ThemePreset::Dark,
            ThemePreset::GoldenWind,
        ] {
            let tokens = ThemeTokens::new(preset, CustomAccent::default(), false);
            for surface in [tokens.canvas, tokens.surface, tokens.raised] {
                assert!(contrast_ratio(tokens.text, surface) >= 4.5, "{preset:?}");
                assert!(contrast_ratio(tokens.muted, surface) >= 4.5, "{preset:?}");
                assert!(contrast_ratio(tokens.error, surface) >= 4.5, "{preset:?}");
                assert!(contrast_ratio(tokens.warning, surface) >= 4.5, "{preset:?}");
                assert!(contrast_ratio(tokens.success, surface) >= 4.5, "{preset:?}");
            }
            assert!(contrast_ratio(tokens.accent_text, tokens.surface) >= 4.5);
            assert!(contrast_ratio(tokens.selection_text, tokens.selection) >= 4.5);
        }
    }

    #[test]
    fn custom_accent_stays_readable_across_hues() {
        for base in BaseTheme::ALL {
            for hue in [0.0, 45.0, 120.0, 220.0, 300.0, 360.0] {
                let custom = CustomAccent {
                    base,
                    hue_degrees: hue,
                    chroma: 0.32,
                };
                let tokens = ThemeTokens::new(ThemePreset::Custom, custom, false);
                assert!(contrast_ratio(tokens.accent_text, tokens.surface) >= 4.5);
            }
        }
    }

    #[test]
    fn radii_are_concentric() {
        let radius = RadiusTokens::default();
        assert_eq!(radius.control + radius.padding, radius.window);
        assert_eq!(radius.inset + radius.padding, radius.control);
    }

    #[test]
    fn reduced_motion_disables_animation() {
        let normal = ThemeTokens::new(ThemePreset::GoldenWind, CustomAccent::default(), false);
        let reduced = ThemeTokens::new(ThemePreset::GoldenWind, CustomAccent::default(), true);
        assert_eq!(normal.motion, MOTION_DURATION);
        assert_eq!(reduced.motion, Duration::ZERO);
        let context = Context::default();
        reduced.apply(&context);
        let style = context.style_of(Theme::Dark);
        assert_eq!(style.animation_time, 0.0);
        assert!(style.scroll_animation.points_per_second.is_infinite());
    }

    #[test]
    fn apply_switches_active_theme_and_panel_colors() {
        let context = Context::default();
        context.set_theme(Theme::Dark);

        let light = ThemeTokens::new(ThemePreset::Light, CustomAccent::default(), false);
        light.apply(&context);
        assert_eq!(context.theme(), Theme::Light);
        assert_eq!(
            context.style_of(Theme::Light).visuals.panel_fill,
            light.canvas
        );
        assert!(!context.style_of(Theme::Light).visuals.dark_mode);

        let dark = ThemeTokens::new(ThemePreset::Dark, CustomAccent::default(), false);
        dark.apply(&context);
        assert_eq!(context.theme(), Theme::Dark);
        assert_eq!(
            context.style_of(Theme::Dark).visuals.panel_fill,
            dark.canvas
        );
        assert!(context.style_of(Theme::Dark).visuals.dark_mode);
    }

    #[test]
    fn theme_transition_interpolates_and_finishes_without_repaint_loop() {
        let mut harness = egui_kittest::HarnessBuilder::default()
            .with_step_dt(1.0 / 60.0)
            .with_max_steps(32)
            .build_ui(|_ui| {});
        harness.step();
        let context = harness.ctx.clone();
        let from = ThemeTokens::new(ThemePreset::GoldenWind, CustomAccent::default(), false);
        let target = ThemeTokens::new(ThemePreset::Light, CustomAccent::default(), false);
        let mut transition = ThemeTransition::new(&context, from, target);

        let midpoint = from.interpolate(target, 0.5);
        assert_eq!(midpoint.dark_mode, from.dark_mode);
        assert_eq!(midpoint.canvas, mix(from.canvas, target.canvas, 0.5));
        assert_eq!(from.interpolate(target, 1.0), target);

        let (_, active) = transition.apply(&context);
        assert!(active);
        for _ in 0..20 {
            let (_, active) = transition.apply(&context);
            harness.step();
            if !active {
                break;
            }
        }
        let (final_tokens, active) = transition.apply(&context);
        assert!(!active);
        assert_eq!(final_tokens, target);
        assert_eq!(context.theme(), Theme::Light);
    }

    #[test]
    fn invalid_custom_values_are_rejected() {
        for custom in [
            CustomAccent {
                hue_degrees: f32::NAN,
                ..CustomAccent::default()
            },
            CustomAccent {
                hue_degrees: -1.0,
                ..CustomAccent::default()
            },
            CustomAccent {
                chroma: 0.33,
                ..CustomAccent::default()
            },
            CustomAccent {
                chroma: f32::INFINITY,
                ..CustomAccent::default()
            },
        ] {
            assert!(custom.validate().is_err());
        }
    }
}
