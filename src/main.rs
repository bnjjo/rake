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

fn draw_border(
    height: u16,
    width: u16,
    stdout: &mut Stdout,
    wall: &mut Vec<[i16; 2]>,
) -> Result<(), std::io::Error> {
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    for y in 0..height {
        for x in 0..width {
            if (y == 0 || y == height - 1) || (x == 0 || x == width - 1) {
                stdout
                    .queue(cursor::MoveTo(x, y))?
                    .queue(style::PrintStyledContent("█".magenta()))?;
                wall.push([x as i16, y as i16]);
            }
        }
    }
    stdout.flush()?;

    Ok(())
}

fn handle_input(
    x: &mut i16,
    y: &mut i16,
    duration: time::Duration,
) -> Result<[i16; 2], std::io::Error> {
    if poll(duration)? {
        let event = read()?;
        if event == Event::Key(KeyCode::Char('w').into()) && *y != 1 {
            *x = 0;
            *y = -1;
            return Ok([*x, *y]);
        } else if event == Event::Key(KeyCode::Char('d').into()) && *x != -1 {
            *x = 1;
            *y = 0;
            return Ok([*x, *y]);
        } else if event == Event::Key(KeyCode::Char('s').into()) && *y != -1 {
            *x = 0;
            *y = 1;
            return Ok([*x, *y]);
        } else if event == Event::Key(KeyCode::Char('a').into()) && *x != 1 {
            *x = -1;
            *y = 0;
            return Ok([*x, *y]);
        } else if event == Event::Key(KeyCode::Esc.into()) {
            Ok([69, 69])
        } else {
            Ok([*x, *y])
        }
    } else {
        Ok([*x, *y])
    }
}

fn spawn_apple(
    apple_exists: &mut bool,
    apple: &mut [i16; 2],
    snake: &Vec<[i16; 2]>,
    wall: &Vec<[i16; 2]>,
    width: u16,
    height: u16,
    rng: &mut ThreadRng,
    stdout: &mut Stdout,
) -> Result<(), std::io::Error> {
    *apple = [
        rng.random_range(0..width as i16),
        rng.random_range(0..height as i16),
    ];

    if !wall.contains(&apple) && !snake.contains(&apple) {
        stdout
            .queue(cursor::MoveTo(
                apple[0].try_into().unwrap(),
                apple[1].try_into().unwrap(),
            ))?
            .queue(style::PrintStyledContent("@".red()))?;
        *apple_exists = true;
    }
    Ok(())
}

fn draw_snake(
    snake: &Vec<[i16; 2]>,
    snake_length: usize,
    head: [i16; 2],
    tail: [i16; 2],
    wake: [i16; 2],
    stdout: &mut Stdout,
) -> Result<(), std::io::Error> {
    stdout
        .queue(cursor::MoveTo(
            head[0].try_into().unwrap(),
            head[1].try_into().unwrap(),
        ))?
        .queue(style::PrintStyledContent("$".green()))?;

    for body in 1..snake_length - 1 {
        let color;
        if body % 2 == 0 {
            color = "$".green();
        } else {
            color = "$".cyan();
        }
        stdout
            .queue(cursor::MoveTo(
                snake[body][0].try_into().unwrap(),
                snake[body][1].try_into().unwrap(),
            ))?
            .queue(style::PrintStyledContent(color))?;
    }
    let color;
    if snake_length % 2 == 0 {
        color = "$".cyan();
    } else {
        color = "$".green();
    }
    stdout
        .queue(cursor::MoveTo(
            tail[0].try_into().unwrap(),
            tail[1].try_into().unwrap(),
        ))?
        .queue(style::PrintStyledContent(color))?;
    stdout
        .queue(cursor::MoveTo(
            wake[0].try_into().unwrap(),
            wake[1].try_into().unwrap(),
        ))?
        .queue(style::Print(" "))?;

    Ok(())
}

fn move_snake(
    snake: &mut Vec<[i16; 2]>,
    snake_length: usize,
    head: &mut [i16; 2],
    tail: &mut [i16; 2],
    wake: &mut [i16; 2],
    direction: [i16; 2],
) -> Result<(), std::io::Error> {
    *wake = [tail[0], tail[1]];
    *tail = [snake[snake_length - 2][0], snake[snake_length - 2][1]];
    let mut i = snake_length - 2;
    loop {
        snake[i] = snake[i - 1];
        i -= 1;
        if i < 1 {
            break;
        }
    }
    snake[1] = [head[0], head[1]];
    head[0] += direction[0] as i16;
    head[1] += direction[1] as i16;
    snake[snake_length - 1] = *tail;
    snake[0] = *head;
    Ok(())
}

fn display_score(
    width: u16,
    height: u16,
    score: u16,
    stdout: &mut Stdout,
) -> Result<(), std::io::Error> {
    stdout.queue(cursor::MoveTo(0, height))?;
    println!("Score: {}", score);
    let msg = "WASD to move, ESC to exit";
    stdout.queue(cursor::MoveTo(width - msg.len() as u16, height))?;
    println!("{}", msg);
    Ok(())
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(cursor::Hide)?;

    let height = 15;
    let width = 40;
    let mut wall: Vec<[i16; 2]> = vec![];
    draw_border(height, width, &mut stdout, &mut wall)?;

    let mut head: [i16; 2] = [width as i16 / 3, height as i16 / 2];
    let mut tail: [i16; 2] = [head[0] - 2, head[1]];
    let mut wake: [i16; 2] = [head[0] - 3, head[1]];
    let mut snake: Vec<[i16; 2]> = vec![head, [head[0] - 1, head[1]], tail];
    let mut apple: [i16; 2] = [0, 0];
    let mut apple_exists = false;
    let mut score = 0;

    // MAIN GAME LOOP
    let mut x = 1;
    let mut y = 0;
    loop {
        let duration = time::Duration::from_millis(100);
        let snake_length = snake.len();
        let mut rng = rand::rng();

        while !apple_exists {
            spawn_apple(
                &mut apple_exists,
                &mut apple,
                &snake,
                &wall,
                width,
                height,
                &mut rng,
                &mut stdout,
            )?;
        }

        if head[0] == apple[0] && head[1] == apple[1] {
            snake.insert(snake_length - 1, [tail[0], tail[1]]);
            apple_exists = false;
            score += 1;
        }

        if snake[1..snake_length].contains(&head) {
            break;
        }

        let direction: [i16; 2] = handle_input(&mut x, &mut y, duration)?;
        if direction == [69, 69] {
            break;
        }

        draw_snake(&snake, snake_length, head, tail, wake, &mut stdout)?;

        move_snake(
            &mut snake,
            snake_length,
            &mut head,
            &mut tail,
            &mut wake,
            direction,
        )?;

        if head[0] == width as i16 - 1 {
            break;
        } else if head[0] == 0 {
            break;
        } else if head[1] == height as i16 - 1 {
            break;
        } else if head[1] == 0 {
            break;
        }

        display_score(width, height, score, &mut stdout)?;

        // can't forget to flush after myself
        stdout.flush()?;
    }

    // and clean up
    disable_raw_mode()?;
    stdout.queue(cursor::MoveTo(0, height + 1))?;
    stdout.execute(cursor::Show)?;

    Ok(())
}
