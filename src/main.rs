use macroquad::prelude::*;

mod audio;
use audio::AudioPlayer;

const TILE_SIZE: f32 = 32.0;
const MAP_WIDTH: i32 = 50;
const MAP_HEIGHT: i32 = 50;
const MOVE_SPEED: f32 = 200.0;

const GAME_WIDTH: f32 = 512.0; //850.0;
const GAME_HEIGHT: f32 = 512.0; //480.0;

#[macroquad::main("Macroquad Tilemap")]
async fn main() {
    let mut world_target = vec2(
        MAP_WIDTH as f32 * TILE_SIZE / 2.0,
        MAP_HEIGHT as f32 * TILE_SIZE / 2.0
    );

    let render_target = render_target(GAME_WIDTH as u32, GAME_HEIGHT as u32);
    //render_target.texture.set_filter(FilterMode::Nearest);

    let spritesheet = load_texture("assets/default_dust.png").await.unwrap();
    //spritesheet.set_filter(FilterMode::Nearest);

    let mut audio = AudioPlayer::new();
    audio.play_file("assets/unrealsoftware.wav", 0.5, [-50.0, 0.0], true);

    loop {
        let delta = get_frame_time();

        if is_key_down(KeyCode::Left)  { world_target.x -= MOVE_SPEED * delta; }
        if is_key_down(KeyCode::Right) { world_target.x += MOVE_SPEED * delta; }
        if is_key_down(KeyCode::Up)    { world_target.y -= MOVE_SPEED * delta; }
        if is_key_down(KeyCode::Down)  { world_target.y += MOVE_SPEED * delta; }

        // ---------------------------------------------------------
        // 1. Calculate Logical Scaling
        // ---------------------------------------------------------
        let s_width = screen_width().max(1.0);
        let s_height = screen_height().max(1.0);

        // Find the maximum scale that fits our 850x480 game inside the window
        let scale = (s_width / GAME_WIDTH).min(s_height / GAME_HEIGHT);

        // Convert the physical screen size into our logical camera space.
        // This tells the camera exactly how much "world" it needs to see.
        let visible_width = s_width / scale;
        let visible_height = s_height / scale;

        // ---------------------------------------------------------
        // 2. Draw the Game World
        // ---------------------------------------------------------
        let cam = Camera2D {
            render_target: Some(render_target.clone()),
            target: vec2(world_target.x.floor(), world_target.y.floor()),
            zoom: vec2(2.0 / GAME_WIDTH, 2.0 / GAME_HEIGHT),
            ..Default::default()
        };
        set_camera(&cam);

        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                draw_texture_ex(
                    &spritesheet,
                    x as f32 * TILE_SIZE, y as f32 * TILE_SIZE,
                    WHITE,
                    DrawTextureParams {
                        source: Option::from(Rect {
                            x: TILE_SIZE * (1 + x % 2) as f32,
                            y: 0.0,
                            w: TILE_SIZE,
                            h: TILE_SIZE
                        }),
                        ..Default::default()
                    }
                );
            }
        }

        // ---------------------------------------------------------
        // 3. Draw the UI overlay (Fixed to 850x480 space)
        // ---------------------------------------------------------
        // By using the exact same zoom, but pointing the camera at the
        // center of our logical 850x480 area, our UI renders perfectly.
        let ui_cam = Camera2D {
            target: vec2(GAME_WIDTH / 2.0, GAME_HEIGHT / 2.0),
            zoom: vec2(2.0 / visible_width, 2.0 / visible_height),
            ..Default::default()
        };
        set_camera(&ui_cam);

        set_default_camera();
        clear_background(BLACK);

        let scale = (screen_width() / GAME_WIDTH).min(screen_height() / GAME_HEIGHT);
        let w = GAME_WIDTH * scale;
        let h = GAME_HEIGHT * scale;
        let x = (screen_width() - w) / 2.0;
        let y = (screen_height() - h) / 2.0;

        draw_texture_ex(
            &render_target.texture,
            x, y, WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                ..Default::default()
            },
        );

        draw_text("Use Arrow Keys to Scroll", 20.0, 30.0, 20.0, WHITE);

        let fps_text = format!("FPS: {}", get_fps());
        let text_dimensions = measure_text(&fps_text, None, 20, 1.0);
        draw_text(
            &fps_text,
            screen_width() - text_dimensions.width - 10.0,
            20.0 + text_dimensions.offset_y,
            20.0,
            GREEN
        );

        next_frame().await
    }
}