pub mod canvas_rs;

use canvas_rs::{
    Canvas,
    Context2d
};


fn main() {
    let mut window = canvas_rs::WinitCanvas::new().expect("Window creation failed!");
    window.set_properties(canvas_rs::CanvasProperties {
        width : 1000,
        height : 1000,
        resizable : false
    });
    window.event_loop(|evt| {
        println!("event");
    }, |ctx : &mut dyn Context2d| {
        ctx.fill_rect(0.0, 0.0, 0.0, 0.0);
    }).unwrap();
}
