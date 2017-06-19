#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate gfx_text;
extern crate image;
extern crate winit;

use gfx::format::{DepthStencil, Rgba8};
use gfx::Device;
use gfx::Factory;
use gfx::traits::FactoryExt;
use winit::KeyboardInput;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    constant Transform {
        transform: [[f32; 4]; 4] = "u_Transform",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::ConstantBuffer<Transform> = "Transform",
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
    }
}

fn main() {

    let builder = glutin::WindowBuilder::new()
        .with_dimensions(100, 100)
        .with_title("Triangle example".to_string());

    let events_loop = glutin::EventsLoop::new();
    let (window, mut device, mut factory, mut color_view, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, &events_loop);


    println!("{:?}", window.get_inner_size());
    println!("{:?}", color_view.get_dimensions());
    println!("{:?}", main_depth.get_dimensions());

    println!("{:?}", window.hidpi_factor());

    let img = image::open("resources/tiles.png").expect("failed to open image").to_rgba();
    let (width, height) = img.dimensions();
    let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);

    let (_, texture) = factory.create_texture_immutable_u8::<gfx::format::Rgba8>(kind, &[&img]).expect("aeou");
    let sampler = factory.create_sampler_linear();

    let mut normal_text = gfx_text::new(factory.clone()).unwrap();

    let pso = factory.create_pipeline_simple(
        include_bytes!("shaders/shdr_150.vert"),
        include_bytes!("shaders/shdr_150.frag"),
        pipe::new()
    ).expect("Failed to create pipeline");

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    const TRIANGLE: [Vertex; 3] = [
        Vertex { pos: [-1.0, 1.0], uv: [0.0, 0.0] },
        Vertex { pos: [0.0, 0.0], uv: [1.0, 0.0] },
        Vertex { pos: [ -1.0,  -1.0], uv: [0.0, 1.0] },
    ];

    const TRANSFORM: Transform = Transform {
        transform: [[1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0]]
    };

    const CLEAR_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];

    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&TRIANGLE, ());
    let transform_buffer = factory.create_constant_buffer(1);

    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        transform: transform_buffer,
        tex: (texture, sampler),
        out: color_view,
    };

    'main: loop {
        let mut running = true;
        events_loop.poll_events(|e| {
            let event = if let glutin::Event::WindowEvent { event, .. } = e {
                event
            } else {
                return;
            };
            match event {
//                glutin::WindowEvent::KeyboardInput {  input: KeyboardInput { virtual_keycode: Some(glutin::VirtualKeyCode::Escape), .. }, .. } |
                glutin::WindowEvent::Closed => running = false,
                glutin::WindowEvent::Resized(_, _) => {
                   //window.set_inner_size(1024, 768);
                    gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                    println!("{:?}", window.get_inner_size());
                    println!("{:?}", data.out.get_dimensions());
                    println!("{:?}", main_depth.get_dimensions());
                }
                _ => {}
            }
        });

        if !running {
            break;
        }

        encoder.clear(&data.out, CLEAR_COLOR);
        normal_text.add("The quick brown fox jumps over the lazy dog", [10, 10], [1.0, 1.0, 1.0, 0.5]);
        normal_text.draw(&mut encoder, &data.out).unwrap();
        encoder.update_buffer(&data.transform, &[TRANSFORM], 0).unwrap();;
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);

        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
