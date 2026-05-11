use macroquad::prelude::*;

const GRID_SIZE: usize = 16;
const MINE_COUNT: usize = 40;
const CELL_SIZE: f32 = 32.0;
const PADDING: f32 = 40.0;

#[derive(Clone, Copy, PartialEq)]
enum CellState {
    Hidden,
    Revealed,
    Flagged,
}

#[derive(Clone, Copy)]
struct Cell {
    is_mine: bool,
    state: CellState,
    adjacent_mines: u8,
}

enum GameState {
    Playing,
    Won,
    Lost,
}

struct Game {
    board: [[Cell; GRID_SIZE]; GRID_SIZE],
    state: GameState,
    first_click: bool,
}

impl Game {
    fn new() -> Self {
        let cell = Cell {
            is_mine: false,
            state: CellState::Hidden,
            adjacent_mines: 0,
        };
        Self {
            board: [[cell; GRID_SIZE]; GRID_SIZE],
            state: GameState::Playing,
            first_click: true,
        }
    }

    fn place_mines(&mut self, exclude_x: usize, exclude_y: usize) {
        let mut placed = 0;
        while placed < MINE_COUNT {
            let x = rand::gen_range(0, GRID_SIZE);
            let y = rand::gen_range(0, GRID_SIZE);
            if (x == exclude_x && y == exclude_y) || self.board[y][x].is_mine {
                continue;
            }
            self.board[y][x].is_mine = true;
            placed += 1;
        }
        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                if !self.board[y][x].is_mine {
                    self.board[y][x].adjacent_mines = self.count_adjacent(x, y);
                }
            }
        }
    }

    fn count_adjacent(&self, x: usize, y: usize) -> u8 {
        let mut count = 0u8;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && nx < GRID_SIZE as i32 && ny >= 0 && ny < GRID_SIZE as i32 {
                    if self.board[ny as usize][nx as usize].is_mine {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn reveal(&mut self, x: usize, y: usize) {
        if x >= GRID_SIZE || y >= GRID_SIZE {
            return;
        }
        if self.board[y][x].state != CellState::Hidden {
            return;
        }

        if self.first_click {
            self.place_mines(x, y);
            self.first_click = false;
        }

        if self.board[y][x].is_mine {
            self.state = GameState::Lost;
            for row in &mut self.board {
                for cell in row {
                    if cell.is_mine {
                        cell.state = CellState::Revealed;
                    }
                }
            }
            return;
        }

        self.board[y][x].state = CellState::Revealed;

        if self.board[y][x].adjacent_mines == 0 {
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < GRID_SIZE as i32 && ny >= 0 && ny < GRID_SIZE as i32 {
                        self.reveal(nx as usize, ny as usize);
                    }
                }
            }
        }

        self.check_win();
    }

    fn chord(&mut self, x: usize, y: usize) {
        if x >= GRID_SIZE || y >= GRID_SIZE {
            return;
        }
        let cell = self.board[y][x];
        if cell.state != CellState::Revealed || cell.adjacent_mines == 0 {
            return;
        }
        let mut flag_count = 0u8;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && nx < GRID_SIZE as i32 && ny >= 0 && ny < GRID_SIZE as i32 {
                    if self.board[ny as usize][nx as usize].state == CellState::Flagged {
                        flag_count += 1;
                    }
                }
            }
        }
        if flag_count == cell.adjacent_mines {
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < GRID_SIZE as i32 && ny >= 0 && ny < GRID_SIZE as i32 {
                        self.reveal(nx as usize, ny as usize);
                    }
                }
            }
        }
    }

    fn toggle_flag(&mut self, x: usize, y: usize) {
        if x >= GRID_SIZE || y >= GRID_SIZE {
            return;
        }
        match self.board[y][x].state {
            CellState::Hidden => self.board[y][x].state = CellState::Flagged,
            CellState::Flagged => self.board[y][x].state = CellState::Hidden,
            _ => {}
        }
    }

    fn check_win(&mut self) {
        for row in &self.board {
            for cell in row {
                if !cell.is_mine && cell.state != CellState::Revealed {
                    return;
                }
            }
        }
        self.state = GameState::Won;
    }

    fn flags_remaining(&self) -> i32 {
        let flagged: i32 = self
            .board
            .iter()
            .flatten()
            .filter(|c| c.state == CellState::Flagged)
            .count() as i32;
        MINE_COUNT as i32 - flagged
    }
}

fn number_color(n: u8) -> Color {
    match n {
        1 => BLUE,
        2 => GREEN,
        3 => RED,
        4 => DARKBLUE,
        5 => MAROON,
        6 => Color::new(0.0, 0.5, 0.5, 1.0),
        7 => BLACK,
        8 => GRAY,
        _ => BLACK,
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Minesweeper".to_string(),
        window_width: (GRID_SIZE as i32) * (CELL_SIZE as i32) + (PADDING as i32) * 2,
        window_height: (GRID_SIZE as i32) * (CELL_SIZE as i32) + (PADDING as i32) * 2 + 40,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();

    loop {
        clear_background(Color::new(0.9, 0.9, 0.9, 1.0));

        let header_text = match game.state {
            GameState::Playing => format!(
                "Mines: {}  |  L: reveal  |  R: flag  |  L+R: chord",
                game.flags_remaining()
            ),
            GameState::Won => "You Won! Click to play again.".to_string(),
            GameState::Lost => "Game Over! Click to play again.".to_string(),
        };
        draw_text(&header_text, PADDING, 28.0, 20.0, DARKGRAY);

        let offset_y = PADDING + 20.0;

        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let px = PADDING + x as f32 * CELL_SIZE;
                let py = offset_y + y as f32 * CELL_SIZE;
                let cell = game.board[y][x];

                match cell.state {
                    CellState::Hidden => {
                        draw_rectangle(
                            px,
                            py,
                            CELL_SIZE - 1.0,
                            CELL_SIZE - 1.0,
                            Color::new(0.6, 0.7, 0.8, 1.0),
                        );
                    }
                    CellState::Flagged => {
                        draw_rectangle(
                            px,
                            py,
                            CELL_SIZE - 1.0,
                            CELL_SIZE - 1.0,
                            Color::new(0.6, 0.7, 0.8, 1.0),
                        );
                        draw_text("F", px + 10.0, py + 22.0, 24.0, RED);
                    }
                    CellState::Revealed => {
                        draw_rectangle(
                            px,
                            py,
                            CELL_SIZE - 1.0,
                            CELL_SIZE - 1.0,
                            Color::new(0.85, 0.85, 0.85, 1.0),
                        );
                        if cell.is_mine {
                            draw_text("*", px + 8.0, py + 24.0, 28.0, BLACK);
                        } else if cell.adjacent_mines > 0 {
                            let txt = format!("{}", cell.adjacent_mines);
                            draw_text(
                                &txt,
                                px + 10.0,
                                py + 22.0,
                                24.0,
                                number_color(cell.adjacent_mines),
                            );
                        }
                    }
                }
            }
        }

        let left_down = is_mouse_button_down(MouseButton::Left);
        let right_down = is_mouse_button_down(MouseButton::Right);
        let left_pressed = is_mouse_button_pressed(MouseButton::Left);
        let right_pressed = is_mouse_button_pressed(MouseButton::Right);
        let both_held = left_down && right_down;
        let chord_triggered =
            both_held && (left_pressed || right_pressed);

        if chord_triggered {
            if matches!(game.state, GameState::Playing) {
                let (mx, my) = mouse_position();
                let gx = ((mx - PADDING) / CELL_SIZE) as i32;
                let gy = ((my - offset_y) / CELL_SIZE) as i32;
                if gx >= 0 && gx < GRID_SIZE as i32 && gy >= 0 && gy < GRID_SIZE as i32 {
                    game.chord(gx as usize, gy as usize);
                }
            }
        } else if left_pressed && !right_down {
            let (mx, my) = mouse_position();
            match game.state {
                GameState::Playing => {
                    let gx = ((mx - PADDING) / CELL_SIZE) as i32;
                    let gy = ((my - offset_y) / CELL_SIZE) as i32;
                    if gx >= 0 && gx < GRID_SIZE as i32 && gy >= 0 && gy < GRID_SIZE as i32 {
                        game.reveal(gx as usize, gy as usize);
                    }
                }
                _ => {
                    game = Game::new();
                }
            }
        } else if right_pressed && !left_down {
            if matches!(game.state, GameState::Playing) {
                let (mx, my) = mouse_position();
                let gx = ((mx - PADDING) / CELL_SIZE) as i32;
                let gy = ((my - offset_y) / CELL_SIZE) as i32;
                if gx >= 0 && gx < GRID_SIZE as i32 && gy >= 0 && gy < GRID_SIZE as i32 {
                    game.toggle_flag(gx as usize, gy as usize);
                }
            }
        }

        next_frame().await
    }
}
