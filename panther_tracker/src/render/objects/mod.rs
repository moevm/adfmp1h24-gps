pub mod image;
pub mod r#box;
pub mod animated_image;


#[rustfmt::skip]
pub static SQUAD_VERTEX_DATA: [f32; 12] = [
    -1.0, -1.0,
    1.0,  1.0,
    1.0, -1.0,
    -1.0, -1.0,
    -1.0,  1.0,
    1.0,  1.0,
];