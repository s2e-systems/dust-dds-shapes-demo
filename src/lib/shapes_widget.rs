use super::app::{self, ShapeKind, ShapeType};
use eframe::egui::{self};

#[derive(Clone)]
pub struct MovingShapeObject {
    kind: ShapeKind,
    color: egui::Color32,
    position: egui::Pos2,
    size: f32,
    velocity: egui::Vec2,
}
impl From<&MovingShapeObject> for ShapeType {
    fn from(value: &MovingShapeObject) -> Self {
        let color = match value.color {
            app::PURPLE => "PURPLE",
            app::BLUE => "BLUE",
            app::RED => "RED",
            app::GREEN => "GREEN",
            app::YELLOW => "YELLOW",
            app::CYAN => "CYAN",
            app::MAGENTA => "MAGENTA",
            app::ORANGE => "ORANGE",
            _ => panic!("Color not valid"),
        }
        .to_string();
        Self {
            color,
            x: value.position.x as i32,
            y: value.position.y as i32,
            shapesize: value.size as i32,
        }
    }
}
impl MovingShapeObject {
    pub fn from_shape_type(kind: ShapeKind, shape_type: &ShapeType, velocity: egui::Vec2) -> Self {
        let color = match shape_type.color.as_str() {
            "PURPLE" => app::PURPLE,
            "BLUE" => app::BLUE,
            "RED" => app::RED,
            "GREEN" => app::GREEN,
            "YELLOW" => app::YELLOW,
            "CYAN" => app::CYAN,
            "MAGENTA" => app::MAGENTA,
            "ORANGE" => app::ORANGE,
            _ => panic!("color not supported"),
        };
        Self {
            kind,
            color,
            position: egui::pos2(shape_type.x as f32, shape_type.y as f32),
            size: shape_type.shapesize as f32,
            velocity,
        }
    }
    pub fn move_within_rect(&mut self, rect_size: egui::Vec2, time_delta: f32) {
        let radius = self.size / 2.0;
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, rect_size);
        // Inset rect to account for shape size
        let rect = rect.expand(-radius);

        let reflection_normal = if self.position.x < rect.left() {
            self.position = egui::pos2(rect.left(), self.position.y);
            Some(egui::vec2(1.0, 0.0))
        } else if self.position.x > rect.right() {
            self.position = egui::pos2(rect.right(), self.position.y);
            Some(egui::vec2(-1.0, 0.0))
        } else if self.position.y < rect.top() {
            self.position = egui::pos2(self.position.x, rect.top());
            Some(egui::vec2(0.0, 1.0))
        } else if self.position.y > rect.bottom() {
            self.position = egui::pos2(self.position.x, rect.bottom());
            Some(egui::vec2(0.0, -1.0))
        } else {
            None
        };
        if let Some(normal) = reflection_normal {
            // reflect motion in respect to normal of surface
            self.velocity = self.velocity - 2.0 * (self.velocity * normal) * normal;
        }
        self.position += self.velocity * time_delta;
    }
}


pub struct ShapesWidget<'a> {
    original_size: egui::Vec2,
    shape_list: &'a [MovingShapeObject],
}

impl<'a> ShapesWidget<'a> {
    pub fn new(original_size: egui::Vec2, shape_list: &'a [MovingShapeObject]) -> Self {
        Self {
            original_size,
            shape_list,
        }
    }

    fn paint_area_and_shapes(&self, ui: &mut egui::Ui) -> egui::Response {
        let max_size = ui.max_rect().size();
        let scale = if self.original_size.y / self.original_size.x > max_size.y / max_size.x {
            max_size.y / self.original_size.y
        } else {
            max_size.x / self.original_size.x
        };
        let desired_size = self.original_size * scale;
        let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
        painter.rect_filled(response.rect, egui::Rounding::ZERO, egui::Color32::WHITE);
        painter.rect_stroke(
            response.rect,
            egui::Rounding::ZERO,
            (0.5, egui::Color32::BLACK),
        );

        for moving_shape in self.shape_list {
            let mut shape = moving_shape_to_shape(&moving_shape, scale);
            shape.translate(response.rect.left_top().to_vec2());
            painter.add(shape);
        }

        response
    }
}


fn moving_shape_to_shape(moving_shape: &MovingShapeObject, scale: f32) -> egui::Shape {
    let stroke = egui::Stroke {
        width: 0.5,
        color: egui::Color32::BLACK,
    };

    let position = moving_shape.position * scale;
    let size = moving_shape.size * scale;

    match moving_shape.kind {
        ShapeKind::Circle => egui::epaint::CircleShape {
            center: position,
            radius: size / 2.0,
            fill: moving_shape.color,
            stroke,
        }
        .into(),
        ShapeKind::Triangle => egui::epaint::PathShape {
            points: vec![
                position + egui::vec2(0.0, -size / 2.0),
                position
                    + egui::vec2(-size / 2.0, size / 2.0),
                position
                    + egui::vec2(size / 2.0, size / 2.0),
            ],
            closed: true,
            fill: moving_shape.color,
            stroke,
        }
        .into(),
        ShapeKind::Square => egui::epaint::RectShape::new(
            egui::Rect::from_center_size(
                position,
                egui::epaint::vec2(size, size),
            ),
            egui::Rounding::ZERO,
            moving_shape.color,
            stroke,
        )
        .into(),
    }
}


impl<'a> egui::Widget for ShapesWidget<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        if ui.max_rect().width() / ui.max_rect().height() > 1.0 {
            ui.vertical_centered(|ui| self.paint_area_and_shapes(ui))
        } else {
            ui.horizontal_centered(|ui| self.paint_area_and_shapes(ui))
        }
        .response
    }
}
