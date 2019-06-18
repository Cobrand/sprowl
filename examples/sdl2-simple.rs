use sdl2;
use sprowl;
use gl;

use sdl2::keyboard::Keycode;
use sdl2::event::{Event, WindowEvent};
use sprowl::*;

fn vanilla(sdl_context: &sdl2::Sdl, window: &sdl2::video::Window, mut canvas: Canvas) {
    use sprowl::shaders::vanilla::*;
    let stick_id = canvas.add_texture_from_image_path("res/stick.png").unwrap();
    let font_id = canvas.add_font_from_bytes(include_bytes!("/usr/share/fonts/TTF/DejaVuSansMono.ttf"));
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut entity_x: i32 = 0;
    let mut entity_y: i32 = 0;

    let mut shader = VanillaShader::new().unwrap();

    'running: for t in 0.. {
        let t0 = ::std::time::Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::Window { win_event: WindowEvent::SizeChanged(w, h), ..} => {
                    debug_assert!(w >= 0);
                    debug_assert!(h >= 0);
                    canvas.set_size((w as u32, h as u32));
                },
                Event::KeyDown { keycode: Some(Keycode::Up), repeat: false, ..} => {
                    entity_y -= 50;
                },
                Event::KeyDown { keycode: Some(Keycode::Down), repeat: false, ..} => {
                    entity_y += 50;
                },
                Event::KeyDown { keycode: Some(Keycode::Left), repeat: false, ..} => {
                    entity_x -= 50;
                },
                Event::KeyDown { keycode: Some(Keycode::Right), repeat: false, ..} => {
                    entity_x += 50;
                },
                _ => {}
            }
        }
        canvas.clear(Some(Color::from_rgb(128u8, 128, 128)));

        let graphic_elements: Vec<sprowl::GraphicElement<&'static str, VanillaRenderParams>> = vec!(
            sprowl::GraphicElement {
                graphic_entity: GraphicEntity::Texture { id: stick_id },
                render_params: VanillaRenderParams {
                    position: Position { origin: Origin::Center, pos_x: entity_x, pos_y: entity_y },
                    rotate: Some(RotateOptions { origin: Origin::Center, angle: (t % 360) as f32 })
                }
            },
            sprowl::GraphicElement {
                graphic_entity: GraphicEntity::Text { font_id, text: "Pote", font_size: 32.0, color: None },
                render_params: VanillaRenderParams {
                    position: Position { origin: Origin::TopLeft(0, 0), pos_x: 10, pos_y: 10 },
                    rotate: None,
                }
            },
        );
        canvas.draw(&mut shader, &graphic_elements);
        window.gl_swap_window();

        let _delta_t = ::std::time::Instant::now() - t0;
        // println!("{} fps (theory)", 1_000_000_000 / delta_t.subsec_nanos());
        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn advanced(sdl_context: &sdl2::Sdl, window: &sdl2::video::Window, mut canvas: Canvas) {
    use sprowl::shaders::advanced::*;
    let characters_id = canvas.add_texture_from_image_path("res/characters.png").unwrap();
    let shapes_id = canvas.add_texture_from_image_path("res/shapes.png").unwrap();
    let font_id = canvas.add_font_from_bytes(include_bytes!("/usr/share/fonts/TTF/DejaVuSansMono.ttf"));
    let mut event_pump = sdl_context.event_pump().unwrap();


    let mut shader = AdvancedShader::new().unwrap();

    let mut outline = false;
    let mut scale = false;
    'running: for t in 0.. {
        let t0 = ::std::time::Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::Window { win_event: WindowEvent::SizeChanged(w, h), ..} => {
                    debug_assert!(w >= 0);
                    debug_assert!(h >= 0);
                    canvas.set_size((w as u32, h as u32));
                },
                Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::A), repeat: false, .. } => {
                    outline = !outline;
                },
                Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::E), repeat: false, .. } => {
                    scale = !scale;
                },
                _ => {}
            }
        }
        canvas.clear(Some(Color::from_rgb(128u8, 128, 128)));

        let graphic_elements: Vec<sprowl::GraphicElement<&'static str, AdvancedRenderParams>> = vec!(
            sprowl::GraphicElement {
                graphic_entity: GraphicEntity::Texture { id: shapes_id },
                render_params: AdvancedRenderParams {
                    position: Position { origin: Origin::Center , pos_x: 100, pos_y: 100 },
                    outline: if outline { Some(Color::from_rgb(0u8, 0, 255)) } else { None },
                    crop: None,
                    rotate: None,
                    scale: None,
                    effect: 0,
                    background_color: None,
                    t: t as f32 / 10.0,
                }
            },
            sprowl::GraphicElement {
                graphic_entity: GraphicEntity::Texture { id: characters_id },
                render_params: AdvancedRenderParams {
                    position: Position { origin: Origin::TopLeft(0, 0) , pos_x: 100, pos_y: 100 },
                    outline: if outline { Some(Color::from_rgb(0u8, 0, 255)) } else { None },
                    crop: Some((32, 160, 32, 32)),
                    rotate: Some((t as f32, Origin::Center)),
                    scale: if scale { Some(3.0) } else { None },
                    effect: 0,
                    background_color: None,
                    t: t as f32 / 10.0,
                }
            },
            sprowl::GraphicElement {
                graphic_entity: GraphicEntity::Shape { shape: crate::Shape::Rect(200, 100) },
                render_params: AdvancedRenderParams {
                    position: Position { origin: Origin::Center, pos_x: 200, pos_y: 200 },
                    outline: None,
                    crop: None,
                    rotate: None,
                    scale: None,
                    effect: 1,
                    background_color: None,
                    t: t as f32 / 10.0,
                }
            },
            sprowl::GraphicElement {
                graphic_entity: GraphicEntity::Shape { shape: crate::Shape::Rect(50, 50) },
                render_params: AdvancedRenderParams {
                    position: Position { origin: Origin::Center, pos_x: 300, pos_y: 300 },
                    outline: None,
                    crop: None,
                    rotate: None,
                    scale: None,
                    effect: 2,
                    background_color: Some(Color::from_rgba(64, 64, 64u8, 255u8)),
                    t: t as f32 / 10.0,
                }
            },
            sprowl::GraphicElement {
                graphic_entity: GraphicEntity::Text { font_id, text: "Potekek", font_size: 30.0, color: None },
                render_params: AdvancedRenderParams {
                    position: Position { origin: Origin::Center, pos_x: 0, pos_y: 0 },
                    outline: if outline { Some(Color::from_rgb(0u8, 0, 255)) } else { None },
                    crop: None,
                    rotate: Some((3.0 * t as f32, Origin::Center)),
                    scale: if scale { Some(3.0) } else { None },
                    effect: 0,
                    background_color: Some(Color::from_rgb(128u8, 128u8, 128u8)),
                    t: t as f32 / 10.0,
                }
            },
        );
        canvas.draw(&mut shader, &graphic_elements);
        window.gl_swap_window();

        let _delta_t = ::std::time::Instant::now() - t0;
        // println!("{} fps (theory)", 1_000_000_000 / delta_t.subsec_nanos());
        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(::sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    // // Enable anti-aliasing
    // gl_attr.set_multisample_buffers(1);
    // gl_attr.set_multisample_samples(1);
    
    let window = video_subsystem.window("Window", 1280, 720)
        .opengl()
        .build()
        .unwrap();

    let _ctx = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
    
    // Yes, we're still using the Core profile
    debug_assert_eq!(gl_attr.context_profile(), sdl2::video::GLProfile::Core);
    // ... and we're still using OpenGL 3.3
    debug_assert_eq!(gl_attr.context_version(), (3, 3));

    let canvas = {
        let (w, h) = window.size();
        Canvas::new((w, h))
    };

    let args = ::std::env::args().skip(1).collect::<Vec<_>>();
    match args.get(0) {
        Some(s) if s == "advanced" => {
            println!("running advanced shader");
            advanced(&sdl_context, &window, canvas);
        },
        _ => {
            println!("running default: vanilla shader");
            vanilla(&sdl_context, &window, canvas);
        }
    }
}