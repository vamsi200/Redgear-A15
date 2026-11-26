mod mouse;
mod protocol;
mod ui;

use raylib::{ffi, prelude::*};
use ui::{HEIGHT, Ui, WIDTH};

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Redgear A15 | Control Panel")
        .build();

    rl.set_target_fps(60);

    let font = rl
        .load_font_ex(&thread, "assets/Iosevka-Nerd-Font-Complete.ttf", 24, None)
        .expect("Failed to load font");

    let mouse_icon = rl
        .load_texture(&thread, "assets/mouse.png")
        .expect("Failed to load mouse icon");

    mouse_icon.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_POINT);

    let raw: &ffi::Font = font.as_ref();

    unsafe {
        ffi::SetTextureFilter(
            raw.texture,
            ffi::TextureFilter::TEXTURE_FILTER_BILINEAR as i32,
        );
    }

    let mut ui = Ui::new(font, mouse_icon);

    while !rl.window_should_close() && !rl.is_key_pressed(KeyboardKey::KEY_Q) {
        ui.update();

        let mut d = rl.begin_drawing(&thread);
        ui.draw(&mut d);
    }
}
