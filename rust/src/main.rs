mod sprites;

use ncurses::*;
use sprites::*;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_sigint(_: libc::c_int) {
    INTERRUPTED.store(true, Ordering::Relaxed);
}

struct Options {
    accident: bool,
    fly: bool,
    logo: bool,
    c51: bool,
}

impl Options {
    fn parse(args: &[String]) -> Self {
        let mut opts = Options {
            accident: false,
            fly: false,
            logo: false,
            c51: false,
        };
        for arg in args {
            if let Some(flags) = arg.strip_prefix('-') {
                for ch in flags.chars() {
                    match ch {
                        'a' => opts.accident = true,
                        'F' => opts.fly = true,
                        'l' => opts.logo = true,
                        'c' => opts.c51 = true,
                        _ => {}
                    }
                }
            }
        }
        opts
    }
}

struct SmokeParticle {
    y: i32,
    x: i32,
    ptrn: usize,
    kind: usize,
}

struct SmokeState {
    particles: Vec<SmokeParticle>,
}

impl SmokeState {
    fn new() -> Self {
        SmokeState {
            particles: Vec::new(),
        }
    }

    fn add(&mut self, y: i32, x: i32) {
        if x % 4 != 0 {
            return;
        }
        for p in &mut self.particles {
            my_mvaddstr(p.y, p.x, ERASER[p.ptrn]);
            p.y -= SMOKE_DY[p.ptrn];
            p.x += SMOKE_DX[p.ptrn];
            if p.ptrn < SMOKE_PTNS - 1 {
                p.ptrn += 1;
            }
            my_mvaddstr(p.y, p.x, SMOKE[p.kind][p.ptrn]);
        }
        let kind = self.particles.len() % 2;
        my_mvaddstr(y, x, SMOKE[kind][0]);
        self.particles.push(SmokeParticle {
            y,
            x,
            ptrn: 0,
            kind,
        });
    }
}

fn my_mvaddstr(y: i32, x: i32, s: &str) {
    let mut x = x;
    let mut chars = s.chars();
    while x < 0 {
        if chars.next().is_none() {
            return;
        }
        x += 1;
    }
    for ch in chars {
        if mvaddch(y, x, ch as u32) == ERR {
            return;
        }
        x += 1;
    }
}

fn add_man(y: i32, x: i32) {
    let idx = ((LOGO_LENGTH + x) / 12 % 2) as usize;
    for (i, line) in MAN[idx].iter().enumerate() {
        my_mvaddstr(y + i as i32, x, line);
    }
}

fn add_d51(x: i32, opts: &Options, smoke: &mut SmokeState) -> bool {
    if x < -D51_LENGTH {
        return false;
    }
    let mut y = LINES() / 2 - 5;
    let mut dy = 0;
    if opts.fly {
        y = (x / 7) + LINES() - (COLS() / 7) - D51_HEIGHT;
        dy = 1;
    }
    let ptrn = ((D51_LENGTH + x) % D51_PATTERNS) as usize;
    for i in 0..7 {
        my_mvaddstr(y + i as i32, x, D51_BODY[i]);
    }
    for i in 0..3 {
        my_mvaddstr(y + 7 + i as i32, x, D51_WHEELS[ptrn][i]);
    }
    my_mvaddstr(y + 10, x, D51_DEL);
    for i in 0..10 {
        my_mvaddstr(y + i as i32 + dy, x + 53, COAL[i]);
    }
    my_mvaddstr(y + 10 + dy, x + 53, COAL_DEL);
    if opts.accident {
        add_man(y + 2, x + 43);
        add_man(y + 2, x + 47);
    }
    smoke.add(y - 1, x + D51_FUNNEL);
    true
}

fn add_c51(x: i32, opts: &Options, smoke: &mut SmokeState) -> bool {
    if x < -C51_LENGTH {
        return false;
    }
    let mut y = LINES() / 2 - 5;
    let mut dy = 0;
    if opts.fly {
        y = (x / 7) + LINES() - (COLS() / 7) - C51_HEIGHT;
        dy = 1;
    }
    let ptrn = ((C51_LENGTH + x) % C51_PATTERNS) as usize;
    for i in 0..7 {
        my_mvaddstr(y + i as i32, x, C51_BODY[i]);
    }
    for i in 0..4 {
        my_mvaddstr(y + 7 + i as i32, x, C51_WHEELS[ptrn][i]);
    }
    my_mvaddstr(y + 11, x, C51_DEL);
    // C51 coal car has a leading COALDEL row
    my_mvaddstr(y + dy, x + 55, COAL_DEL);
    for i in 0..10 {
        my_mvaddstr(y + 1 + i as i32 + dy, x + 55, COAL[i]);
    }
    my_mvaddstr(y + 11 + dy, x + 55, COAL_DEL);
    if opts.accident {
        add_man(y + 3, x + 45);
        add_man(y + 3, x + 49);
    }
    smoke.add(y - 1, x + C51_FUNNEL);
    true
}

fn add_sl(x: i32, opts: &Options, smoke: &mut SmokeState) -> bool {
    if x < -LOGO_LENGTH {
        return false;
    }
    let mut y = LINES() / 2 - 3;
    let (mut py1, mut py2, mut py3) = (0, 0, 0);
    if opts.fly {
        y = (x / 6) + LINES() - (COLS() / 6) - LOGO_HEIGHT;
        py1 = 2;
        py2 = 4;
        py3 = 6;
    }
    let ptrn = ((LOGO_LENGTH + x) / 3 % LOGO_PATTERNS) as usize;
    for i in 0..4 {
        my_mvaddstr(y + i as i32, x, LOGO_BODY[i]);
    }
    for i in 0..2 {
        my_mvaddstr(y + 4 + i as i32, x, LOGO_WHEELS[ptrn][i]);
    }
    my_mvaddstr(y + 6, x, LOGO_DEL);
    for i in 0..6 {
        my_mvaddstr(y + i as i32 + py1, x + 21, LCOAL[i]);
    }
    my_mvaddstr(y + 6 + py1, x + 21, LOGO_DEL);
    for i in 0..6 {
        my_mvaddstr(y + i as i32 + py2, x + 42, LCAR[i]);
    }
    my_mvaddstr(y + 6 + py2, x + 42, LOGO_DEL);
    for i in 0..6 {
        my_mvaddstr(y + i as i32 + py3, x + 63, LCAR[i]);
    }
    my_mvaddstr(y + 6 + py3, x + 63, LOGO_DEL);
    if opts.accident {
        add_man(y + 1, x + 14);
        add_man(y + 1 + py2, x + 45);
        add_man(y + 1 + py2, x + 53);
        add_man(y + 1 + py3, x + 66);
        add_man(y + 1 + py3, x + 74);
    }
    smoke.add(y - 1, x + LOGO_FUNNEL);
    true
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let opts = Options::parse(&args[1..]);

    initscr();
    unsafe {
        if env::var_os("SL_ESCAPABLE").is_some() {
            libc::signal(libc::SIGINT, handle_sigint as *const () as libc::sighandler_t);
        } else {
            libc::signal(libc::SIGINT, libc::SIG_IGN);
        }
    }
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    nodelay(stdscr(), true);
    leaveok(stdscr(), true);
    scrollok(stdscr(), false);

    let mut smoke = SmokeState::new();

    let mut x = COLS() - 1;
    loop {
        let ok = if opts.logo {
            add_sl(x, &opts, &mut smoke)
        } else if opts.c51 {
            add_c51(x, &opts, &mut smoke)
        } else {
            add_d51(x, &opts, &mut smoke)
        };
        if !ok || INTERRUPTED.load(Ordering::Relaxed) {
            break;
        }
        getch();
        refresh();
        thread::sleep(Duration::from_micros(40000));
        x -= 1;
    }
    clear();
    refresh();
    mvcur(0, COLS() - 1, LINES() - 1, 0);
    endwin();
}
