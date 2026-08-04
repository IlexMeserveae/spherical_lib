use crate::plotting::{cpos, CanvasPainter, CanvasPos, CubicBezier, QuadraticBezier, SphericalPlotter};
use eframe::egui::{vec2, Color32, Painter, Pos2, Rect, Stroke};
use eframe::epaint::{CubicBezierShape, QuadraticBezierShape};
use crate::plotting::Color;

pub type EguiPlotter = SphericalPlotter<EguiCanvasPainter>;

    impl From<Color32> for Color {
        fn from(value: Color32) -> Self { Self::new(value.to_array()) }
    }
    impl From<Color> for Color32 {
        fn from(value: Color) -> Self {
            Self::from_rgba_unmultiplied(value.r(), value.g(), value.b(), value.a())
        }
    }

    pub struct EguiCanvasPainter {
        painter: Painter,
        canvas: Rect,
        dot_size: f32,
        line_width: f32,
    }
    impl EguiCanvasPainter {
        pub fn new(painter: Painter, canvas: Rect, dot_size: f32, line_width: f32)
                   -> EguiCanvasPainter { Self { painter, canvas, dot_size, line_width } }
        pub fn destruct(self) -> Painter { self.painter }
        pub fn pos2_from_canvas_pos(&self, pos: CanvasPos) -> Pos2 {
            self.canvas.center() + vec2(pos.x as f32, pos.y as f32)
        }
        pub fn canvas_pos_from_pos2(&self, pos: Pos2) -> CanvasPos {
            let pos = pos - self.canvas.center().to_vec2();
            cpos(pos.x as f64, pos.y as f64)
        }
    }
    impl CanvasPainter for EguiCanvasPainter {
        fn paint_dot(&self, pos: CanvasPos, color: Color) {
            let pos = self.pos2_from_canvas_pos(pos);

            self.painter.circle_filled(pos, self.dot_size, color);
        }

        fn paint_line(&self, start: CanvasPos, end: CanvasPos, color: Color) {
            let start = self.pos2_from_canvas_pos(start);
            let end = self.pos2_from_canvas_pos(end);
            let stroke = Stroke::new(self.line_width, color);

            self.painter.line_segment([start, end], stroke);
        }

        fn paint_quadratic_bezier(&self, bezier: QuadraticBezier, color: Color) {
            let points = bezier.destruct()
                .map(|p| self.pos2_from_canvas_pos(p));
            let stroke = Stroke::new(self.line_width, color);
            let bezier = QuadraticBezierShape::from_points_stroke(
                points, false, Color32::default(), stroke);

            self.painter.add(bezier);
        }

        fn paint_cubic_bezier(&self, bezier: CubicBezier, color: Color) {
            let points = bezier.destruct()
                .map(|p| self.pos2_from_canvas_pos(p));
            let stroke = Stroke::new(self.line_width, color);
            let bezier = CubicBezierShape::from_points_stroke(
                points, false, Color32::default(), stroke);

            self.painter.add(bezier);

        }
    }
