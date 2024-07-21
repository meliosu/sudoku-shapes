use std::io::Write;

use crossterm::{
    cursor::{Hide, MoveDown, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Color, ContentStyle, Print, PrintStyledContent, StyledContent, Stylize},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;

fn main() -> std::io::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen, Show);
        println!("thread {info}");
    }));

    let mut stdout = std::io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let mut sudoku = Sudoku::new();

    loop {
        render_sudoku(&sudoku)?;

        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => break,

            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                sudoku.place();
            }

            Event::Key(KeyEvent {
                code: KeyCode::Up | KeyCode::Char('w'),
                ..
            }) => {
                sudoku.shift(Direction::Up);
            }

            Event::Key(KeyEvent {
                code: KeyCode::Down | KeyCode::Char('s'),
                ..
            }) => {
                sudoku.shift(Direction::Down);
            }

            Event::Key(KeyEvent {
                code: KeyCode::Left | KeyCode::Char('a'),
                ..
            }) => {
                sudoku.shift(Direction::Left);
            }

            Event::Key(KeyEvent {
                code: KeyCode::Right | KeyCode::Char('d'),
                ..
            }) => {
                sudoku.shift(Direction::Right);
            }

            _ => {}
        }
    }

    terminal::disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, Show)?;

    println!("You've got {} points!", sudoku.score);

    Ok(())
}

fn render_sudoku(sudoku: &Sudoku) -> std::io::Result<()> {
    let mut stdout = std::io::stdout();

    let (cols, rows) = terminal::size()?;
    let (cell_width, cell_height) = (5, 3);
    let (width, height) = (9 * cell_width, 9 * cell_height);
    let (x, y) = ((cols - width) / 2, (rows - height) / 2);

    let top = "▛▀▀▀▜";
    let mid = "▌   ▐";
    let bot = "▙▄▄▄▟";

    for i in 0..9 {
        for j in 0..9 {
            let (x, y) = (x + i as u16 * cell_width, y + j as u16 * cell_height);

            let style = if sudoku.board[i][j] {
                ContentStyle::new().dark_blue().on_blue()
            } else {
                ContentStyle::new().dark_grey().on_grey()
            };

            queue!(
                stdout,
                MoveTo(x, y),
                PrintStyledContent(StyledContent::new(style, top)),
                MoveTo(x, y + 1),
                PrintStyledContent(StyledContent::new(style, mid)),
                MoveTo(x, y + 2),
                PrintStyledContent(StyledContent::new(style, bot)),
            )?;
        }
    }

    let block = "⯀";

    let color = if sudoku.legal() {
        Color::Green
    } else {
        Color::Red
    };

    for i in 0..3 {
        for j in 0..3 {
            if !sudoku.curr.shape[i as usize][j as usize] {
                continue;
            }

            let (sx, sy) = ((sudoku.curr.x + i) as usize, (sudoku.curr.y + j) as usize);

            let bgcolor = if sudoku.board[sx][sy] {
                Color::Blue
            } else {
                Color::Grey
            };

            let x = x + (sudoku.curr.x + i) * cell_width + (cell_width / 2);
            let y = y + (sudoku.curr.y + j) * cell_height + (cell_height / 2);

            let style = ContentStyle::new().with(color).on(bgcolor);

            queue!(
                stdout,
                MoveTo(x, y),
                PrintStyledContent(StyledContent::new(style, block))
            )?;
        }
    }

    stdout.flush()?;

    Ok(())
}

#[derive(Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Sudoku {
    board: [[bool; 9]; 9],
    score: u64,
    curr: Piece,
}

struct Piece {
    x: u16,
    y: u16,
    shape: [[bool; 3]; 3],
}

impl Piece {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            shape: [[false; 3]; 3],
        }
    }

    fn random() -> Self {
        Self {
            x: 0,
            y: 0,
            shape: std::array::from_fn(|_| {
                std::array::from_fn(|_| rand::thread_rng().gen_ratio(1, 3))
            }),
        }
    }

    fn bounds(&self) -> (u16, u16, u16, u16) {
        let (mut xmin, mut ymin) = (2, 2);
        let (mut xmax, mut ymax) = (0, 0);

        for x in 0..3 {
            for y in 0..3 {
                if self.shape[x as usize][y as usize] {
                    xmin = xmin.min(x);
                    xmax = xmax.max(x);
                    ymin = ymin.min(y);
                    ymax = ymax.max(y);
                }
            }
        }

        (xmin, ymin, xmax, ymax)
    }
}

impl Sudoku {
    fn new() -> Self {
        Self {
            score: 0,
            board: [[false; 9]; 9],
            curr: Piece::random(),
        }
    }

    fn random() -> Self {
        Self {
            score: 0,
            board: std::array::from_fn(|_| std::array::from_fn(|_| rand::random())),
            curr: Piece::random(),
        }
    }

    fn shift(&mut self, dir: Direction) {
        //let (xmin, ymin, xmax, ymax) = self.curr.bounds();

        match dir {
            Direction::Up => {
                if self.curr.y > 0 {
                    self.curr.y -= 1;
                }
            }

            Direction::Down => {
                if self.curr.y < 9 - 3 {
                    self.curr.y += 1;
                }
            }

            Direction::Left => {
                if self.curr.x > 0 {
                    self.curr.x -= 1;
                }
            }

            Direction::Right => {
                if self.curr.x < 9 - 3 {
                    self.curr.x += 1;
                }
            }
        }
    }

    fn legal(&self) -> bool {
        for i in 0..3 {
            for j in 0..3 {
                if !self.curr.shape[i][j] {
                    continue;
                }

                let (x, y) = (self.curr.x as usize + i, self.curr.y as usize + j);

                if self.board[x][y] {
                    return false;
                }
            }
        }

        true
    }

    fn place(&mut self) {
        if self.legal() {
            for i in 0..3 {
                for j in 0..3 {
                    if self.curr.shape[i][j] {
                        self.board[self.curr.x as usize + i][self.curr.y as usize + j] = true;
                    }
                }
            }

            // Check rows
            for i in 0..9 {
                if self.board[i].iter().all(|&filled| filled) {
                    self.board[i] = [false; 9];
                    self.score += 9;
                }
            }

            // Check columns
            for i in 0..9 {
                if self.board.iter().all(|row| row[i]) {
                    for j in 0..9 {
                        self.board[j][i] = false;
                    }

                    self.score += 9;
                }
            }

            // Check blocks
            for i in 0..3 {
                for j in 0..3 {
                    if self.board[i * 3..(i + 1) * 3]
                        .iter()
                        .all(|row| row[j * 3..(j + 1) * 3].iter().all(|&filled| filled))
                    {
                        for k in 0..3 {
                            for l in 0..3 {
                                self.board[i * 3 + k][j * 3 + l] = false;
                            }
                        }

                        self.score += 9;
                    }
                }
            }

            // Replace current piece
            self.curr = Piece::random();
        }
    }
}
