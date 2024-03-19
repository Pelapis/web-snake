use rand::prelude::*;
use wasm_bindgen::prelude::*;

// 设定画布的宽高，以及网格的行列数，网格的颜色
const CELL_NUMBER: i32 = 25;
const GRID_COLOR: &str = "#CCCCCC";

fn main() {
    // 获取 canvas 上下文
    let canvas = gloo::utils::document()
        .query_selector("canvas")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
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

    let (sender, pressed_key) = std::sync::mpsc::sync_channel::<Direction>(1024);
    let key_sender = sender.clone();
    // 监听键盘事件
    gloo::events::EventListener::new(&gloo::utils::document_element(), "keydown", move |x| {
        let event = x.dyn_ref::<web_sys::KeyboardEvent>().unwrap();
        let key = event.key();
        match key.as_str() {
            "ArrowUp" | "w" => key_sender.try_send(Direction::Up).unwrap(),
            "ArrowDown" | "s" => key_sender.try_send(Direction::Down).unwrap(),
            "ArrowLeft" | "a" => key_sender.try_send(Direction::Left).unwrap(),
            "ArrowRight" | "d" => key_sender.try_send(Direction::Right).unwrap(),
            _ => {}
        }
    })
    .forget();

    // 监听手机触摸事件，点击画布上下左右
    gloo::events::EventListener::new(&gloo::utils::document_element(), "touchstart", move |x| {
        let event = x.dyn_ref::<web_sys::TouchEvent>().unwrap();
        let touch = event.touches().get(0).unwrap();
        let x = touch.client_x() as f64;
        let y = touch.client_y() as f64;
        let canvas_rect = canvas.get_bounding_client_rect();
        // 写出对角线方程，判断点击的位置
        let up_line1 = y
            <= canvas.height() as f64 / canvas.width() as f64 * (x - canvas_rect.x())
                + canvas_rect.y();
        let up_line2 = y
            <= canvas.height() as f64 / canvas.width() as f64 * (canvas_rect.right() - x)
                + canvas_rect.y();
        match (up_line1, up_line2) {
            (true, true) => sender.try_send(Direction::Up).unwrap(),
            (true, false) => sender.try_send(Direction::Right).unwrap(),
            (false, true) => sender.try_send(Direction::Left).unwrap(),
            (false, false) => sender.try_send(Direction::Down).unwrap(),
        }
    })
    .forget();

    // 设置定时器，每隔一段时间更新一次世界
    let interval = 400;
    gloo::timers::callback::Interval::new(interval, move || {
        // 取出最后一个键盘输入或手机触摸输入
        let mut order_direction = Direction::None;
        while let Ok(direction) = pressed_key.try_recv() {
            order_direction = direction;
        }
        match order_direction {
            Direction::Up | Direction::Down => match snake.head_direction {
                Direction::Left | Direction::Right => snake.head_direction = order_direction,
                _ => {}
            },
            Direction::Left | Direction::Right => match snake.head_direction {
                Direction::Up | Direction::Down => snake.head_direction = order_direction,
                _ => {}
            },
            Direction::None => {}
        };
        update_world(&mut snake, &mut world, &mut food);
        render_world(&ctx, &world);
    })
    .forget();
}

// 画出网格
fn draw_grid(ctx: &web_sys::CanvasRenderingContext2d, grid_color: &str, cell_number: i32) {
    ctx.begin_path();
    ctx.set_stroke_style(&JsValue::from(grid_color));

    // 画出竖线
    for i in (0..=cell_number).map(|i| i * ctx.canvas().unwrap().width() as i32 / cell_number) {
        ctx.move_to(i as f64, 0.0);
        ctx.line_to(i as f64, ctx.canvas().unwrap().height() as f64);
    }
    // 画出横线
    for i in (0..=cell_number).map(|i| i * ctx.canvas().unwrap().height() as i32 / cell_number) {
        ctx.move_to(0.0, i as f64);
        ctx.line_to(ctx.canvas().unwrap().width() as f64, i as f64);
    }

    ctx.stroke();
}

// 渲染世界
fn draw_cells(ctx: &web_sys::CanvasRenderingContext2d, world: &[bool], cell_number: i32) {
    for y in 0..cell_number {
        for x in 0..cell_number {
            if world[(y * cell_number + x) as usize] {
                ctx.fill_rect(
                    (x * ctx.canvas().unwrap().width() as i32 / cell_number) as f64,
                    (y * ctx.canvas().unwrap().height() as i32 / cell_number) as f64,
                    (ctx.canvas().unwrap().width() as i32 / cell_number) as f64,
                    (ctx.canvas().unwrap().height() as i32 / cell_number) as f64,
                );
            }
        }
    }
}

// 渲染世界
fn render_world(ctx: &web_sys::CanvasRenderingContext2d, world: &Vec<bool>) {
    // 清空画布
    ctx.clear_rect(
        0.0,
        0.0,
        ctx.canvas().unwrap().width() as f64,
        ctx.canvas().unwrap().height() as f64,
    );
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
