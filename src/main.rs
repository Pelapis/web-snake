use leptos::*;
use rand::prelude::*;
use wasm_bindgen::prelude::*;

// 设定画布的宽高，以及网格的行列数，网格的颜色
const WIDTH: i32 = 500;
const HEIGHT: i32 = 500;
const CELL_NUMBER: i32 = 25;
const GRID_COLOR: &str = "#CCCCCC";

#[component]
fn App() -> impl IntoView {
    view! {
        <header>
            <h1>"贪吃蛇"</h1>
        </header>
        <main>
            <canvas width={format!("{}", WIDTH)} height={format!("{}", HEIGHT)} />
        </main>
        <caption>"上下左右键或者 w a s d 键控制蛇的移动。"</caption>
        <footer>
            "Made by Cavendish."
        </footer>
    }
}

fn main() {
    // 将 App 组件挂载到 body 上
    mount_to_body(App);

    // 获取 canvas 上下文
    let ctx: web_sys::CanvasRenderingContext2d = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector("canvas")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap()
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();

    // 初始化世界
    let mut world = vec![false; CELL_NUMBER.pow(2) as usize];
    // 初始化蛇
    let mut snake = Snake {
        body: vec![
            (CELL_NUMBER * (CELL_NUMBER - 3) + CELL_NUMBER / 2),
            (CELL_NUMBER * (CELL_NUMBER - 2) + CELL_NUMBER / 2),
            (CELL_NUMBER * (CELL_NUMBER - 1) + CELL_NUMBER / 2),
        ],
        head_direction: Direction::Up,
    };
    for &i in snake.body.iter() {
        world[i as usize] = true;
    }
    // 初始化食物
    let food_number = 5;
    let mut food: Vec<i32> = (0..food_number).map(|_| gen_one_food(&world)).collect();
    for &i in food.iter() {
        world[i as usize] = true;
    }
    // 渲染世界
    render_world(&ctx, &world);

    let (pressed_key, set_pressed_key) = create_signal(Direction::None);

    // 监听键盘事件
    let closure = Closure::wrap(Box::new({
        move |event: web_sys::KeyboardEvent| {
            let key = event.key();
            match key.as_str() {
                "ArrowUp" | "w" => set_pressed_key.set(Direction::Up),
                "ArrowDown" | "s" => set_pressed_key.set(Direction::Down),
                "ArrowLeft" | "a" => set_pressed_key.set(Direction::Left),
                "ArrowRight" | "d" => set_pressed_key.set(Direction::Right),
                _ => {}
            }
        }
    }) as Box<dyn FnMut(_)>);
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();

    let interval = 500;

    // 每隔一段时间更新一次世界
    let closure = Closure::wrap(Box::new(move || {
        match pressed_key.get() {
            Direction::Up | Direction::Down => match snake.head_direction {
                Direction::Left | Direction::Right => snake.head_direction = pressed_key.get(),
                _ => {}
            },
            Direction::Left | Direction::Right => match snake.head_direction {
                Direction::Up | Direction::Down => snake.head_direction = pressed_key.get(),
                _ => {}
            },
            Direction::None => {}
        };
        update_world(&mut snake, &mut world, &mut food);
        render_world(&ctx, &world);
        set_pressed_key.set(Direction::None);
    }) as Box<dyn FnMut()>);
    web_sys::window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            interval,
        )
        .unwrap();
    closure.forget();
}

// 画出网格
fn draw_grid(ctx: &web_sys::CanvasRenderingContext2d, grid_color: &str, cell_number: i32) {
    ctx.begin_path();
    ctx.set_stroke_style(&JsValue::from(grid_color));

    // 画出竖线
    for i in (0..=cell_number).map(|i| i * WIDTH as i32 / cell_number) {
        ctx.move_to(i as f64, 0.0);
        ctx.line_to(i as f64, HEIGHT as f64);
    }
    // 画出横线
    for i in (0..=cell_number).map(|i| i * HEIGHT as i32 / cell_number) {
        ctx.move_to(0.0, i as f64);
        ctx.line_to(WIDTH as f64, i as f64);
    }

    ctx.stroke();
}

// 渲染世界
fn draw_cells(ctx: &web_sys::CanvasRenderingContext2d, world: &[bool], cell_number: i32) {
    for y in 0..cell_number {
        for x in 0..cell_number {
            if world[(y * cell_number + x) as usize] {
                ctx.fill_rect(
                    (x * WIDTH / cell_number) as f64,
                    (y * HEIGHT / cell_number) as f64,
                    (WIDTH / cell_number) as f64,
                    (HEIGHT / cell_number) as f64,
                );
            }
        }
    }
}

// 渲染世界
fn render_world(ctx: &web_sys::CanvasRenderingContext2d, world: &Vec<bool>) {
    // 清空画布
    ctx.clear_rect(0.0, 0.0, WIDTH as f64, HEIGHT as f64);
    // 画出网格
    draw_grid(&ctx, GRID_COLOR, CELL_NUMBER);
    // 渲染世界
    draw_cells(&ctx, &world, CELL_NUMBER);
}

// 更新世界
fn update_world(snake: &mut Snake, world: &mut Vec<bool>, food: &mut Vec<i32>) {
    // 更新蛇
    snake.update_snake(world, food);
    // 重新生成世界
    for i in world.iter_mut() {
        *i = false;
    }
    for &i in snake.body.iter() {
        world[i as usize] = true;
    }
    for &i in food.iter() {
        world[i as usize] = true;
    }
}

// 生成一个食物
fn gen_one_food(world: &Vec<bool>) -> i32 {
    match thread_rng().gen_range(0..CELL_NUMBER.pow(2)) {
        x if world[x as usize] => gen_one_food(world),
        x => x,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

// 定义蛇结构
struct Snake {
    body: Vec<i32>,
    head_direction: Direction,
}

impl Snake {
    fn update_snake(&mut self, world: &Vec<bool>, food: &mut Vec<i32>) {
        let new_head = Self::update_body(&self.body[0], &self.head_direction);
        match world[new_head as usize] {
            false => {
                self.body.pop();
                self.body.insert(0, new_head);
            }
            true => {
                for i in food.iter_mut() {
                    if i == &new_head {
                        *i = gen_one_food(&world);
                        self.body.insert(0, new_head);
                        return;
                    }
                }
                // 碰到自己，游戏结束
                web_sys::window()
                    .unwrap()
                    .alert_with_message(
                        format!("游戏结束！您的得分是：{}！", self.body.len()).as_str(),
                    )
                    .unwrap();
                // 刷新页面
                web_sys::window().unwrap().location().reload().unwrap();
            }
        }
    }

    fn update_body(&head: &i32, &direction: &Direction) -> i32 {
        match direction {
            Direction::Up => match head / CELL_NUMBER {
                x if x == 0 => head - CELL_NUMBER + CELL_NUMBER.pow(2),
                _ => head - CELL_NUMBER,
            },
            Direction::Down => match head / CELL_NUMBER {
                x if x == CELL_NUMBER - 1 => head + CELL_NUMBER - CELL_NUMBER.pow(2),
                _ => head + CELL_NUMBER,
            },
            Direction::Left => match head % CELL_NUMBER {
                0 => head - 1 + CELL_NUMBER,
                _ => head - 1,
            },
            Direction::Right => match head % CELL_NUMBER {
                x if x == CELL_NUMBER - 1 => head + 1 - CELL_NUMBER,
                _ => head + 1,
            },
            Direction::None => head,
        }
    }
}
