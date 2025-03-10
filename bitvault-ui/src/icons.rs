#![allow(non_snake_case)]

use egui::{Color32, Pos2, Stroke, Ui, Vec2};

// Helper function to draw an SVG-like path in egui
fn draw_svg_path(ui: &mut Ui, points: &[Vec2], closed: bool, stroke: Stroke) {
    let painter = ui.painter();
    // Convert Vec2 to Pos2 for the line function
    let pos_points: Vec<Pos2> = points.iter().map(|&v| Pos2::new(v.x, v.y)).collect();
    painter.add(egui::Shape::line(pos_points, stroke));
    if closed && points.len() > 2 {
        painter.add(egui::Shape::line(
            vec![
                Pos2::new(points[points.len() - 1].x, points[points.len() - 1].y),
                Pos2::new(points[0].x, points[0].y),
            ],
            stroke,
        ));
    }
}

pub fn draw_caret_left(ui: &mut Ui, rect: egui::Rect, color: Color32) {
    let stroke = Stroke::new(2.0, color);
    let size = rect.size();

    let points = vec![
        Vec2::new(rect.center().x - size.x * 0.2, rect.center().y),
        Vec2::new(
            rect.center().x + size.x * 0.2,
            rect.center().y - size.y * 0.3,
        ),
        Vec2::new(
            rect.center().x + size.x * 0.2,
            rect.center().y + size.y * 0.3,
        ),
    ];

    draw_svg_path(ui, &points, true, stroke);
}

#[allow(dead_code)]
pub fn draw_wallet(ui: &mut Ui, rect: egui::Rect, color: Color32) {
    let stroke = Stroke::new(2.0, color);
    let size = rect.size();

    // Draw wallet body
    let wallet_rect = egui::Rect::from_min_size(
        rect.min + Vec2::new(size.x * 0.1, size.y * 0.2),
        Vec2::new(size.x * 0.8, size.y * 0.6),
    );
    ui.painter().rect_stroke(wallet_rect, 4.0, stroke);

    // Draw wallet flap
    let flap_rect = egui::Rect::from_min_size(
        wallet_rect.right_top() + Vec2::new(-size.x * 0.3, 0.0),
        Vec2::new(size.x * 0.3, size.y * 0.3),
    );
    ui.painter().rect_stroke(flap_rect, 4.0, stroke);
}

#[allow(dead_code)]
pub fn draw_arrow_left(ui: &mut Ui, rect: egui::Rect, color: Color32) {
    let stroke = Stroke::new(2.0, color);
    let size = rect.size();

    // Draw horizontal line
    let h_line = vec![
        Vec2::new(rect.center().x - size.x * 0.4, rect.center().y),
        Vec2::new(rect.center().x + size.x * 0.4, rect.center().y),
    ];
    draw_svg_path(ui, &h_line, false, stroke);

    // Draw arrow head
    let arrow_head = vec![
        Vec2::new(
            rect.center().x - size.x * 0.2,
            rect.center().y - size.y * 0.2,
        ),
        Vec2::new(rect.center().x - size.x * 0.4, rect.center().y),
        Vec2::new(
            rect.center().x - size.x * 0.2,
            rect.center().y + size.y * 0.2,
        ),
    ];
    draw_svg_path(ui, &arrow_head, false, stroke);
}
