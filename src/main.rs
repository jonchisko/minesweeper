use std::fmt::Write;

fn main() {
    println!("{}", welcome_msg());
}

fn welcome_msg() -> &'static str {
    "Welcome to minesweeper\nKeymaps:\nplay-1,\nhighscores-2,\nquit-3"
}

fn clear_console() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char)
}

fn game_loop() {
    // init game

    // game loop
    // - draw board state
    // - wait for input
    // - execute command
    // - show command result and start a wait thread that is polled
    // - continue after 3 secs
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Coordinate(u16, u16);

enum BoardCommand {
    Pass,
    ClearMark(Coordinate),
    SetMarkFlag(Coordinate),
    SetMarkNote(Coordinate),
    Explore(Coordinate),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NeighbourMines(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    NoMark,
    MarkNote,
    MarkFlag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CellInfo(Mark, NeighbourMines);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoardCell {
    Explored(NeighbourMines),
    NoMine(CellInfo),
    Mine(Mark),
}

#[derive(Clone, Copy)]
struct GameConfiguration {
    width: u16,
    height: u16,
    total_mines: u32,
}

impl GameConfiguration {
    pub fn new(width: u16, height: u16, total_mines: u32) -> Self {
        GameConfiguration {
            width,
            height,
            total_mines,
        }
    }

    pub fn w(&self) -> u16 {
        self.width
    }

    pub fn h(&self) -> u16 {
        self.height
    }

    pub fn mines(&self) -> u32 {
        self.total_mines
    }
}

impl Default for GameConfiguration {
    fn default() -> Self {
        GameConfiguration {
            width: 32,
            height: 32,
            total_mines: 90,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GameResolve {
    Continue,
    MineHit,
    AllMinesDiscovered,
}

struct GameBoard {
    game_configuration: GameConfiguration,
    mines_discovered: u32,
    cells: Vec<BoardCell>,
}

impl GameBoard {
    pub fn new(game_configuration: GameConfiguration) -> GameBoard {
        GameBoard {
            game_configuration,
            mines_discovered: 0,
            cells: vec![
                BoardCell::NoMine(CellInfo(Mark::NoMark, NeighbourMines(0)));
                game_configuration.w() as usize * game_configuration.h() as usize
            ],
        }
    }

    pub fn manipulate_cell(&mut self, command: BoardCommand) -> GameResolve {
        let command_result = match command {
            BoardCommand::Pass => GameResolve::Continue,
            BoardCommand::ClearMark(coordinate) => self.clear_mark(coordinate),
            BoardCommand::SetMarkFlag(coordinate) => self.set_mark_flag(coordinate),
            BoardCommand::SetMarkNote(coordinate) => self.set_mark_note(coordinate),
            BoardCommand::Explore(coordinate) => self.explore(coordinate),
        };

        match command_result {
            GameResolve::Continue | GameResolve::AllMinesDiscovered => {
                if self.mines_discovered == self.game_configuration.mines() {
                    GameResolve::AllMinesDiscovered
                } else {
                    GameResolve::Continue
                }
            }
            other => other,
        }
    }

    fn clear_mark(&mut self, coordinate: Coordinate) -> GameResolve {
        let linear_index = self.compute_linear_index(coordinate);

        match &self.cells[linear_index] {
            &BoardCell::NoMine(ref cell_info) => {
                self.cells[linear_index] = BoardCell::NoMine(CellInfo(Mark::NoMark, cell_info.1));
            }
            &BoardCell::Mine(ref mark) => {
                if let Mark::MarkFlag = mark {
                    self.mines_discovered -= 1;
                }
                self.cells[linear_index] = BoardCell::Mine(Mark::NoMark);
            }
            _ => {}
        }

        GameResolve::Continue
    }

    fn set_mark_flag(&mut self, coordinate: Coordinate) -> GameResolve {
        let linear_index = self.compute_linear_index(coordinate);

        match &self.cells[linear_index] {
            &BoardCell::NoMine(ref cell_info) => {
                self.cells[linear_index] = BoardCell::NoMine(CellInfo(Mark::MarkFlag, cell_info.1))
            }
            &BoardCell::Mine(ref mark) => {
                match &mark {
                    Mark::NoMark | Mark::MarkNote => self.mines_discovered += 1,
                    _ => {}
                }
                self.cells[linear_index] = BoardCell::Mine(Mark::MarkFlag);
            }
            _ => {}
        }

        GameResolve::Continue
    }

    fn set_mark_note(&mut self, coordinate: Coordinate) -> GameResolve {
        let linear_index = self.compute_linear_index(coordinate);

        match &self.cells[linear_index] {
            &BoardCell::NoMine(ref cell_info) => {
                self.cells[linear_index] = BoardCell::NoMine(CellInfo(Mark::MarkNote, cell_info.1))
            }
            &BoardCell::Mine(ref mark) => {
                if let Mark::MarkFlag = mark {
                    self.mines_discovered -= 1;
                }
                self.cells[linear_index] = BoardCell::Mine(Mark::MarkNote);
            }
            _ => {}
        }

        GameResolve::Continue
    }

    fn explore(&mut self, coordinate: Coordinate) -> GameResolve {
        let linear_index = self.compute_linear_index(coordinate);

        match &self.cells[linear_index] {
            &BoardCell::NoMine(_) => {
                self.explore_cells(coordinate);
                GameResolve::Continue
            }
            &BoardCell::Mine(_) => GameResolve::MineHit,
            _ => GameResolve::Continue,
        }
    }

    fn compute_linear_index(&self, coordinate: Coordinate) -> usize {
        (coordinate.0 * self.game_configuration.w() + coordinate.1) as usize
    }

    fn explore_cells(&mut self, coordinate: Coordinate) {
        let mut queue: Vec<Coordinate> = vec![coordinate];

        while let Some(cell_coordinate) = queue.pop() {
            let linear_index = self.compute_linear_index(cell_coordinate);
            match &self.cells[linear_index] {
                &BoardCell::Explored(_) => continue,
                &BoardCell::NoMine(ref cell_info) => {
                    self.add_neighbours(&mut queue, cell_coordinate);
                    self.cells[linear_index] = BoardCell::Explored(cell_info.1);
                }
                _ => {}
            }
        }
    }

    fn add_neighbours(&self, queue: &mut Vec<Coordinate>, center: Coordinate) {
        for i in -1..1 {
            for j in -1..1 {
                let x = center.0 as i32 + i;
                let y = center.1 as i32 + j;

                if x == center.0 as i32 && y == center.1 as i32
                    || x < 0
                    || y < 0
                    || x >= self.game_configuration.h() as i32
                    || y >= self.game_configuration.w() as i32
                {
                    continue;
                }

                queue.push(Coordinate(x as u16, y as u16))
            }
        }
    }
}
