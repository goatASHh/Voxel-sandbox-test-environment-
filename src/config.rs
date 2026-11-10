use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub creative_mode: bool,
    pub graphics: GraphicsConfig,
    pub streaming: StreamingConfig,
    pub saving: SavingConfig,
    pub mobs: MobsConfig,
    pub time: TimeConfig,
    pub clouds: CloudsConfig,
    pub overlay: OverlayConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            creative_mode: true,
            graphics: GraphicsConfig::default(),
            streaming: StreamingConfig::default(),
            saving: SavingConfig::default(),
            mobs: MobsConfig::default(),
            time: TimeConfig::default(),
            clouds: CloudsConfig::default(),
            overlay: OverlayConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    pub max_fps: u16,
    pub render_scale: f32,
    pub full_detail_distance_chunks: u8,
    pub shadows: bool,
    pub shadow_quality: u8,
    pub ambient_occlusion: bool,
    pub water_reflections: bool,
    pub bloom: bool,
    pub volumetric_cloud_lighting: bool,
    pub fog: bool,
    pub anti_aliasing: bool,
    pub shader_effects_enabled: bool,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            max_fps: 120,
            render_scale: 1.0,
            full_detail_distance_chunks: 8,
            shadows: true,
            shadow_quality: 2,
            ambient_occlusion: true,
            water_reflections: true,
            bloom: false,
            volumetric_cloud_lighting: false,
            fog: true,
            anti_aliasing: true,
            shader_effects_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Chunk data may be resident without geometry being built or drawn.
    pub load_radius_chunks: u8,
    pub full_detail_radius_chunks: u8,
    pub lod_radius_chunks: u8,
    /// Detailed terrain depth below the local surface; does not delete world data.
    pub detailed_depth_below_surface_blocks: u16,
    /// Additional detailed depth activated as the player digs.
    pub dig_expansion_blocks: u8,
    /// Cave detail is requested only near the player or when interacted with.
    pub cave_detail_radius_blocks: u16,
    pub render_unmeshed_chunks: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            load_radius_chunks: 2,
            full_detail_radius_chunks: 8,
            lod_radius_chunks: 16,
            detailed_depth_below_surface_blocks: 5,
            dig_expansion_blocks: 2,
            cave_detail_radius_blocks: 24,
            render_unmeshed_chunks: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingConfig {
    pub background_saving_enabled: bool,
    pub save_interval_seconds: u32,
    pub save_on_exit: bool,
    pub incremental_saving: bool,
}

impl Default for SavingConfig {
    fn default() -> Self {
        Self {
            background_saving_enabled: false,
            save_interval_seconds: 300,
            save_on_exit: true,
            incremental_saving: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobsConfig {
    pub enabled: bool,
    pub spawn_density: f32,
    pub simulation_distance_chunks: u8,
}

impl Default for MobsConfig {
    fn default() -> Self {
        Self { enabled: false, spawn_density: 0.5, simulation_distance_chunks: 4 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConfig {
    pub cycle_enabled: bool,
    pub day_length_minutes: u16,
    /// 0.0 = midnight, 0.25 = sunrise, 0.5 = noon, 0.75 = sunset.
    pub time_of_day: f32,
}

impl Default for TimeConfig {
    fn default() -> Self {
        Self { cycle_enabled: true, day_length_minutes: 20, time_of_day: 0.5 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudsConfig {
    pub enabled: bool,
    pub density: f32,
    pub movement_speed: f32,
    pub altitude_blocks: u16,
    pub quality: u8,
    pub cast_shadows: bool,
}

impl Default for CloudsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            density: 0.55,
            movement_speed: 1.0,
            altitude_blocks: 96,
            quality: 2,
            cast_shadows: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub enabled: bool,
    pub show_fps: bool,
    pub show_frame_time: bool,
    pub show_average_fps: bool,
    pub show_one_percent_low: bool,
    pub show_chunk_activity: bool,
    pub font_scale: f32,
    pub opacity: f32,
    pub anchor_x: f32,
    pub anchor_y: f32,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_fps: true,
            show_frame_time: true,
            show_average_fps: false,
            show_one_percent_low: false,
            show_chunk_activity: false,
            font_scale: 1.0,
            opacity: 0.8,
            anchor_x: 0.03,
            anchor_y: 0.03,
        }
    }
}
