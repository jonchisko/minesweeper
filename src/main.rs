use std::{fmt::Write, num::ParseIntError};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Coordinate(u16, u16);

#[derive(Debug, Clone, PartialEq, Eq)]
enum BoardCommandError {
    MalformedString,
    MalformedCoordinate,
    CoordinateParsing(ParseIntError),
    NotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoardCommand {
    Pass,
    Quit,
    ClearMark(Coordinate),
    SetMarkFlag(Coordinate),
    SetMarkNote(Coordinate),
    Explore(Coordinate),
}

impl TryFrom<&str> for BoardCommand {
    type Error = BoardCommandError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_lowercase().trim().to_string();

        if value == "pass" {
            return Ok(BoardCommand::Pass);
        }

        if value == "quit" {
            return Ok(BoardCommand::Quit);
        }

        let command_coordinate = value
            .split_once('(')
            .ok_or(BoardCommandError::MalformedString)?;

        let value = command_coordinate.1;

        let value = value.trim();
        let (value_x, value_y) = value
            .split_once(',')
            .ok_or(BoardCommandError::MalformedCoordinate)?;

        let value_x = value_x
            .parse::<u16>()
            .map_err(|err| BoardCommandError::CoordinateParsing(err))?;

        let value_y = value_y.replace(&['\n', ')'], "").trim().to_string();
        let value_y = value_y
            .parse::<u16>()
            .map_err(|err| BoardCommandError::CoordinateParsing(err))?;

        let command = command_coordinate.0.trim();

        match command {
            "clear" => Ok(BoardCommand::ClearMark(Coordinate(value_x, value_y))),
            "flag" => Ok(BoardCommand::SetMarkFlag(Coordinate(value_x, value_y))),
            "note" => Ok(BoardCommand::SetMarkNote(Coordinate(value_x, value_y))),
            "explore" => Ok(BoardCommand::Explore(Coordinate(value_x, value_y))),
            _ => Err(BoardCommandError::NotFound),
        }
    }
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
    Quit,
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
            BoardCommand::Quit => GameResolve::Quit,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_command_test() {
        let command = "pass";
        assert_eq!(BoardCommand::Pass, command.try_into().unwrap());

        let command = "quit";
        assert_eq!(BoardCommand::Quit, command.try_into().unwrap());

        let command = "clear(0, 0)";
        assert_eq!(
            BoardCommand::ClearMark(Coordinate(0, 0)),
            command.try_into().unwrap()
        );

        let command = "note(2,1)";
        assert_eq!(
            BoardCommand::SetMarkNote(Coordinate(2, 1)),
            command.try_into().unwrap()
        );

        let command = "flag(100, 21)";
        assert_eq!(
            BoardCommand::SetMarkFlag(Coordinate(100, 21)),
            command.try_into().unwrap()
        );

        let command = "explore(20, 20)";
        assert_eq!(
            BoardCommand::Explore(Coordinate(20, 20)),
            command.try_into().unwrap()
        );
    }

    #[test]
    fn fail_to_create_command_test() {
        let command = "asd";
        let result: Result<BoardCommand, BoardCommandError> = command.try_into();
        assert_eq!(Err(BoardCommandError::MalformedString), result);

        let command = "mark(10,10,10)";
        let result: Result<BoardCommand, BoardCommandError> = command.try_into();
        // not the best way to handle these errors in such a way. One thing is that the msg is lost
        if let Err(BoardCommandError::CoordinateParsing(_)) = result {
            assert!(true);
        } else {
            assert!(false);
        }

        let command = "mark(10.10)";
        let result: Result<BoardCommand, BoardCommandError> = command.try_into();
        assert_eq!(Err(BoardCommandError::MalformedCoordinate), result);

        let command = "flag(1000_000, 10)";
        let result: Result<BoardCommand, BoardCommandError> = command.try_into();
        if let Err(BoardCommandError::CoordinateParsing(_)) = result {
            assert!(true);
        } else {
            assert!(false);
        }

        let command = "flag((1000, 20))";
        let result: Result<BoardCommand, BoardCommandError> = command.try_into();
        if let Err(BoardCommandError::CoordinateParsing(_)) = result {
            assert!(true);
        } else {
            assert!(false);
        }

        let command = "test(10, 10)";
        let result: Result<BoardCommand, BoardCommandError> = command.try_into();
        assert_eq!(Err(BoardCommandError::NotFound), result);
    }
}
