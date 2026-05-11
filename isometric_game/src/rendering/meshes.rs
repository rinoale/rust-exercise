use bevy::prelude::*;

pub fn create_diamond_mesh(width: f32, height: f32) -> Mesh {
    let hw = width * 0.5;
    let hh = height * 0.5;

    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0.0, hh, 0.0],
            [hw, 0.0, 0.0],
            [0.0, -hh, 0.0],
            [-hw, 0.0, 0.0],
        ],
    )
    .with_inserted_indices(bevy::render::mesh::Indices::U32(vec![0, 1, 2, 0, 2, 3]))
}

pub fn create_rect_mesh(width: f32, height: f32) -> Mesh {
    let hw = width * 0.5;

    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-hw, 0.0, 0.0],
            [hw, 0.0, 0.0],
            [hw, height, 0.0],
            [-hw, height, 0.0],
        ],
    )
    .with_inserted_indices(bevy::render::mesh::Indices::U32(vec![0, 1, 2, 0, 2, 3]))
}
