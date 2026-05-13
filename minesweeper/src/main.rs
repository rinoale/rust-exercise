use macroquad::prelude::*;

const CELL_SIZE: f32 = 34.0;
const PADDING: f32 = 32.0;
const HUD_HEIGHT: f32 = 74.0;
const CONTROLS_HEIGHT: f32 = 104.0;
const CONTROLS_GAP: f32 = 10.0;
const HUD_GAP: f32 = 14.0;
const MIN_WINDOW_WIDTH: f32 = 960.0;

const MIN_CUSTOM_WIDTH: usize = 5;
const MAX_CUSTOM_WIDTH: usize = 30;
const MIN_CUSTOM_HEIGHT: usize = 5;
const MAX_CUSTOM_HEIGHT: usize = 20;

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

impl Cell {
    fn empty() -> Self {
        Self {
            is_mine: false,
            state: CellState::Hidden,
            adjacent_mines: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Playing,
    Won,
    Lost,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Beginner,
    Intermediate,
    Advanced,
    Custom,
}

#[derive(Clone, Copy)]
struct BoardConfig {
    width: usize,
    height: usize,
    mines: usize,
}

#[derive(Clone, Copy)]
struct CustomConfig {
    width: usize,
    height: usize,
    mines: usize,
}

impl CustomConfig {
    fn default() -> Self {
        Self {
            width: 16,
            height: 16,
            mines: 40,
        }
    }

    fn clamped(mut self) -> Self {
        self.width = self.width.clamp(MIN_CUSTOM_WIDTH, MAX_CUSTOM_WIDTH);
        self.height = self.height.clamp(MIN_CUSTOM_HEIGHT, MAX_CUSTOM_HEIGHT);
        self.mines = self.mines.clamp(1, max_mines_for(self.width, self.height));
        self
    }
}

#[derive(Clone, Copy, PartialEq)]
enum CustomField {
    Width,
    Height,
    Mines,
}

#[derive(Clone, Copy, PartialEq)]
enum ButtonAction {
    Reset,
    Mode(Mode),
    AdjustCustom(CustomField, i32),
}

#[derive(Clone, Copy, PartialEq)]
enum HeldAction {
    Reveal(usize, usize),
    Flag(usize, usize),
    Chord(usize, usize),
    Button(ButtonAction),
}

struct Game {
    board: Vec<Cell>,
    width: usize,
    height: usize,
    mine_count: usize,
    mode: Mode,
    custom: CustomConfig,
    state: GameState,
    first_click: bool,
    started_at: Option<f64>,
    elapsed_at_end: f64,
}

impl Game {
    fn new(mode: Mode, custom: CustomConfig) -> Self {
        let custom = custom.clamped();
        let config = config_for(mode, custom);
        Self {
            board: vec![Cell::empty(); config.width * config.height],
            width: config.width,
            height: config.height,
            mine_count: config.mines,
            mode,
            custom,
            state: GameState::Playing,
            first_click: true,
            started_at: None,
            elapsed_at_end: 0.0,
        }
    }

    fn restart(&mut self) {
        *self = Self::new(self.mode, self.custom);
    }

    fn set_mode(&mut self, mode: Mode) {
        *self = Self::new(mode, self.custom);
    }

    fn adjust_custom(&mut self, field: CustomField, delta: i32) {
        let mut custom = self.custom;
        match field {
            CustomField::Width => {
                custom.width = adjusted_usize(custom.width, delta, MIN_CUSTOM_WIDTH, MAX_CUSTOM_WIDTH);
            }
            CustomField::Height => {
                custom.height =
                    adjusted_usize(custom.height, delta, MIN_CUSTOM_HEIGHT, MAX_CUSTOM_HEIGHT);
            }
            CustomField::Mines => {
                custom.mines = adjusted_usize(
                    custom.mines,
                    delta,
                    1,
                    max_mines_for(custom.width, custom.height),
                );
            }
        }
        *self = Self::new(Mode::Custom, custom.clamped());
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn cell(&self, x: usize, y: usize) -> Cell {
        self.board[self.index(x, y)]
    }

    fn place_mines(&mut self, exclude_x: usize, exclude_y: usize) {
        let mut placed = 0;
        while placed < self.mine_count {
            let x = rand::gen_range(0, self.width);
            let y = rand::gen_range(0, self.height);
            let index = self.index(x, y);
            if (x == exclude_x && y == exclude_y) || self.board[index].is_mine {
                continue;
            }
            self.board[index].is_mine = true;
            placed += 1;
        }
        for y in 0..self.height {
            for x in 0..self.width {
                let index = self.index(x, y);
                if !self.board[index].is_mine {
                    self.board[index].adjacent_mines = self.count_adjacent(x, y);
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
                if let Some((nx, ny)) = self.neighbor_coord(x, y, dx, dy) {
                    if self.cell(nx, ny).is_mine {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn neighbor_coord(&self, x: usize, y: usize, dx: i32, dy: i32) -> Option<(usize, usize)> {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
            Some((nx as usize, ny as usize))
        } else {
            None
        }
    }

    fn reveal(&mut self, x: usize, y: usize) {
        if !matches!(self.state, GameState::Playing) || x >= self.width || y >= self.height {
            return;
        }
        let index = self.index(x, y);
        if self.board[index].state != CellState::Hidden {
            return;
        }

        if self.first_click {
            self.place_mines(x, y);
            self.started_at = Some(get_time());
            self.first_click = false;
        }

        if self.board[index].is_mine {
            self.state = GameState::Lost;
            self.freeze_timer();
            for cell in &mut self.board {
                if cell.is_mine {
                    cell.state = CellState::Revealed;
                }
            }
            return;
        }

        self.board[index].state = CellState::Revealed;

        if self.board[index].adjacent_mines == 0 {
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    if let Some((nx, ny)) = self.neighbor_coord(x, y, dx, dy) {
                        self.reveal(nx, ny);
                    }
                }
            }
        }

        self.check_win();
    }

    fn chord(&mut self, x: usize, y: usize) {
        if !matches!(self.state, GameState::Playing) || x >= self.width || y >= self.height {
            return;
        }
        let cell = self.cell(x, y);
        if cell.state != CellState::Revealed || cell.adjacent_mines == 0 {
            return;
        }
        let mut flag_count = 0u8;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if let Some((nx, ny)) = self.neighbor_coord(x, y, dx, dy) {
                    if self.cell(nx, ny).state == CellState::Flagged {
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
                    if let Some((nx, ny)) = self.neighbor_coord(x, y, dx, dy) {
                        self.reveal(nx, ny);
                    }
                }
            }
        }
    }

    fn toggle_flag(&mut self, x: usize, y: usize) {
        if !matches!(self.state, GameState::Playing) || x >= self.width || y >= self.height {
            return;
        }
        let index = self.index(x, y);
        match self.board[index].state {
            CellState::Hidden => self.board[index].state = CellState::Flagged,
            CellState::Flagged => self.board[index].state = CellState::Hidden,
            _ => {}
        }
    }

    fn check_win(&mut self) {
        if !matches!(self.state, GameState::Playing) {
            return;
        }
        for cell in &self.board {
            if !cell.is_mine && cell.state != CellState::Revealed {
                return;
            }
        }
        self.state = GameState::Won;
        self.freeze_timer();
    }

    fn freeze_timer(&mut self) {
        self.elapsed_at_end = self
            .started_at
            .map(|started_at| get_time() - started_at)
            .unwrap_or(0.0);
    }

    fn elapsed_seconds(&self) -> i32 {
        let elapsed = match self.state {
            GameState::Playing => self
                .started_at
                .map(|started_at| get_time() - started_at)
                .unwrap_or(0.0),
            GameState::Won | GameState::Lost => self.elapsed_at_end,
        };
        elapsed.floor().clamp(0.0, 999.0) as i32
    }

    fn flags_remaining(&self) -> i32 {
        let flagged = self
            .board
            .iter()
            .filter(|c| c.state == CellState::Flagged)
            .count() as i32;
        self.mine_count as i32 - flagged
    }
}

#[derive(Clone, Copy)]
struct Layout {
    panel_x: f32,
    panel_w: f32,
    hud_y: f32,
    controls_y: f32,
    board_x: f32,
    board_y: f32,
    board_w: f32,
    board_h: f32,
}

fn adjusted_usize(value: usize, delta: i32, min: usize, max: usize) -> usize {
    (value as i32 + delta).clamp(min as i32, max as i32) as usize
}

fn max_mines_for(width: usize, height: usize) -> usize {
    width * height - 1
}

fn config_for(mode: Mode, custom: CustomConfig) -> BoardConfig {
    match mode {
        Mode::Beginner => BoardConfig {
            width: 9,
            height: 9,
            mines: 10,
        },
        Mode::Intermediate => BoardConfig {
            width: 16,
            height: 16,
            mines: 40,
        },
        Mode::Advanced => BoardConfig {
            width: 30,
            height: 16,
            mines: 99,
        },
        Mode::Custom => {
            let custom = custom.clamped();
            BoardConfig {
                width: custom.width,
                height: custom.height,
                mines: custom.mines,
            }
        }
    }
}

fn mode_label(mode: Mode) -> &'static str {
    match mode {
        Mode::Beginner => "BEGINNER",
        Mode::Intermediate => "INTERMEDIATE",
        Mode::Advanced => "ADVANCED",
        Mode::Custom => "CUSTOM",
    }
}

fn target_window_width(game: &Game) -> f32 {
    (game.width as f32 * CELL_SIZE + PADDING * 2.0).max(MIN_WINDOW_WIDTH)
}

fn target_window_height(game: &Game) -> f32 {
    PADDING
        + HUD_HEIGHT
        + CONTROLS_GAP
        + CONTROLS_HEIGHT
        + HUD_GAP
        + game.height as f32 * CELL_SIZE
        + PADDING
}

fn sync_window_size(game: &Game) {
    request_new_screen_size(target_window_width(game), target_window_height(game));
}

fn layout_for(game: &Game) -> Layout {
    let board_w = game.width as f32 * CELL_SIZE;
    let board_h = game.height as f32 * CELL_SIZE;
    let screen_w = screen_width().max(target_window_width(game));
    let panel_w = screen_w - PADDING * 2.0;
    let board_x = (screen_w - board_w) * 0.5;
    let hud_y = PADDING;
    let controls_y = hud_y + HUD_HEIGHT + CONTROLS_GAP;
    let board_y = controls_y + CONTROLS_HEIGHT + HUD_GAP;

    Layout {
        panel_x: PADDING,
        panel_w,
        hud_y,
        controls_y,
        board_x,
        board_y,
        board_w,
        board_h,
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

fn shade(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r * amount).clamp(0.0, 1.0),
        g: (color.g * amount).clamp(0.0, 1.0),
        b: (color.b * amount).clamp(0.0, 1.0),
        a: color.a,
    }
}

fn number_color(n: u8) -> Color {
    match n {
        1 => rgb(31, 94, 191),
        2 => rgb(34, 135, 50),
        3 => rgb(204, 38, 33),
        4 => rgb(112, 58, 171),
        5 => rgb(142, 68, 42),
        6 => rgb(25, 135, 136),
        7 => rgb(36, 36, 36),
        8 => rgb(108, 108, 108),
        _ => BLACK,
    }
}

fn char_pattern(ch: char) -> [&'static str; 7] {
    match ch {
        '0' => [
            "01110", "10001", "10011", "10101", "11001", "10001", "01110",
        ],
        '1' => [
            "00100", "01100", "00100", "00100", "00100", "00100", "01110",
        ],
        '2' => [
            "11110", "00001", "00001", "01110", "10000", "10000", "11111",
        ],
        '3' => [
            "11110", "00001", "00001", "01110", "00001", "00001", "11110",
        ],
        '4' => [
            "10010", "10010", "10010", "11111", "00010", "00010", "00010",
        ],
        '5' => [
            "11111", "10000", "10000", "11110", "00001", "00001", "11110",
        ],
        '6' => [
            "01111", "10000", "10000", "11110", "10001", "10001", "01110",
        ],
        '7' => [
            "11111", "00001", "00010", "00100", "01000", "01000", "01000",
        ],
        '8' => [
            "01110", "10001", "10001", "01110", "10001", "10001", "01110",
        ],
        '9' => [
            "01110", "10001", "10001", "01111", "00001", "00001", "11110",
        ],
        '-' => [
            "00000", "00000", "00000", "11110", "00000", "00000", "00000",
        ],
        _ => [
            "00000", "00000", "00000", "00000", "00000", "00000", "00000",
        ],
    }
}

fn draw_pixel_char(ch: char, x: f32, y: f32, block: f32, gap: f32, color: Color) {
    let pattern = char_pattern(ch);
    for (shadow_x, shadow_y, fill) in [(1.5, 1.5, shade(color, 0.35)), (0.0, 0.0, color)] {
        for (row, line) in pattern.iter().enumerate() {
            for (col, bit) in line.chars().enumerate() {
                if bit == '1' {
                    draw_rectangle(
                        x + shadow_x + col as f32 * (block + gap),
                        y + shadow_y + row as f32 * (block + gap),
                        block,
                        block,
                        fill,
                    );
                }
            }
        }
    }
}

fn pixel_text_width(text: &str, block: f32, gap: f32, char_gap: f32) -> f32 {
    let len = text.chars().count() as f32;
    len * (5.0 * block + 4.0 * gap) + (len - 1.0).max(0.0) * char_gap
}

fn draw_pixel_text(text: &str, x: f32, y: f32, block: f32, gap: f32, color: Color) {
    let char_gap = block * 1.4;
    let char_width = 5.0 * block + 4.0 * gap;
    for (index, ch) in text.chars().enumerate() {
        draw_pixel_char(
            ch,
            x + index as f32 * (char_width + char_gap),
            y,
            block,
            gap,
            color,
        );
    }
}

fn draw_cell_number(n: u8, x: f32, y: f32) {
    let block = 4.0;
    let gap = 0.75;
    let width = 5.0 * block + 4.0 * gap;
    let height = 7.0 * block + 6.0 * gap;
    draw_pixel_char(
        char::from_digit(n as u32, 10).unwrap(),
        x + (CELL_SIZE - width) * 0.5,
        y + (CELL_SIZE - height) * 0.5 + 0.5,
        block,
        gap,
        number_color(n),
    );
}

fn draw_centered_text_fit(
    text: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    max_size: f32,
    color: Color,
) {
    let mut size = max_size;
    let mut dimensions = measure_text(text, None, size as u16, 1.0);
    while dimensions.width > w - 12.0 && size > 10.0 {
        size -= 1.0;
        dimensions = measure_text(text, None, size as u16, 1.0);
    }
    draw_text(
        text,
        x + (w - dimensions.width) * 0.5,
        y + (h + dimensions.height) * 0.5 - 4.0,
        size,
        color,
    );
}

fn draw_wood_panel(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, rgb(54, 30, 15));
    draw_rectangle(x + 4.0, y + 4.0, w - 8.0, h - 8.0, rgb(126, 76, 31));
    draw_rectangle(x + 4.0, y + 4.0, w - 8.0, 5.0, rgb(192, 126, 50));
    draw_rectangle(x + 4.0, y + h - 9.0, w - 8.0, 5.0, rgb(61, 35, 18));
    draw_rectangle(x + 4.0, y + 4.0, 5.0, h - 8.0, rgb(163, 95, 37));
    draw_rectangle(x + w - 9.0, y + 4.0, 5.0, h - 8.0, rgb(70, 39, 18));

    let mut plank_x = x + 42.0;
    while plank_x < x + w - 16.0 {
        draw_rectangle(plank_x, y + 7.0, 3.0, h - 14.0, rgb(88, 49, 20));
        draw_rectangle(plank_x + 3.0, y + 8.0, 1.0, h - 16.0, rgb(177, 106, 39));
        plank_x += 68.0;
    }

    let mut grain_y = y + 18.0;
    while grain_y < y + h - 12.0 {
        draw_rectangle(x + 12.0, grain_y, w - 24.0, 2.0, rgb(105, 61, 25));
        grain_y += 18.0;
    }
}

fn draw_counter_panel(x: f32, y: f32, w: f32, h: f32, label: &str, value: i32) {
    draw_rectangle(x, y, w, h, rgb(48, 28, 14));
    draw_rectangle(x + 3.0, y + 3.0, w - 6.0, h - 6.0, rgb(34, 25, 18));
    draw_rectangle(x + 3.0, y + 3.0, w - 6.0, 3.0, rgb(18, 13, 9));
    draw_text(label, x + 10.0, y + 15.0, 13.0, rgb(236, 202, 139));

    let text = if value < 0 {
        format!("-{:02}", (-value).min(99))
    } else {
        format!("{:03}", value.min(999))
    };
    let block = 4.8;
    let gap = 1.0;
    let text_width = pixel_text_width(&text, block, gap, block * 1.4);
    draw_pixel_text(
        &text,
        x + w - text_width - 12.0,
        y + 23.0,
        block,
        gap,
        rgb(255, 215, 53),
    );
}

fn draw_button(rect: Rect, label: &str, active: bool, pressed: bool) {
    let base = if active {
        rgb(98, 149, 55)
    } else {
        rgb(111, 69, 31)
    };
    let push = if pressed { 2.0 } else { 0.0 };
    draw_rectangle(rect.x + 3.0, rect.y + 4.0, rect.w, rect.h, rgb(42, 27, 15));
    draw_rectangle(rect.x, rect.y + push, rect.w, rect.h, rgb(45, 30, 15));
    draw_rectangle(
        rect.x + 3.0,
        rect.y + 3.0 + push,
        rect.w - 6.0,
        rect.h - 6.0,
        base,
    );
    draw_rectangle(
        rect.x + 3.0,
        rect.y + 3.0 + push,
        rect.w - 6.0,
        4.0,
        shade(base, if pressed { 0.7 } else { 1.35 }),
    );
    draw_rectangle(
        rect.x + 3.0,
        rect.y + rect.h - 7.0 + push,
        rect.w - 6.0,
        4.0,
        shade(base, 0.55),
    );
    draw_centered_text_fit(
        label,
        rect.x,
        rect.y + push,
        rect.w,
        rect.h,
        18.0,
        rgb(246, 232, 181),
    );
}

fn draw_value_box(rect: Rect, value: usize) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, rgb(45, 30, 15));
    draw_rectangle(
        rect.x + 3.0,
        rect.y + 3.0,
        rect.w - 6.0,
        rect.h - 6.0,
        rgb(35, 26, 19),
    );
    draw_centered_text_fit(
        &value.to_string(),
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        19.0,
        rgb(255, 215, 53),
    );
}

fn reset_button_rect(layout: Layout) -> Rect {
    Rect::new(
        layout.panel_x + layout.panel_w * 0.5 - 48.0,
        layout.hud_y + 13.0,
        96.0,
        48.0,
    )
}

fn mode_button_rects(layout: Layout) -> [(Rect, Mode); 4] {
    let y = layout.controls_y + 12.0;
    let start_x = layout.panel_x + 16.0;
    [
        (Rect::new(start_x, y, 116.0, 32.0), Mode::Beginner),
        (Rect::new(start_x + 124.0, y, 148.0, 32.0), Mode::Intermediate),
        (Rect::new(start_x + 280.0, y, 120.0, 32.0), Mode::Advanced),
        (Rect::new(start_x + 408.0, y, 112.0, 32.0), Mode::Custom),
    ]
}

fn custom_button_rects(layout: Layout) -> [(Rect, ButtonAction); 6] {
    let y = layout.controls_y + 58.0;
    let start_x = layout.panel_x + 152.0;
    [
        (
            Rect::new(start_x + 22.0, y, 28.0, 28.0),
            ButtonAction::AdjustCustom(CustomField::Width, -1),
        ),
        (
            Rect::new(start_x + 114.0, y, 28.0, 28.0),
            ButtonAction::AdjustCustom(CustomField::Width, 1),
        ),
        (
            Rect::new(start_x + 178.0, y, 28.0, 28.0),
            ButtonAction::AdjustCustom(CustomField::Height, -1),
        ),
        (
            Rect::new(start_x + 270.0, y, 28.0, 28.0),
            ButtonAction::AdjustCustom(CustomField::Height, 1),
        ),
        (
            Rect::new(start_x + 358.0, y, 28.0, 28.0),
            ButtonAction::AdjustCustom(CustomField::Mines, -1),
        ),
        (
            Rect::new(start_x + 450.0, y, 28.0, 28.0),
            ButtonAction::AdjustCustom(CustomField::Mines, 1),
        ),
    ]
}

fn button_action_at(game: &Game, layout: Layout, mouse: Vec2) -> Option<ButtonAction> {
    let reset = reset_button_rect(layout);
    if reset.contains(mouse) {
        return Some(ButtonAction::Reset);
    }

    for (rect, mode) in mode_button_rects(layout) {
        if rect.contains(mouse) {
            return Some(ButtonAction::Mode(mode));
        }
    }

    for (rect, action) in custom_button_rects(layout) {
        if rect.contains(mouse) {
            return Some(action);
        }
    }

    let _ = game;
    None
}

fn apply_button_action(game: &mut Game, action: ButtonAction) {
    match action {
        ButtonAction::Reset => game.restart(),
        ButtonAction::Mode(mode) => game.set_mode(mode),
        ButtonAction::AdjustCustom(field, delta) => game.adjust_custom(field, delta),
    }
    sync_window_size(game);
}

fn draw_hud(game: &Game, layout: Layout, held_action: Option<HeldAction>) {
    let x = layout.panel_x;
    let y = layout.hud_y;
    draw_wood_panel(x, y, layout.panel_w, HUD_HEIGHT);
    draw_counter_panel(x + 16.0, y + 12.0, 132.0, 50.0, "MINES", game.flags_remaining());
    draw_counter_panel(
        x + layout.panel_w - 148.0,
        y + 12.0,
        132.0,
        50.0,
        "TIME",
        game.elapsed_seconds(),
    );

    let reset = reset_button_rect(layout);
    let reset_pressed = held_action == Some(HeldAction::Button(ButtonAction::Reset));
    let active = !matches!(game.state, GameState::Playing);
    draw_button(reset, "RESET", active, reset_pressed);
}

fn draw_controls(game: &Game, layout: Layout, held_action: Option<HeldAction>) {
    let x = layout.panel_x;
    let y = layout.controls_y;
    draw_wood_panel(x, y, layout.panel_w, CONTROLS_HEIGHT);

    for (rect, mode) in mode_button_rects(layout) {
        let action = ButtonAction::Mode(mode);
        draw_button(
            rect,
            mode_label(mode),
            game.mode == mode,
            held_action == Some(HeldAction::Button(action)),
        );
    }

    let mode_summary = format!("{}  {} x {}  {} mines", mode_label(game.mode), game.width, game.height, game.mine_count);
    draw_text(
        &mode_summary,
        x + layout.panel_w - 330.0,
        y + 34.0,
        18.0,
        rgb(246, 232, 181),
    );

    draw_text("CUSTOM", x + 24.0, y + 79.0, 18.0, rgb(236, 202, 139));
    draw_custom_stepper(game, layout, held_action, CustomField::Width, "X", game.custom.width, 0.0);
    draw_custom_stepper(
        game,
        layout,
        held_action,
        CustomField::Height,
        "Y",
        game.custom.height,
        156.0,
    );
    draw_custom_stepper(
        game,
        layout,
        held_action,
        CustomField::Mines,
        "MINES",
        game.custom.mines,
        336.0,
    );
}

fn draw_custom_stepper(
    _game: &Game,
    layout: Layout,
    held_action: Option<HeldAction>,
    field: CustomField,
    label: &str,
    value: usize,
    x_offset: f32,
) {
    let y = layout.controls_y + 58.0;
    let start_x = layout.panel_x + 152.0 + x_offset;
    draw_text(label, start_x - 28.0, y + 20.0, 17.0, rgb(246, 232, 181));

    let minus = Rect::new(start_x + 22.0, y, 28.0, 28.0);
    let value_box = Rect::new(start_x + 55.0, y, 54.0, 28.0);
    let plus = Rect::new(start_x + 114.0, y, 28.0, 28.0);
    let minus_action = ButtonAction::AdjustCustom(field, -1);
    let plus_action = ButtonAction::AdjustCustom(field, 1);

    draw_button(
        minus,
        "-",
        false,
        held_action == Some(HeldAction::Button(minus_action)),
    );
    draw_value_box(value_box, value);
    draw_button(
        plus,
        "+",
        false,
        held_action == Some(HeldAction::Button(plus_action)),
    );
}

fn draw_board_frame(layout: Layout) {
    draw_wood_panel(
        layout.board_x - 10.0,
        layout.board_y - 10.0,
        layout.board_w + 20.0,
        layout.board_h + 20.0,
    );
    draw_rectangle(
        layout.board_x - 3.0,
        layout.board_y - 3.0,
        layout.board_w + 6.0,
        layout.board_h + 6.0,
        rgb(38, 29, 18),
    );
}

fn draw_hidden_tile(x: f32, y: f32, grid_x: usize, grid_y: usize, pressed: bool) {
    let size = CELL_SIZE - 1.0;
    let top_x = x + if pressed { 3.0 } else { 2.0 };
    let top_y = y + if pressed { 5.0 } else { 2.0 };
    let top_w = size - if pressed { 7.0 } else { 6.0 };
    let top_h = size - if pressed { 9.0 } else { 7.0 };
    let base = if pressed {
        rgb(92, 139, 48)
    } else {
        rgb(119, 174, 61)
    };

    draw_rectangle(x, y, size, size, rgb(35, 48, 22));
    draw_rectangle(x + 2.0, y + size - 5.0, size - 3.0, 5.0, rgb(22, 31, 14));
    draw_rectangle(x + size - 5.0, y + 2.0, 5.0, size - 4.0, rgb(28, 38, 17));
    draw_rectangle(top_x, top_y, top_w, top_h, base);
    draw_rectangle(
        top_x,
        top_y,
        top_w,
        4.0,
        if pressed {
            rgb(69, 104, 39)
        } else {
            rgb(167, 214, 76)
        },
    );
    draw_rectangle(
        top_x,
        top_y,
        4.0,
        top_h,
        if pressed {
            rgb(72, 111, 39)
        } else {
            rgb(147, 202, 71)
        },
    );
    draw_rectangle(top_x, top_y + top_h - 4.0, top_w, 4.0, rgb(65, 103, 35));
    draw_rectangle(top_x + top_w - 4.0, top_y, 4.0, top_h, rgb(74, 113, 40));

    let seed = grid_x * 37 + grid_y * 19;
    for i in 0..4 {
        let patch_x = top_x + 6.0 + ((seed + i * 11) % 17) as f32;
        let patch_y = top_y + 6.0 + ((seed / 2 + i * 7) % 15) as f32;
        let patch_color = if i % 2 == 0 {
            rgb(139, 194, 70)
        } else {
            rgb(89, 143, 48)
        };
        draw_rectangle(patch_x, patch_y, 5.0, 4.0, patch_color);
    }
}

fn draw_revealed_tile(x: f32, y: f32, grid_x: usize, grid_y: usize) {
    let size = CELL_SIZE - 1.0;
    draw_rectangle(x, y, size, size, rgb(99, 82, 55));
    draw_rectangle(x + 1.0, y + 1.0, size - 2.0, size - 2.0, rgb(190, 169, 124));
    draw_rectangle(x + 1.0, y + 1.0, size - 2.0, 2.0, rgb(223, 204, 156));
    draw_rectangle(x + 1.0, y + 1.0, 2.0, size - 2.0, rgb(214, 194, 148));
    draw_rectangle(x + 1.0, y + size - 3.0, size - 2.0, 2.0, rgb(137, 117, 82));
    draw_rectangle(x + size - 3.0, y + 1.0, 2.0, size - 2.0, rgb(146, 124, 86));

    let seed = grid_x * 23 + grid_y * 29;
    for i in 0..3 {
        let patch_x = x + 6.0 + ((seed + i * 13) % 19) as f32;
        let patch_y = y + 6.0 + ((seed / 3 + i * 9) % 18) as f32;
        draw_rectangle(
            patch_x,
            patch_y,
            4.0,
            4.0,
            if i % 2 == 0 {
                rgb(179, 157, 111)
            } else {
                rgb(207, 186, 139)
            },
        );
    }
}

fn draw_flag(x: f32, y: f32, pressed: bool) {
    let push = if pressed { 2.0 } else { 0.0 };
    let pole_x = x + CELL_SIZE * 0.45;
    let top_y = y + 8.0 + push;
    let base_y = y + CELL_SIZE - 8.0 + push;
    draw_rectangle(pole_x, top_y, 3.0, base_y - top_y, rgb(79, 58, 31));
    draw_triangle(
        vec2(pole_x + 2.0, top_y),
        vec2(pole_x + 17.0, top_y + 6.0),
        vec2(pole_x + 2.0, top_y + 14.0),
        rgb(213, 36, 28),
    );
    draw_rectangle(pole_x - 7.0, base_y, 17.0, 4.0, rgb(111, 78, 36));
    draw_rectangle(pole_x - 5.0, base_y - 3.0, 13.0, 3.0, rgb(165, 117, 51));
}

fn draw_mine(x: f32, y: f32) {
    let cx = x + CELL_SIZE * 0.5;
    let cy = y + CELL_SIZE * 0.5;
    let dark = rgb(20, 24, 26);
    draw_rectangle(cx - 3.0, y + 7.0, 6.0, CELL_SIZE - 14.0, dark);
    draw_rectangle(x + 7.0, cy - 3.0, CELL_SIZE - 14.0, 6.0, dark);
    draw_rectangle(cx - 2.0, y + 5.0, 4.0, CELL_SIZE - 10.0, rgb(35, 40, 42));
    draw_rectangle(x + 5.0, cy - 2.0, CELL_SIZE - 10.0, 4.0, rgb(35, 40, 42));
    draw_circle(cx, cy, CELL_SIZE * 0.24, rgb(31, 34, 36));
    draw_circle(cx - 3.0, cy - 4.0, CELL_SIZE * 0.07, rgb(100, 106, 108));
}

fn draw_cell(layout: Layout, x: usize, y: usize, cell: Cell, pressed: bool) {
    let px = layout.board_x + x as f32 * CELL_SIZE;
    let py = layout.board_y + y as f32 * CELL_SIZE;

    match cell.state {
        CellState::Hidden => draw_hidden_tile(px, py, x, y, pressed),
        CellState::Flagged => {
            draw_hidden_tile(px, py, x, y, pressed);
            draw_flag(px, py, pressed);
        }
        CellState::Revealed => {
            draw_revealed_tile(px, py, x, y);
            if cell.is_mine {
                draw_mine(px, py);
            } else if cell.adjacent_mines > 0 {
                draw_cell_number(cell.adjacent_mines, px, py);
            }
        }
    }
}

fn is_chord_pressed_cell(game: &Game, center_x: usize, center_y: usize, x: usize, y: usize) -> bool {
    if center_x >= game.width || center_y >= game.height {
        return false;
    }
    let center = game.cell(center_x, center_y);
    center.state == CellState::Revealed
        && center.adjacent_mines > 0
        && x.abs_diff(center_x) <= 1
        && y.abs_diff(center_y) <= 1
        && !(x == center_x && y == center_y)
        && game.cell(x, y).state == CellState::Hidden
}

fn is_cell_pressed(game: &Game, held_action: Option<HeldAction>, x: usize, y: usize) -> bool {
    match held_action {
        Some(HeldAction::Reveal(held_x, held_y)) | Some(HeldAction::Flag(held_x, held_y)) => {
            held_x == x && held_y == y && game.cell(x, y).state != CellState::Revealed
        }
        Some(HeldAction::Chord(center_x, center_y)) => {
            is_chord_pressed_cell(game, center_x, center_y, x, y)
        }
        Some(HeldAction::Button(_)) | None => false,
    }
}

fn draw_game(game: &Game, held_action: Option<HeldAction>) {
    clear_background(rgb(58, 60, 52));
    let layout = layout_for(game);
    draw_hud(game, layout, held_action);
    draw_controls(game, layout, held_action);
    draw_board_frame(layout);

    for y in 0..game.height {
        for x in 0..game.width {
            draw_cell(
                layout,
                x,
                y,
                game.cell(x, y),
                is_cell_pressed(game, held_action, x, y),
            );
        }
    }
}

fn cell_at(game: &Game, layout: Layout, mx: f32, my: f32) -> Option<(usize, usize)> {
    if mx < layout.board_x
        || mx >= layout.board_x + layout.board_w
        || my < layout.board_y
        || my >= layout.board_y + layout.board_h
    {
        return None;
    }
    let x = ((mx - layout.board_x) / CELL_SIZE) as usize;
    let y = ((my - layout.board_y) / CELL_SIZE) as usize;
    if x < game.width && y < game.height {
        Some((x, y))
    } else {
        None
    }
}

fn handle_input(game: &mut Game, held_action: &mut Option<HeldAction>) {
    let layout = layout_for(game);
    let (mx, my) = mouse_position();
    let mouse = vec2(mx, my);
    let left_down = is_mouse_button_down(MouseButton::Left);
    let right_down = is_mouse_button_down(MouseButton::Right);
    let left_pressed = is_mouse_button_pressed(MouseButton::Left);
    let right_pressed = is_mouse_button_pressed(MouseButton::Right);
    let left_released = is_mouse_button_released(MouseButton::Left);
    let right_released = is_mouse_button_released(MouseButton::Right);

    if !matches!(*held_action, Some(HeldAction::Button(_))) {
        if matches!(game.state, GameState::Playing) && left_down && right_down {
            *held_action = cell_at(game, layout, mx, my)
                .filter(|(x, y)| game.cell(*x, *y).state == CellState::Revealed)
                .map(|(x, y)| HeldAction::Chord(x, y));
        } else if left_pressed {
            if let Some(action) = button_action_at(game, layout, mouse) {
                *held_action = Some(HeldAction::Button(action));
            } else if matches!(game.state, GameState::Playing) {
                if let Some((x, y)) = cell_at(game, layout, mx, my) {
                    *held_action = Some(HeldAction::Reveal(x, y));
                }
            }
        } else if right_pressed && !left_down && matches!(game.state, GameState::Playing) {
            if let Some((x, y)) = cell_at(game, layout, mx, my) {
                *held_action = Some(HeldAction::Flag(x, y));
            }
        }
    }

    match *held_action {
        Some(HeldAction::Button(action)) if left_released => {
            if button_action_at(game, layout, mouse) == Some(action) {
                apply_button_action(game, action);
            }
            *held_action = None;
        }
        Some(HeldAction::Reveal(x, y)) if left_released => {
            if matches!(game.state, GameState::Playing) && cell_at(game, layout, mx, my) == Some((x, y)) {
                game.reveal(x, y);
            }
            *held_action = None;
        }
        Some(HeldAction::Flag(x, y)) if right_released => {
            if matches!(game.state, GameState::Playing) && cell_at(game, layout, mx, my) == Some((x, y)) {
                game.toggle_flag(x, y);
            }
            *held_action = None;
        }
        Some(HeldAction::Chord(x, y)) if left_released || right_released => {
            if matches!(game.state, GameState::Playing) && cell_at(game, layout, mx, my) == Some((x, y)) {
                game.chord(x, y);
            }
            *held_action = None;
        }
        _ => {}
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Minesweeper".to_string(),
        window_width: MIN_WINDOW_WIDTH as i32,
        window_height: (PADDING
            + HUD_HEIGHT
            + CONTROLS_GAP
            + CONTROLS_HEIGHT
            + HUD_GAP
            + 9.0 * CELL_SIZE
            + PADDING) as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new(Mode::Beginner, CustomConfig::default());
    let mut held_action = None;
    sync_window_size(&game);

    loop {
        handle_input(&mut game, &mut held_action);
        draw_game(&game, held_action);
        next_frame().await
    }
}
