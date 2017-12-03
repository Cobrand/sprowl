extern crate sdl2;
extern crate sprowl;
extern crate gl;

use sdl2::keyboard::Keycode;
use sdl2::event::{Event, WindowEvent};
use sprowl::*;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(::sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    // Enable anti-aliasing
    gl_attr.set_multisample_buffers(1);
    gl_attr.set_multisample_samples(4);
    
    let window = video_subsystem.window("Window", 800, 600)
        .resizable()
        .opengl()
        .build()
        .unwrap();

    let _ctx = window.gl_create_context().unwrap();
    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
    
    // Yes, we're still using the Core profile
    debug_assert_eq!(gl_attr.context_profile(), sdl2::video::GLProfile::Core);
    // ... and we're still using OpenGL 3.3
    debug_assert_eq!(gl_attr.context_version(), (3, 3));

    let mut canvas = {
        let (w, h) = window.size();
        Canvas::new((0, 0, w, h)).unwrap()
    };

    let stick_id = canvas.add_texture_from_image_path("res/stick.png").unwrap();
    let font_id = canvas.add_font_from_bytes(include_bytes!("/usr/share/fonts/TTF/DejaVuSansMono.ttf"));
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut entity_x: i32 = 500;
    let mut entity_y: i32 = 500;
    'running: loop {
        let t0 = ::std::time::Instant::now();
        let window_size = window.size();
        unsafe {
            gl::Viewport(0, 0, window_size.0 as i32, window_size.1 as i32);
        }
        canvas.clear(Some(Color::from_rgb(128u8, 128, 128)));
        let graphic_entities: Vec<sprowl::GraphicEntity> = vec!(
            GraphicEntity::Texture {
                id: stick_id,
                repr: Graphic2DRepresentation::WorldAbsolute {
                    x: 0,
                    y: 0,
                },
                render_options: RenderOptions {
                    filter_color: None,
                    blend_color: Some(Color::from_rgba(255, 255, 255, 192)),
                    outline: Some((5.0, Color::from_rgb(0, 0, 255))),
                    // outline: None,
                    flip: Flip::None
                },
                scale: Some(0.1)
            },
            GraphicEntity::Texture {
                id: stick_id,
                repr: Graphic2DRepresentation::WorldAbsolute {
                    x: entity_x,
                    y: entity_y,
                },
                render_options: RenderOptions {
                    filter_color: None,
                    blend_color: Some(Color::from_rgba(255, 255, 255, 192)),
                    outline: Some((5.0, Color::from_rgb(0, 0, 255))),
                    // outline: None
                    flip: Flip::Both
                },
                scale: None,
            },
            GraphicEntity::Texture {
                id: stick_id,
                repr: Graphic2DRepresentation::WorldAbsolute {
                    x: 10,
                    y: 10,
                },
                render_options: RenderOptions {
                    filter_color: None,
                    blend_color: Some(Color::from_rgba(255, 255, 255, 192)),
                    outline: Some((5.0, Color::from_rgb(0, 0, 255))),
                    // outline: None
                    flip: Flip::Vertical
                },
                scale: None,
            },
            GraphicEntity::Texture {
                id: stick_id,
                repr: Graphic2DRepresentation::CameraRelative {
                    position: CameraRelativePosition::FromBottomLeft(10, 10),
                },
                render_options: Default::default(),
                scale: None
            },
            GraphicEntity::Text {
                font_id: font_id,
                repr: Graphic2DRepresentation::CameraRelative {
                    position: CameraRelativePosition::FromTopRight(10, 10),
                },
                render_options: RenderOptions {
                    outline: Some((2.0, Color::from_rgb(0, 0, 255))),
                    ..Default::default()
                },
                text: "Pote",
                font_size: 30.0,
                color: None,
            },
        );
        canvas.draw(&graphic_entities);
        window.gl_swap_window();
        let (cam_x, cam_y, _, _) = canvas.camera_bounds();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::Window { win_event: WindowEvent::SizeChanged(w, h), ..} => {
                    debug_assert!(w >= 0);
                    debug_assert!(h >= 0);
                    canvas.set_camera_size((w as u32, h as u32));
                },
                Event::KeyDown { keycode: Some(Keycode::Kp8), repeat: false, ..} =>
                    canvas.set_camera_position((cam_x, cam_y - 50)),
                Event::KeyDown { keycode: Some(Keycode::Kp2), repeat: false, ..} =>
                    canvas.set_camera_position((cam_x, cam_y + 50)),
                Event::KeyDown { keycode: Some(Keycode::Kp6), repeat: false, ..} =>
                    canvas.set_camera_position((cam_x + 50, cam_y)),
                Event::KeyDown { keycode: Some(Keycode::Kp4), repeat: false, ..} =>
                    canvas.set_camera_position((cam_x - 50, cam_y)),
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
                Event::KeyDown { keycode: Some(Keycode::KpPlus), repeat: false, ..} => {
                    let zoom_level = canvas.zoom_level();
                    canvas.set_zoom_level(zoom_level * 1.2)
                },
                Event::KeyDown { keycode: Some(Keycode::KpMinus), repeat: false, ..} => {
                    let zoom_level = canvas.zoom_level();
                    canvas.set_zoom_level(zoom_level / 1.2)
                },
                _ => {}
            }
        }

        let delta_t = ::std::time::Instant::now() - t0;
        println!("{}", 1_000_000_000 / delta_t.subsec_nanos());
        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 30));
    }
}