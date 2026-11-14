use crate::{
    diagonal::{Game, Shape},
    presentation::{Hud, RenderScene},
    voxel::VoxelVertex,
};
use glam::{Mat4, Vec3};

impl RenderScene for Game {
    fn revision(&self) -> u64 {
        self.world.revision
    }
    fn mesh(&self) -> Vec<VoxelVertex> {
        self.world.mesh()
    }
    fn view_projection(&self, aspect: f32) -> Mat4 {
        self.player.view_projection(aspect)
    }
    fn eye(&self) -> Vec3 {
        self.player.eye()
    }
    fn selection(&self) -> [f32; 4] {
        [
            self.target().map_or(-1., |hit| hit.index as f32 + 1.),
            0.,
            0.,
            2.,
        ]
    }
    fn preview(&self) -> Vec<VoxelVertex> {
        let Some(mut candidate) = self.candidate() else {
            return Vec::new();
        };
        candidate.color = if self.can_place(&candidate) {
            [0.40, 0.95, 0.73]
        } else {
            [1., 0.24, 0.18]
        };
        let mut vertices = candidate.mesh(self.world.pieces.len());
        for v in &mut vertices {
            v.padding[1] = 1.;
        }
        vertices
    }
    fn hud(&self, width: f32, height: f32, captured: bool) -> Vec<VoxelVertex> {
        let (mut hud, w, h) = Hud::new(width, height);
        let dark = [0.025, 0.045, 0.055, 1.];
        let white = [0.91, 0.95, 0.92, 1.];
        let mint = [0.36, 0.85, 0.70, 1.];
        let muted = [0.61, 0.72, 0.72, 1.];
        hud.rect(16., 16., 296., 53., dark);
        hud.rect(16., 16., 3., 53., mint);
        hud.text("DIAGONAL WORLD", 28., 27., white);
        hud.text("5 - 12 - 13   A DIFFERENT KIND OF BLOCK", 28., 43., mint);
        hud.text("CUBES. BOXES. REAL SLOPES.", 28., 55., muted);
        hud.rect(w - 222., 16., 206., 53., dark);
        hud.text("12 CUBES = 5 BOXES = 60 UNITS", w - 212., 27., white);
        hud.text("0   22.62   45   67.38   90", w - 212., 43., mint);
        hud.text(
            if self.player.flying {
                "FLY MODE   SPACE UP   SHIFT DOWN"
            } else {
                "WALK MODE   SPACE JUMP"
            },
            w - 212.,
            55.,
            muted,
        );

        let slot_width = 98.;
        let start = w / 2. - 6. * slot_width / 2.;
        for (index, shape) in Shape::ALL.iter().enumerate() {
            let x = start + index as f32 * slot_width;
            let selected = self.selected == index;
            hud.rect(
                x,
                h - 101.,
                94.,
                54.,
                if selected {
                    mint
                } else {
                    [0.13, 0.20, 0.20, 1.]
                },
            );
            hud.rect(x + 1., h - 100., 92., 52., dark);
            let c = shape.color();
            hud.rect(x + 8., h - 92., 13., 13., [c[0], c[1], c[2], 1.]);
            hud.text(&format!("{}", index + 1), x + 79., h - 90., muted);
            hud.text(
                shape.name(),
                x + 8.,
                h - 72.,
                if selected { mint } else { white },
            );
            let d = shape.dimensions();
            hud.text(
                &format!("{} X {} X {}", d.x, d.y, d.z),
                x + 8.,
                h - 60.,
                muted,
            );
        }
        hud.rect(start, h - 122., 6. * slot_width - 4., 16., dark);
        hud.text(
            &format!(
                "R ROTATE {}   F FLY   V OVERVIEW   T RESPAWN   UNLIMITED PIECES",
                self.rotation as u16 * 90
            ),
            start + 8.,
            h - 118.,
            muted,
        );
        hud.rect(16., h - 39., w - 32., 23., dark);
        hud.text(
            if captured {
                "WASD MOVE   MOUSE/ARROWS LOOK   H LEVEL VIEW   ESC RELEASE"
            } else {
                "CLICK TO PLAY   WASD MOVE   MOUSE/ARROWS LOOK   H LEVEL   Q QUIT"
            },
            28.,
            h - 31.,
            white,
        );
        hud.rect(w / 2. - 7., h / 2. - 1., 14., 2., white);
        hud.rect(w / 2. - 1., h / 2. - 7., 2., 14., white);
        if let Some(hit) = self.target() {
            let label = format!(
                "{}   {:.1} UNITS",
                self.world.pieces[hit.index].shape.name(),
                hit.distance * 5.
            );
            hud.rect(w / 2. - 118., h / 2. + 18., 236., 17., dark);
            hud.text(
                &label,
                w / 2. - label.len() as f32 * 3.,
                h / 2. + 23.,
                white,
            );
        }
        hud.rect(
            16.,
            78.,
            (self.message.len() as f32 * 6. + 16.).min(w - 32.),
            17.,
            dark,
        );
        hud.text(&self.message, 24., 83., muted);
        hud.vertices
    }
}
