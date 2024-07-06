use std::{
    fmt::{Display, Write},
    io::stdin,
    num::ParseIntError,
    time::SystemTime,
};

use rand;
use rand::seq::SliceRandom;

fn main() {
    println!("{}", welcome_msg());

    game_loop();
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

    println!("Enter game config - example: 10 10\nThis means board 10x10 with 10 mines.");
    let mut config = String::new();
    stdin()
        .read_line(&mut config)
        .expect("Did not enter string?");

    let mut game_board = GameBoard::new(GameConfiguration::try_from(&config[..])
        .expect("Try again, config should look like the following: 10 10\nFirst one is dimension, second number of mines."));

    game_board.generate_world();

    let now = SystemTime::now();

    loop {
        println!("{}", &game_board);
        let mut cmd = String::new();
        stdin()
            .read_line(&mut cmd)
            .expect("Did not enter a string?!");
        clear_console();

        if let Ok(cmd) = BoardCommand::try_from(&cmd[..]) {
            let resolve = game_board.manipulate_cell(cmd);
            match resolve {
                GameResolve::Quit => break,
                GameResolve::Continue => continue,
                GameResolve::MineHit => {
                    println!("HIT MINE!");
                    break;
                }
                GameResolve::AllMinesDiscovered => {
                    println!("YOU WON!");
                    break;
                }
            }
        }
    }
    if let Ok(elapsed) = now.elapsed() {
        println!("Game took {} s.", elapsed.as_secs())
    }
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
            width: 5,
            height: 5,
            total_mines: 10,
        }
    }
}

#[derive(Debug)]
enum GameConfigurationError {
    MalformedString,
    MalformedInteger(ParseIntError),
}

impl TryFrom<&str> for GameConfiguration {
    type Error = GameConfigurationError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let game_config = value
            .split_once(" ")
            .ok_or(GameConfigurationError::MalformedString);
        let (dimensions, mines) = game_config.unwrap();

        Ok(GameConfiguration {
            width: dimensions
                .trim()
                .parse::<u16>()
                .or_else(|err| Err(GameConfigurationError::MalformedInteger(err)))?,
            height: dimensions
                .trim()
                .parse::<u16>()
                .or_else(|err| Err(GameConfigurationError::MalformedInteger(err)))?,
            total_mines: mines
                .trim()
                .parse::<u32>()
                .or_else(|err| Err(GameConfigurationError::MalformedInteger(err)))?,
        })
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
    fn new(game_configuration: GameConfiguration) -> GameBoard {
        GameBoard {
            game_configuration,
            mines_discovered: 0,
            cells: vec![
                BoardCell::NoMine(CellInfo(Mark::NoMark, NeighbourMines(0)));
                game_configuration.w() as usize * game_configuration.h() as usize
            ],
        }
    }

    fn generate_world(&mut self) {
        let mut mine_positions: Vec<u32> = (0..(self.game_configuration.h() as u32
            * self.game_configuration.w() as u32))
            .collect();
        mine_positions.shuffle(&mut rand::thread_rng());

        for mine_lin_index in &mine_positions[0..self.game_configuration.total_mines as usize] {
            self.cells[*mine_lin_index as usize] = BoardCell::Mine(Mark::NoMark);
        }

        // this one goes through all fields, a bit unnecessary
        // for row in 0..self.game_configuration.h() {
        //     for col in 0..self.game_configuration.w() {}
        // }

        // quiet inefficient, but I am lazy atm
        for mine_lin_index in &mine_positions[0..self.game_configuration.total_mines as usize] {
            let mut neighbours: Vec<Coordinate> = vec![];
            self.add_neighbours(
                &mut neighbours,
                Coordinate(
                    (mine_lin_index / self.game_configuration.w() as u32) as u16,
                    (mine_lin_index % self.game_configuration.w() as u32) as u16,
                ),
            );

            for neighbour in neighbours {
                let lin_index = self.compute_linear_index(neighbour);
                self.cells[lin_index] = match &self.cells[lin_index] {
                    &BoardCell::NoMine(cell_info) => BoardCell::NoMine(CellInfo(
                        Mark::NoMark,
                        NeighbourMines(cell_info.1 .0 + 1),
                    )),
                    &anything => anything,
                }
            }
        }
    }

    fn manipulate_cell(&mut self, command: BoardCommand) -> GameResolve {
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
        for i in -1..=1 {
            for j in -1..=1 {
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

    fn get_dimensions(&self) -> (u16, u16) {
        (self.game_configuration.w(), self.game_configuration.h())
    }

    fn get_cell_at(&self, coordinate: Coordinate) -> &BoardCell {
        &self.cells[self.compute_linear_index(coordinate)]
    }
}

impl Display for GameBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (width, height) = self.get_dimensions();

        write!(f, "{:>3}", "");
        for col in 0..width {
            write!(f, "{:>3}", col);
        }
        write!(f, "\n");

        for row in 0..height {
            write!(f, "{:>3}|", row);

            for col in 0..width {
                let symbol = match self.get_cell_at(Coordinate(row, col)) {
                    BoardCell::NoMine(cell_info) => match cell_info.0 {
                        Mark::NoMark => "|X|".to_string(),
                        Mark::MarkNote => "|N|".to_string(),
                        Mark::MarkFlag => "|F|".to_string(),
                    },
                    BoardCell::Mine(mark_info) => match mark_info {
                        Mark::NoMark => "|X|".to_string(),
                        Mark::MarkNote => "|N|".to_string(),
                        Mark::MarkFlag => "|F|".to_string(),
                    },
                    BoardCell::Explored(neighbour_info) => {
                        if neighbour_info.0 == 0 {
                            "| |".to_string()
                        } else {
                            format!("|{}|", neighbour_info.0.to_string())
                        }
                    }
                };

                write!(f, "{:>3}", symbol)
                    .expect("Writing a new symbol failed in game board display.");
            }
            write!(f, "\n").expect("Writing new line failed in game board display.");
        }

        Ok(())
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
