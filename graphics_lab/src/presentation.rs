use crate::voxel::{Block, Game, VoxelVertex};
/// Both native block demos share the GPU pipeline and window-independent capture.
pub trait RenderScene {
    fn revision(&self) -> u64;
    fn mesh(&self) -> Vec<VoxelVertex>;
    fn view_projection(&self, aspect: f32) -> glam::Mat4;
    fn eye(&self) -> glam::Vec3;
    fn selection(&self) -> [f32; 4];
    fn hud(&self, width: f32, height: f32, captured: bool) -> Vec<VoxelVertex>;
    fn preview(&self) -> Vec<VoxelVertex> {
        Vec::new()
    }
}

impl RenderScene for Game {
    fn revision(&self) -> u64 {
        self.world.revision
    }
    fn mesh(&self) -> Vec<VoxelVertex> {
        self.world.mesh()
    }
    fn view_projection(&self, aspect: f32) -> glam::Mat4 {
        self.player.view_projection(aspect)
    }
    fn eye(&self) -> glam::Vec3 {
        self.player.eye()
    }
    fn selection(&self) -> [f32; 4] {
        self.target()
            .map_or([0.; 4], |hit| hit.cell.as_vec3().extend(1.).to_array())
    }
    fn hud(&self, width: f32, height: f32, captured: bool) -> Vec<VoxelVertex> {
        hud(self, width, height, captured)
    }
}

pub(crate) struct Hud {
    pub(crate) vertices: Vec<VoxelVertex>,
    width: f32,
    height: f32,
    scale: f32,
}
impl Hud {
    pub(crate) fn new(width: f32, height: f32) -> (Self, f32, f32) {
        let scale = (width / 800.).min(height / 500.).max(0.4);
        (
            Self {
                vertices: Vec::new(),
                width,
                height,
                scale,
            },
            width / scale,
            height / scale,
        )
    }
    pub(crate) fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        for [u, v] in [[0., 0.], [1., 0.], [1., 1.], [0., 0.], [1., 1.], [0., 1.]] {
            self.vertices.push(VoxelVertex {
                position: [
                    (x + u * w) * self.scale / self.width * 2.0 - 1.0,
                    1.0 - (y + v * h) * self.scale / self.height * 2.0,
                    0.0,
                    1.0,
                ],
                normal: [0.0; 4],
                color,
                uv: [0.0; 2],
                padding: [0.0; 2],
            });
        }
    }
    pub(crate) fn text(&mut self, text: &str, x: f32, y: f32, color: [f32; 4]) {
        for (i, ch) in text.to_ascii_uppercase().chars().enumerate() {
            let rows = glyph(ch);
            for (row, bits) in rows.into_iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        self.rect(
                            x + i as f32 * 6.0 + col as f32,
                            y + row as f32,
                            1.0,
                            1.0,
                            color,
                        );
                    }
                }
            }
        }
    }
}
fn hud(game: &Game, width: f32, height: f32, captured: bool) -> Vec<VoxelVertex> {
    let scale = (width / 640.0).min(height / 400.0).max(0.5);
    let (w, h) = (width / scale, height / scale);
    let mut hud = Hud {
        vertices: Vec::new(),
        width,
        height,
        scale,
    };
    let white = [0.95, 0.97, 1.0, 1.0];
    let dark = [0.035, 0.05, 0.065, 1.0];
    let gold = [1.0, 0.7, 0.2, 1.0];
    hud.rect(10., 10., 370., 33., dark);
    hud.text("BLOCK WORLD", 18., 17., gold);
    hud.text(
        "WASD MOVE   SPACE JUMP   MOUSE/ARROWS LOOK",
        18.,
        30.,
        white,
    );
    hud.rect(10., h - 23., 370., 15., dark);
    hud.text(
        if captured {
            "LEFT DIG   RIGHT PLACE   1-4 SELECT   ESC RELEASE"
        } else {
            "CLICK TO PLAY   WASD MOVE   ESC RELEASE   Q QUIT"
        },
        18.,
        h - 19.,
        white,
    );
    for (slot, block) in Block::ALL.into_iter().enumerate() {
        let x = w * 0.5 - 140.0 + slot as f32 * 72.0;
        let selected = slot == game.selected;
        hud.rect(
            x,
            h - 88.,
            68.,
            55.,
            if selected {
                gold
            } else {
                [0.1, 0.13, 0.16, 1.0]
            },
        );
        hud.rect(x + 2., h - 86., 64., 51., dark);
        let c = block.color();
        hud.rect(x + 8., h - 78., 16., 16., [c[0], c[1], c[2], 1.0]);
        hud.text(&format!("{}", slot + 1), x + 51., h - 78., white);
        hud.text(
            block.name(),
            x + 8.,
            h - 57.,
            if selected { gold } else { white },
        );
        hud.text(&game.inventory[slot].to_string(), x + 8., h - 46., white);
    }
    hud.rect(w * 0.5 - 7., h * 0.5 - 2., 14., 4., dark);
    hud.rect(w * 0.5 - 2., h * 0.5 - 7., 4., 14., dark);
    hud.rect(w * 0.5 - 6., h * 0.5 - 0.5, 12., 1., white);
    hud.rect(w * 0.5 - 0.5, h * 0.5 - 6., 1., 12., white);
    if let Some(hit) = game.target() {
        let label = format!("{}  {:.1}M", hit.block.name(), hit.distance);
        hud.text(
            &label,
            w * 0.5 - label.len() as f32 * 3.0,
            h * 0.5 + 16.,
            white,
        );
    }
    hud.vertices
}
fn glyph(c: char) -> [u8; 7] {
    match c {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '=' => [0, 0, 31, 0, 31, 0, 0],
        '.' => [0, 0, 0, 0, 0, 12, 12],
        _ => [0; 7],
    }
}
