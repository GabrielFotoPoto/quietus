//! Generation en pur Rust (sans dependance externe) de l'icone RGBA 64x64
//! de Quietus pour la barre de titre et la barre des taches Windows.

use eframe::egui::IconData;

const SIZE: u32 = 64;

fn dist(x: f32, y: f32, cx: f32, cy: f32) -> f32 {
    ((x - cx).powi(2) + (y - cy).powi(2)).sqrt()
}

fn in_rounded_rect(x: f32, y: f32, w: f32, h: f32, r: f32) -> bool {
    if x < 0.0 || y < 0.0 || x >= w || y >= h {
        return false;
    }
    let dx = if x < r {
        r - x
    } else if x > w - r {
        x - (w - r)
    } else {
        0.0
    };
    let dy = if y < r {
        r - y
    } else if y > h - r {
        y - (h - r)
    } else {
        0.0
    };
    (dx * dx + dy * dy).sqrt() <= r
}

fn dist_to_segment(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let vx = bx - ax;
    let vy = by - ay;
    let len2 = vx * vx + vy * vy;
    let t = if len2 > 0.0 {
        (((px - ax) * vx + (py - ay) * vy) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    dist(px, py, ax + t * vx, ay + t * vy)
}

/// Construit l'icone `IconData` 64x64 coherente avec `docs/logo/quietus.svg`.
pub fn app_icon() -> IconData {
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

    for py in 0..SIZE {
        for px in 0..SIZE {
            let x = px as f32 + 0.5;
            let y = py as f32 + 0.5;
            let idx = ((py * SIZE + px) * 4) as usize;

            if !in_rounded_rect(x, y, 64.0, 64.0, 14.0) {
                continue;
            }

            // Fond bleu Kikan / Quietus (#1d428a)
            let mut color = [0x1d, 0x42, 0x8a, 0xff];

            // Anneau du Q blanc (centre 28,28, rayon 15, epaisseur 7.5)
            let d_q = (dist(x, y, 28.0, 28.0) - 15.0).abs();
            let d_tail = dist_to_segment(x, y, 35.0, 35.0, 45.0, 45.0);
            if d_q <= 3.75 || d_tail <= 3.75 {
                color = [0xff, 0xff, 0xff, 0xff];
            }

            // Onde au coeur du Q (barres ambre #d2aa3c + trait vert #50b450)
            let bar1 = (21.0..=24.2).contains(&x) && (24.0..=32.0).contains(&y);
            let bar2 = (26.0..=29.2).contains(&x) && (20.0..=36.0).contains(&y);
            if bar1 || bar2 {
                color = [0xd2, 0xaa, 0x3c, 0xff];
            } else if dist_to_segment(x, y, 31.5, 28.0, 36.5, 28.0) <= 1.7 {
                color = [0x50, 0xb4, 0x50, 0xff];
            }

            // Medaillon d'angle en bas a droite (centre 46,46)
            let d_med = dist(x, y, 46.0, 46.0);
            if d_med <= 13.0 {
                color = [0x1d, 0x42, 0x8a, 0xff];
            }
            if d_med <= 10.5 {
                color = [0xd8, 0x60, 0x18, 0xff];
                // Coche blanche dans le medaillon
                let c1 = dist_to_segment(x, y, 41.2, 46.4, 44.8, 49.8);
                let c2 = dist_to_segment(x, y, 44.8, 49.8, 51.2, 43.0);
                if c1 <= 1.6 || c2 <= 1.6 {
                    color = [0xff, 0xff, 0xff, 0xff];
                }
            }

            rgba[idx..idx + 4].copy_from_slice(&color);
        }
    }

    IconData {
        rgba,
        width: SIZE,
        height: SIZE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_icon_has_valid_64x64_rgba_buffer() {
        let icon = app_icon();
        assert_eq!(icon.width, 64);
        assert_eq!(icon.height, 64);
        assert_eq!(icon.rgba.len(), 64 * 64 * 4);
        // Coin (0,0) hors du carre arrondi -> transparent
        assert_eq!(icon.rgba[3], 0);
        // Centre du medaillon (46,46) -> opaque
        let center_idx = ((46 * 64 + 46) * 4) as usize;
        assert_eq!(icon.rgba[center_idx + 3], 255);
    }
}
