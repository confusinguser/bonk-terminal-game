use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use rand::Rng;
use terminal_pixel_renderer::TerminalDisplay;

const PLAYER_DIMS: (usize, usize) = (3, 2);

fn main() {
    let mut game = Game {
        play_area: (60, 80),
        enemies: Vec::new(),
        player: Entity::default(),
        coin_pos: (0, 0),
        score: 0,
    };
    let mut terminal_display = TerminalDisplay::default();

    game.player.position = game.get_random_position_on_board();
    game.coin_pos = game.get_random_position_on_board();

    let arc = Arc::<Mutex<usize>>::new(Mutex::new(0));
    let cloned_arc = Arc::clone(&arc);
    thread::spawn(move || {
        loop {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).expect("Failed to read line");
            *cloned_arc.lock().unwrap() += 1;
        }
    });

    let mut latest_enter = 0;
    loop {
        {
            let current_enter = *arc.lock().unwrap();
            if latest_enter != current_enter { // Means other side did trigger while we were sleeping
                game.player.going_right = !game.player.going_right;
                TerminalDisplay::move_cursor_up((current_enter - latest_enter) as u16);
                latest_enter = current_enter;
            }
        }
        game.update_board();
        let board = game.render_board();
        terminal_display.update_display(TerminalDisplay::render(&board));
        thread::sleep(Duration::from_millis(100))
    }
}

struct Game {
    enemies: Vec<Entity>,
    player: Entity,
    play_area: (usize, usize),
    coin_pos: (usize, usize),
    score: u32,
}

struct Entity {
    position: (usize, usize),
    going_right: bool,
    going_down: bool,
}

impl Entity {
    pub(crate) fn update_entity(&mut self, play_area: (usize, usize)) {
        let mut new_position = self.calc_new_position();
        (self.going_right, self.going_down) = self.calc_new_headings(new_position, play_area);
        new_position = self.calc_new_position();
        self.position = (new_position.0.0, new_position.1.0);
    }

    fn calc_new_position(&self) -> ((usize, bool), (usize, bool)) {
        let mut x = self.position.0.overflowing_add_signed(self.going_right as isize * 2 - 1);
        if x.1 { x.0 = 0; }

        let mut y = self.position.1.overflowing_add_signed(self.going_down as isize * 2 - 1);
        if y.1 { y.0 = 0; }

        (x, y)
    }

    /// Arguments:
    /// * new_position: ((usize, bool), (usize, bool)) - The booleans indicate whether the coordinates have overflowed.
    /// returns: (bool, bool). First one is the calculated value for `going_right`, and the second is for `going_down`
    fn calc_new_headings(&self, new_position: ((usize, bool), (usize, bool)), play_area: (usize, usize)) -> (bool, bool) {
        let mut going_right = self.going_right;
        let mut going_down = self.going_down;
        if new_position.0.1 { going_right = true }
        if new_position.1.1 { going_down = true }
        if new_position.0.0 >= play_area.0 { going_right = false }
        if new_position.1.0 >= play_area.1 { going_down = false }
        (going_right, going_down)
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            position: (0, 0),
            going_right: false,
            going_down: false,
        }
    }
}

impl Game {
    fn get_random_position_on_board(&self) -> (usize, usize) {
        let mut rng = rand::thread_rng();
        (rng.gen_range(0..self.play_area.0), rng.gen_range(0..self.play_area.1))
    }

    fn render_board(&self) -> Vec<Vec<bool>> {
        let mut pixels = vec![vec![false; self.play_area.0 + 2]; self.play_area.1 + 2];
        for enemy in &self.enemies {
            pixels[enemy.position.1][enemy.position.0] = true;
        }

        for (y, row) in pixels.iter_mut().enumerate() {
            for (x, pixel) in row.iter_mut().enumerate() {
                if (x == 0 || x == self.play_area.0 + 1) && y % 2 == 0 ||
                    (y == 0 || y == self.play_area.1 + 1) && x % 2 == 0 {
                    *pixel = true;
                    continue;
                } else if x == 0 || y == 0 { continue; }
                let x = x - 1;
                let y = y - 1;
                if self.player.position.0.abs_diff(x) < PLAYER_DIMS.0 &&
                    self.player.position.1.abs_diff(y) < PLAYER_DIMS.1 {
                    *pixel = true;
                } else if self.coin_pos.0.abs_diff(x) < 2 &&
                    self.coin_pos.1.abs_diff(y) < 2 {
                    *pixel = true;
                } else {
                    for enemy in &self.enemies {
                        if enemy.position.0.abs_diff(x) < 3 && enemy.position.1.abs_diff(y) < 2 {
                            *pixel = true;
                        }
                    }
                }
            }
        }

        pixels[self.player.position.1][self.player.position.0] = true;
        pixels
    }

    fn update_board(&mut self) {
        for enemy in &mut self.enemies {
            enemy.update_entity(self.play_area);
        }
        self.player.update_entity(self.play_area);
        if self.player.position.0.abs_diff(self.coin_pos.0) <= PLAYER_DIMS.0 + 2 &&
            self.player.position.1.abs_diff(self.coin_pos.1) <= PLAYER_DIMS.1 + 2 {
            self.score += 1;
            self.coin_pos = self.get_random_position_on_board();
        }
    }

    pub(crate) fn is_inside_board(&self, position: ((usize, bool), (usize, bool))) -> bool {
        !position.0.1 && !position.1.1 && position.0.0 < self.play_area.0 && position.1.0 < self.play_area.1
    }
}

fn distance_sqr(lhs: (usize, usize), rhs: (usize, usize)) -> usize {
    ((lhs.0 as isize - rhs.0 as isize).pow(2) + (lhs.1 as isize - rhs.1 as isize).pow(2)) as usize
}