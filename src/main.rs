use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    event::{Event, KeyCode, poll, read},
    style::{self, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use rand::Rng;
use rand::rngs::ThreadRng;
use std::io::{self, Stdout, Write};
use std::time;

struct Game {
    height: u16,
    width: u16,
    wall: Vec<[i16; 2]>,
    score: u16,
    polling_rate: time::Duration,
}

impl Game {
    fn new(
        height: u16,
        width: u16,
        wall: Vec<[i16; 2]>,
        score: u16,
        polling_rate: time::Duration,
    ) -> Game {
        Game {
            height,
            width,
            wall,
            score,
            polling_rate,
        }
    }

    fn draw_border(&mut self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;

        for y in 0..self.height {
            for x in 0..self.width {
                if (y == 0 || y == self.height - 1) || (x == 0 || x == self.width - 1) {
                    stdout
                        .queue(cursor::MoveTo(x, y))?
                        .queue(style::PrintStyledContent("█".magenta()))?;
                    self.wall.push([x as i16, y as i16]);
                }
            }
        }
        stdout.flush()?;

        Ok(())
    }

    fn handle_input(&self, snake: &mut Snake) -> Result<(), std::io::Error> {
        if poll(self.polling_rate)? {
            let event = read()?;
            if event == Event::Key(KeyCode::Char('w').into()) && snake.direction[1] != 1 {
                snake.direction[0] = 0;
                snake.direction[1] = -1;
                Ok(())
            } else if event == Event::Key(KeyCode::Char('d').into()) && snake.direction[0] != -1 {
                snake.direction[0] = 1;
                snake.direction[1] = 0;
                Ok(())
            } else if event == Event::Key(KeyCode::Char('s').into()) && snake.direction[1] != -1 {
                snake.direction[0] = 0;
                snake.direction[1] = 1;
                Ok(())
            } else if event == Event::Key(KeyCode::Char('a').into()) && snake.direction[0] != 1 {
                snake.direction[0] = -1;
                snake.direction[1] = 0;
                Ok(())
            } else if event == Event::Key(KeyCode::Esc.into()) {
                snake.direction[0] = 69;
                snake.direction[1] = 69;
                Ok(())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn increase_score(&mut self) {
        self.score += 1;
    }

    fn display_score(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        stdout.queue(cursor::MoveTo(0, self.height))?;
        println!("Score: {}", self.score);
        let msg = "WASD to move, ESC to exit";
        stdout.queue(cursor::MoveTo(self.width - msg.len() as u16, self.height))?;
        println!("{}", msg);
        Ok(())
    }
}

impl Default for Game {
    fn default() -> Game {
        let height = 15;
        let width = 40;
        let wall: Vec<[i16; 2]> = vec![];
        let score = 0;
        let polling_rate = time::Duration::from_millis(100);
        Game::new(height, width, wall, score, polling_rate)
    }
}

struct Snake {
    body: Vec<[i16; 2]>,
    head: [i16; 2],
    tail: [i16; 2],
    wake: [i16; 2],
    length: usize,
    direction: [i16; 2],
}

impl Snake {
    fn new(
        head: [i16; 2],
        body: Vec<[i16; 2]>,
        tail: [i16; 2],
        wake: [i16; 2],
        length: usize,
        direction: [i16; 2],
    ) -> Snake {
        Snake {
            head,
            body,
            tail,
            wake,
            length,
            direction,
        }
    }

    fn spawn(game: &Game) -> Snake {
        let head: [i16; 2] = [game.width as i16 / 3, game.height as i16 / 2];
        let tail: [i16; 2] = [head[0] - 2, head[1]];
        let wake: [i16; 2] = [head[0] - 3, head[1]];
        let body: Vec<[i16; 2]> = vec![head, [head[0] - 1, head[1]], tail];
        let length: usize = 3;
        let direction: [i16; 2] = [1, 0];
        Snake::new(head, body, tail, wake, length, direction)
    }

    fn ate(&self, apple: &mut Apple) -> bool {
        self.head[0] == apple.position[0] && self.head[1] == apple.position[1]
    }

    fn grow(&mut self) {
        self.body
            .insert(self.length - 1, [self.tail[0], self.tail[1]]);
        self.length += 1;
    }

    fn draw(&mut self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        stdout
            .queue(cursor::MoveTo(
                self.head[0].try_into().unwrap(),
                self.head[1].try_into().unwrap(),
            ))?
            .queue(style::PrintStyledContent("$".green()))?;

        for i in 1..self.length - 1 {
            let color;
            if i % 2 == 0 {
                color = "$".green();
            } else {
                color = "$".cyan();
            }
            stdout
                .queue(cursor::MoveTo(
                    self.body[i][0].try_into().unwrap(),
                    self.body[i][1].try_into().unwrap(),
                ))?
                .queue(style::PrintStyledContent(color))?;
        }
        let color;
        if self.length % 2 == 0 {
            color = "$".cyan();
        } else {
            color = "$".green();
        }
        stdout
            .queue(cursor::MoveTo(
                self.tail[0].try_into().unwrap(),
                self.tail[1].try_into().unwrap(),
            ))?
            .queue(style::PrintStyledContent(color))?;
        stdout
            .queue(cursor::MoveTo(
                self.wake[0].try_into().unwrap(),
                self.wake[1].try_into().unwrap(),
            ))?
            .queue(style::Print(" "))?;

        Ok(())
    }

    fn slither(&mut self) -> Result<(), std::io::Error> {
        self.wake = [self.tail[0], self.tail[1]];
        self.tail = [self.body[self.length - 2][0], self.body[self.length - 2][1]];
        let mut i = self.length - 2;
        loop {
            self.body[i] = self.body[i - 1];
            i -= 1;
            if i < 1 {
                break;
            }
        }
        self.body[1] = [self.head[0], self.head[1]];
        self.head[0] += self.direction[0] as i16;
        self.head[1] += self.direction[1] as i16;
        self.body[self.length - 1] = self.tail;
        self.body[0] = self.head;
        Ok(())
    }

    fn collided_with_self(&self) -> bool {
        self.body[1..self.length].contains(&self.head)
    }

    fn collided_with_wall(&self, game: &Game) -> bool {
        if self.head[0] == game.width as i16 - 1 {
            return true;
        } else if self.head[0] == 0 {
            return true;
        } else if self.head[1] == game.height as i16 - 1 {
            return true;
        } else if self.head[1] == 0 {
            return true;
        }
        false
    }
}

struct Apple {
    position: [i16; 2],
    exists: bool,
}

impl Apple {
    fn new(position: [i16; 2], exists: bool) -> Apple {
        Apple { position, exists }
    }

    fn spawn(
        &mut self,
        snake: &Snake,
        game: &Game,
        rng: &mut ThreadRng,
        stdout: &mut Stdout,
    ) -> Result<(), std::io::Error> {
        self.position = [
            rng.random_range(0..game.width as i16),
            rng.random_range(0..game.height as i16),
        ];

        if !game.wall.contains(&self.position) && !snake.body.contains(&self.position) {
            stdout
                .queue(cursor::MoveTo(
                    self.position[0].try_into().unwrap(),
                    self.position[1].try_into().unwrap(),
                ))?
                .queue(style::PrintStyledContent("@".red()))?;
            self.exists = true;
        }
        Ok(())
    }
}

impl Default for Apple {
    fn default() -> Apple {
        Apple::new([0, 0], false)
    }
}

const EXIT_SIGNAL: [i16; 2] = [69, 69];

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let mut rng = rand::rng();
    stdout.execute(cursor::Hide)?;

    let mut game = Game::default();
    game.draw_border(&mut stdout)?;
    let mut snake = Snake::spawn(&game);
    let mut apple = Apple::default();

    // MAIN GAME LOOP
    loop {
        game.handle_input(&mut snake)?;
        if snake.direction == EXIT_SIGNAL {
            // escape pressed
            break;
        }

        while !apple.exists {
            apple.spawn(&snake, &game, &mut rng, &mut stdout)?;
        }

        if snake.ate(&mut apple) {
            apple.exists = false;
            snake.grow();
            game.increase_score();
        }

        snake.slither()?;

        if snake.collided_with_self() {
            break;
        }

        if snake.collided_with_wall(&game) {
            break;
        }

        snake.draw(&mut stdout)?;

        game.display_score(&mut stdout)?;

        // can't forget to flush after myself
        stdout.flush()?;
    }

    // and clean up
    disable_raw_mode()?;
    stdout.queue(cursor::MoveTo(0, game.height + 1))?;
    stdout.execute(cursor::Show)?;

    Ok(())
}
