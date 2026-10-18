pub const NUM_HARES: usize = 2;
pub const HARE_MAX_SPEED: f32 = 50.0;

pub const MAX_BERRIES: u64 = 2;

pub const BERRY_RENDER_HEIGHT: f32 = 5.0;
pub const BERRY_RENDER_WIDTH: f32 = 5.0;

pub const HARE_SPRITE_HEIGHT: u32 = 32;
pub const HARE_SPRITE_WIDTH: u32 = 32;

pub const MAP_COLUMNS: usize = 8;
pub const MAP_ROWS: usize = 8;
pub const TILE_WIDTH: f32 = 32.0;
pub const TILE_HEIGHT: f32 = 32.0;
pub const MAP_WIDTH: f32 = TILE_WIDTH * MAP_COLUMNS as f32;
pub const MAP_HEIGHT: f32 = TILE_HEIGHT * MAP_ROWS as f32;
pub const PLAYABLE_AREA_X0: f32 = -MAP_WIDTH / 2.0 + TILE_WIDTH;
pub const PLAYABLE_AREA_X1: f32 = MAP_WIDTH / 2.0 - TILE_WIDTH;
pub const PLAYABLE_AREA_Y0: f32 = -MAP_HEIGHT / 2.0 + TILE_HEIGHT;
pub const PLAYABLE_AREA_Y1: f32 = MAP_HEIGHT / 2.0 - TILE_HEIGHT;
