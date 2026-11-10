mod block;
mod chunk;
mod config;
mod terrain;
mod renderer;
mod voxel_raycast;
mod save;

use macroquad::prelude::*;
use config::GameConfig;
use renderer::{build_chunk_meshes, draw_chunk_meshes};
use block::BlockId;
use chunk::{Chunk, CHUNK_SIZE, WORLD_HEIGHT};
use terrain::TerrainGenerator;
use voxel_raycast::{raycast_voxels, VoxelHit};

const WORLD_SEED: u64 = 0x504F_434F_F600_0001;

struct VisibleChunk { data: Chunk, meshes: Vec<Mesh> }

fn make_visible_world(generator: &TerrainGenerator) -> Vec<VisibleChunk> {
    let mut data = Vec::new();
    for z in -1..=1 {
        for x in -1..=1 {
            data.push(generator.generate_chunk(x, z));
        }
    }
    let snapshot = data.clone();
    data.into_iter().map(|chunk| {
        let meshes = build_chunk_meshes(&chunk, &snapshot, generator);
        VisibleChunk { data: chunk, meshes }
    }).collect()
}

fn refresh_affected_meshes(
    chunks: &mut [VisibleChunk],
    edited_index: usize,
    local_x: usize,
    local_z: usize,
    generator: &TerrainGenerator,
) {
    let snapshot: Vec<Chunk> = chunks.iter().map(|c| c.data.clone()).collect();
    let mut affected = vec![edited_index];
    let edited = &chunks[edited_index].data;
    let cx = edited.x;
    let cz = edited.z;

    // A neighbour only needs remeshing when the edit changes visibility
    // across that chunk boundary.
    let neighbour_coords = [
        (local_x == 0).then_some((cx - 1, cz)),
        (local_x + 1 == CHUNK_SIZE).then_some((cx + 1, cz)),
        (local_z == 0).then_some((cx, cz - 1)),
        (local_z + 1 == CHUNK_SIZE).then_some((cx, cz + 1)),
    ];
    for coord in neighbour_coords.into_iter().flatten() {
        if let Some(index) = chunks.iter().position(|c| (c.data.x, c.data.z) == coord) {
            if !affected.contains(&index) {
                affected.push(index);
            }
        }
    }

    for index in affected {
        let meshes = build_chunk_meshes(&snapshot[index], &snapshot, generator);
        chunks[index].meshes = meshes;
    }
}

fn chunk_index_at(chunks: &[VisibleChunk], wx: i32, wz: i32) -> Option<(usize, usize, usize)> {
    let cx = wx.div_euclid(CHUNK_SIZE as i32);
    let cz = wz.div_euclid(CHUNK_SIZE as i32);
    let index = chunks.iter().position(|c| c.data.x == cx && c.data.z == cz)?;
    let lx = wx.rem_euclid(CHUNK_SIZE as i32) as usize;
    let lz = wz.rem_euclid(CHUNK_SIZE as i32) as usize;
    Some((index, lx, lz))
}

fn block_at_world(chunks: &[VisibleChunk], wx: i32, wy: i32, wz: i32) -> BlockId {
    if wy < 0 || wy >= WORLD_HEIGHT as i32 { return BlockId::Air; }
    let Some((index, lx, lz)) = chunk_index_at(chunks, wx, wz) else { return BlockId::Air; };
    chunks[index].data.get(lx, wy as usize, lz)
}

fn raycast_world(origin: Vec3, direction: Vec3, chunks: &[VisibleChunk], reach: f32) -> Option<VoxelHit> {
    raycast_voxels(
        [origin.x, origin.y, origin.z],
        [direction.x, direction.y, direction.z],
        reach,
        |x, y, z| block_at_world(chunks, x, y, z),
    )
}

fn edit_world_block(
    chunks: &mut [VisibleChunk],
    hit: Option<VoxelHit>,
    selected: BlockId,
    place: bool,
    generator: &TerrainGenerator,
) {
    let Some(hit) = hit else { return; };
    let target = if place { hit.previous } else { hit.cell };
    if target[1] < 0 || target[1] >= WORLD_HEIGHT as i32 { return; }
    let Some((index, lx, lz)) = chunk_index_at(chunks, target[0], target[2]) else { return; };
    let current = chunks[index].data.get(lx, target[1] as usize, lz);

    let replacement = if place {
        if !current.is_replaceable() || !selected.is_solid() { return; }
        // Do not let a creative placement overwrite the protected world floor.
        if target[1] == 0 { return; }
        selected
    } else {
        if current == BlockId::Air || current == BlockId::Bedrock { return; }
        BlockId::Air
    };

    if chunks[index].data.set(lx, target[1] as usize, lz, replacement) {
        refresh_affected_meshes(chunks, index, lx, lz, generator);
    }
}

const BUILD_BLOCKS: [BlockId; 5] = [BlockId::Grass, BlockId::Dirt, BlockId::Stone, BlockId::Sand, BlockId::OakLog];
const BUILD_NAMES: [&str; 5] = ["GRASS", "DIRT", "STONE", "SAND", "OAK"];

#[macroquad::main("Poco Voxel Sandbox")]
async fn main() {
    let config = GameConfig::default();
    let generator = TerrainGenerator::new(WORLD_SEED);
    let mut chunks = make_visible_world(&generator);
    let mut selected_block = 0usize;

    let mut yaw = 0.78_f32;
    let mut pitch = 0.0_f32;
    let spawn_y = generator.column_info(0, 0).height as f32 + 2.0;
    let mut player_position = vec3(0.5, spawn_y, 0.5);
    let mut player_velocity = vec3(0.0, 0.0, 0.0);
    let mut on_ground = false;
    let mut distance = 52.0_f32;
    let mut save_status = String::from("WORLD: UNSAVED");
    let mut fly_mode = false;
    let mut fly_position = vec3(0.0, 58.0, 0.0);
    let mut fly_speed = 18.0_f32;
    let mut last_pointer = vec2(0.0, 0.0);
    let mut dragging = false;
    let mut show_help = true;
    // Independent touch IDs allow movement, looking and vertical flight at once.
    let mut move_touch: Option<u64> = None;
    let mut look_touch: Option<u64> = None;
    let mut look_last = vec2(0.0, 0.0);
    let mut move_axis = vec2(0.0, 0.0);
    // The joystick anchors where the left thumb first touches, avoiding a
    // forced reach to a fixed on-screen centre on differently sized phones.
    let mut move_origin = vec2(0.0, 0.0);
    let mut rise_held = false;
    let mut descend_held = false;
    simulate_mouse_with_touch(false);

    // A rolling sample makes short frame-time spikes visible instead of
    // relying only on the instantaneous FPS counter.
    let mut frame_times_ms: Vec<f32> = Vec::with_capacity(300);
    let mut frame_counter: u32 = 0;
    let mut one_percent_low_fps = 0.0_f32;
    let mut show_performance = true;

    loop {
        let raw_frame_time = get_frame_time();
        let dt = raw_frame_time.min(0.05);
        frame_times_ms.push(raw_frame_time * 1000.0);
        if frame_times_ms.len() > 300 {
            frame_times_ms.remove(0);
        }
        frame_counter = frame_counter.wrapping_add(1);
        if frame_counter % 30 == 0 && !frame_times_ms.is_empty() {
            let mut sorted = frame_times_ms.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let p99_index = ((sorted.len() as f32 * 0.99).ceil() as usize)
                .saturating_sub(1)
                .min(sorted.len() - 1);
            one_percent_low_fps = if sorted[p99_index] > 0.0 {
                1000.0 / sorted[p99_index]
            } else {
                0.0
            };
        }
        let average_fps = if frame_times_ms.is_empty() {
            0.0
        } else {
            1000.0 * frame_times_ms.len() as f32
                / frame_times_ms.iter().copied().sum::<f32>().max(0.001)
        };
        let frame_time_ms = raw_frame_time * 1000.0;
        let w = screen_width();
        let h = screen_height();
        let fly_rect = Rect::new(w - 112.0, 14.0, 98.0, 44.0);
        let perf_rect = Rect::new(w - 218.0, 14.0, 96.0, 44.0);
        let rise_rect = Rect::new(w - 104.0, h - 174.0, 76.0, 54.0);
        let descend_rect = Rect::new(w - 104.0, h - 108.0, 76.0, 54.0);
        let toolbar_y = h - 66.0;
        let block_rects: [Rect; 5] = std::array::from_fn(|i| Rect::new(12.0 + i as f32 * 76.0, toolbar_y, 70.0, 52.0));
        let break_rect = Rect::new(w - 178.0, toolbar_y, 78.0, 52.0);
        let place_rect = Rect::new(w - 92.0, toolbar_y, 80.0, 52.0);
        let jump_rect = Rect::new(w - 184.0, h - 132.0, 78.0, 54.0);
        let sprint_rect = Rect::new(w - 96.0, h - 132.0, 82.0, 54.0);
        let save_rect = Rect::new(w - 218.0, 66.0, 96.0, 38.0);
        let load_rect = Rect::new(w - 112.0, 66.0, 100.0, 38.0);
        let mut break_requested = false;
        let mut place_requested = false;
        let mut jump_requested = false;
        let mut sprint_held = false;
        let mut save_requested = false;
        let mut load_requested = false;

        // Native multi-touch: left thumb moves, right thumb looks, right-side
        // buttons control vertical flight. Touches are independent.
        let active_touches = touches();
        rise_held = false;
        descend_held = false;
        for touch in &active_touches {
            let p = touch.position;
            if touch.phase == TouchPhase::Started {
                if let Some(i) = block_rects.iter().position(|r| r.contains(p)) {
                    selected_block = i;
                } else if break_rect.contains(p) {
                    break_requested = true;
                } else if place_rect.contains(p) {
                    place_requested = true;
                } else if save_rect.contains(p) {
                    save_requested = true;
                } else if load_rect.contains(p) {
                    load_requested = true;
                } else if perf_rect.contains(p) {
                    show_performance = !show_performance;
                } else if fly_rect.contains(p) {
                    fly_mode = !fly_mode;
                } else if !fly_mode && jump_rect.contains(p) {
                    jump_requested = true;
                } else if fly_mode && rise_rect.contains(p) {
                    // The button is also handled as a held control below.
                } else if fly_mode && descend_rect.contains(p) {
                    // The button is also handled as a held control below.
                } else if p.x < w * 0.46 && p.y > h * 0.42 && move_touch.is_none() {
                    move_touch = Some(touch.id);
                    move_origin = p;
                } else if p.x >= w * 0.46 && look_touch.is_none() {
                    look_touch = Some(touch.id);
                    look_last = p;
                }
            }
            if Some(touch.id) == move_touch {
                let delta = p - move_origin;
                let radius = 58.0;
                let len = delta.length();
                move_axis = if len > radius { delta / len } else { delta / radius };
            }
            if Some(touch.id) == look_touch {
                if touch.phase == TouchPhase::Moved || touch.phase == TouchPhase::Stationary {
                    let delta = p - look_last;
                    if touch.phase == TouchPhase::Moved {
                        yaw -= delta.x * 0.006;
                        pitch = (pitch - delta.y * 0.005).clamp(-1.35, 1.35);
                    }
                    look_last = p;
                }
            }
            if fly_mode && rise_rect.contains(p) { rise_held = true; }
            if fly_mode && descend_rect.contains(p) { descend_held = true; }
            if !fly_mode && sprint_rect.contains(p) { sprint_held = true; }
        }
        if !active_touches.iter().any(|t| Some(t.id) == move_touch) {
            move_touch = None;
            move_axis = vec2(0.0, 0.0);
        }
        if !active_touches.iter().any(|t| Some(t.id) == look_touch) { look_touch = None; }

        // Desktop mouse fallback. Android uses the native multi-touch path above.
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            last_pointer = vec2(mx, my);
            dragging = true;
        }
        if is_mouse_button_released(MouseButton::Left) { dragging = false; }
        if dragging && is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let now = vec2(mx, my);
            let delta = now - last_pointer;
            yaw -= delta.x * 0.006;
            pitch = (pitch - delta.y * 0.005).clamp(-1.35, 1.35);
            last_pointer = now;
        }
        let (_, wheel_y) = mouse_wheel();
        distance = (distance - wheel_y * 2.5).clamp(12.0, 115.0);
        if is_key_pressed(KeyCode::H) { show_help = !show_help; }
        if is_key_pressed(KeyCode::F3) { show_performance = !show_performance; }
        if is_key_pressed(KeyCode::F) { fly_mode = !fly_mode; player_velocity = Vec3::ZERO; }
        if is_key_pressed(KeyCode::F5) || save_requested {
            let data: Vec<Chunk> = chunks.iter().map(|c| c.data.clone()).collect();
            save_status = match save::save_world("poco_world.json", WORLD_SEED, &data) {
                Ok(()) => String::from("WORLD SAVED (F5)"),
                Err(_) => String::from("SAVE FAILED (F5)"),
            };
        }
        if is_key_pressed(KeyCode::F9) || load_requested {
            match save::load_world("poco_world.json") {
                Ok((seed, loaded)) if seed == WORLD_SEED => {
                    let snapshot = loaded.clone();
                    chunks = loaded.into_iter().map(|data| {
                        let meshes = build_chunk_meshes(&data, &snapshot, &generator);
                        VisibleChunk { data, meshes }
                    }).collect();
                    save_status = String::from("WORLD LOADED (F9)");
                }
                Ok(_) => save_status = String::from("SAVE WORLD SEED MISMATCH"),
                Err(_) => save_status = String::from("LOAD FAILED - NO VALID SAVE"),
            }
        }
        if is_key_pressed(KeyCode::Key1) { selected_block = 0; }
        if is_key_pressed(KeyCode::Key2) { selected_block = 1; }
        if is_key_pressed(KeyCode::Key3) { selected_block = 2; }
        if is_key_pressed(KeyCode::Key4) { selected_block = 3; }
        if is_key_pressed(KeyCode::Key5) { selected_block = 4; }
        if is_key_pressed(KeyCode::Q) { break_requested = true; }
        if is_key_pressed(KeyCode::E) { place_requested = true; }
        if is_mouse_button_pressed(MouseButton::Right) { break_requested = true; }
        if is_mouse_button_pressed(MouseButton::Middle) { place_requested = true; }
        let mut forward_input = if is_key_down(KeyCode::W) { 1.0 } else { 0.0 };
        forward_input -= if is_key_down(KeyCode::S) { 1.0 } else { 0.0 };
        let mut strafe_input = if is_key_down(KeyCode::D) { 1.0 } else { 0.0 };
        strafe_input -= if is_key_down(KeyCode::A) { 1.0 } else { 0.0 };
        forward_input += -move_axis.y;
        strafe_input += move_axis.x;
        let forward_dir = vec3(yaw.sin(), 0.0, yaw.cos());
        let right_dir = vec3(yaw.cos(), 0.0, -yaw.sin());
        let mut wish = forward_dir * forward_input + right_dir * strafe_input;
        if wish.length_squared() > 1.0 { wish = wish.normalize(); }
        if !fly_mode {
            let sprinting = is_key_down(KeyCode::LeftShift) || sprint_held;
            let speed = if sprinting { 8.0 } else { 4.5 };
            let step = wish * speed * dt;
            // Resolve horizontal axes independently to avoid tunnelling through corners.
            for axis in 0..2 {
                let candidate = if axis == 0 { vec3(player_position.x + step.x, player_position.y, player_position.z) }
                    else { vec3(player_position.x, player_position.y, player_position.z + step.z) };
                let min_x = (candidate.x - 0.30).floor() as i32;
                let max_x = (candidate.x + 0.30).floor() as i32;
                let min_z = (candidate.z - 0.30).floor() as i32;
                let max_z = (candidate.z + 0.30).floor() as i32;
                let min_y = (candidate.y).floor() as i32;
                let max_y = (candidate.y + 1.75).floor() as i32;
                let mut blocked = false;
                'collision: for bx in min_x..=max_x { for bz in min_z..=max_z { for by in min_y..=max_y {
                    if block_at_world(&chunks, bx, by, bz).is_solid() { blocked = true; break 'collision; }
                }}};
                if !blocked { player_position = candidate; }
            }
            player_velocity.y -= 22.0 * dt;
            if on_ground && (is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Up) || jump_requested) {
                player_velocity.y = 8.0; on_ground = false;
            }
            let mut next_y = player_position.y + player_velocity.y * dt;
            if player_velocity.y > 0.0 {
                let head_y = (next_y + 1.72).floor() as i32;
                let mut hit_ceiling = false;
                for bx in (player_position.x - 0.28).floor() as i32..=(player_position.x + 0.28).floor() as i32 {
                    for bz in (player_position.z - 0.28).floor() as i32..=(player_position.z + 0.28).floor() as i32 {
                        if block_at_world(&chunks, bx, head_y, bz).is_solid() { hit_ceiling = true; }
                    }
                }
                if hit_ceiling { next_y = head_y as f32 - 1.75; player_velocity.y = 0.0; }
            }
            let feet_y = (next_y - 0.05).floor() as i32;
            let mut grounded = false;
            for bx in (player_position.x - 0.28).floor() as i32..=(player_position.x + 0.28).floor() as i32 {
                for bz in (player_position.z - 0.28).floor() as i32..=(player_position.z + 0.28).floor() as i32 {
                    if block_at_world(&chunks, bx, feet_y, bz).is_solid() { grounded = true; }
                }
            }
            if grounded && player_velocity.y <= 0.0 { player_position.y = feet_y as f32 + 1.0; player_velocity.y = 0.0; on_ground = true; }
            else { player_position.y = next_y; on_ground = false; }
            player_position.y = player_position.y.max(1.0).min(WORLD_HEIGHT as f32 - 2.0);
        }
        if fly_mode {
            let mut forward = if is_key_down(KeyCode::W) { 1.0 } else { 0.0 };
            forward -= if is_key_down(KeyCode::S) { 1.0 } else { 0.0 };
            let mut strafe = if is_key_down(KeyCode::D) { 1.0 } else { 0.0 };
            strafe -= if is_key_down(KeyCode::A) { 1.0 } else { 0.0 };
            forward += -move_axis.y;
            strafe += move_axis.x;
            let forward_dir = vec3(yaw.sin(), 0.0, yaw.cos());
            let right_dir = vec3(yaw.cos(), 0.0, -yaw.sin());
            let move_vec = forward_dir * forward + right_dir * strafe;
            if move_vec.length_squared() > 1.0 { fly_position += move_vec.normalize() * fly_speed * dt; }
            else { fly_position += move_vec * fly_speed * dt; }
            if is_key_down(KeyCode::Space) || rise_held { fly_position.y += fly_speed * dt; }
            if is_key_down(KeyCode::LeftShift) || descend_held { fly_position.y -= fly_speed * dt; }
            if is_key_down(KeyCode::LeftControl) { fly_speed = 36.0; } else { fly_speed = 18.0; }
        }

        clear_background(Color::from_rgba(126, 174, 207, 255));
        let camera_position = if fly_mode { fly_position } else { player_position + vec3(0.0, 1.62, 0.0) };
        let view_direction = vec3(yaw.sin() * pitch.cos(), pitch.sin(), yaw.cos() * pitch.cos()).normalize();
        let target = camera_position + view_direction * 10.0;
        let aimed_hit = raycast_world(camera_position, view_direction, &chunks, 72.0);
        if break_requested {
            edit_world_block(&mut chunks, aimed_hit, BUILD_BLOCKS[selected_block], false, &generator);
        } else if place_requested {
            edit_world_block(&mut chunks, aimed_hit, BUILD_BLOCKS[selected_block], true, &generator);
        }

        set_camera(&Camera3D {
            position: camera_position,
            target,
            up: vec3(0.0, 1.0, 0.0),
            fovy: 55.0,
            z_near: 0.1,
            z_far: 500.0,
            ..Default::default()
        });
        for chunk in &chunks {
            draw_chunk_meshes(&chunk.meshes);
        }
        set_default_camera();

        // Centre reticle marks the exact voxel currently targeted for editing.
        let cx = w * 0.5;
        let cy = h * 0.5;
        draw_line(cx - 9.0, cy, cx + 9.0, cy, 2.0, WHITE);
        draw_line(cx, cy - 9.0, cx, cy + 9.0, 2.0, WHITE);
        let fps = get_fps();
        draw_rectangle(perf_rect.x, perf_rect.y, perf_rect.w, perf_rect.h, Color::from_rgba(20,30,40,210));
        draw_text(if show_performance { "PERF: ON" } else { "PERF: OFF" }, perf_rect.x + 8.0, 42.0, 15.0, WHITE);
        draw_rectangle(w - 112.0, 14.0, 98.0, 44.0, if fly_mode { Color::from_rgba(35,120,75,220) } else { Color::from_rgba(20,30,40,200) });
        draw_rectangle(save_rect.x, save_rect.y, save_rect.w, save_rect.h, Color::from_rgba(30, 90, 65, 220));
        draw_rectangle(load_rect.x, load_rect.y, load_rect.w, load_rect.h, Color::from_rgba(55, 75, 115, 220));
        draw_text("SAVE", save_rect.x + 27.0, save_rect.y + 25.0, 16.0, WHITE);
        draw_text("LOAD", load_rect.x + 29.0, load_rect.y + 25.0, 16.0, WHITE);
        draw_text(if fly_mode { "FLY: ON" } else { "FLY: OFF" }, w - 102.0, 42.0, 17.0, WHITE);
        {
            // Thumb-anchored joystick: its base follows the initial touch point.
            let stick_center = if move_touch.is_some() { move_origin } else { vec2(w.min(900.0) * 0.105, h - 112.0) };
            draw_circle(stick_center.x, stick_center.y, 58.0, Color::from_rgba(12,20,28,105));
            draw_circle_lines(stick_center.x, stick_center.y, 58.0, 2.0, Color::from_rgba(225,235,245,180));
            let knob = stick_center + move_axis * 38.0;
            draw_circle(knob.x, knob.y, 23.0, Color::from_rgba(190,210,225,190));
            if fly_mode {
            draw_rectangle(rise_rect.x, rise_rect.y, rise_rect.w, rise_rect.h, if rise_held { Color::from_rgba(55,135,95,230) } else { Color::from_rgba(20,30,40,200) });
            draw_rectangle(descend_rect.x, descend_rect.y, descend_rect.w, descend_rect.h, if descend_held { Color::from_rgba(55,135,95,230) } else { Color::from_rgba(20,30,40,200) });
            draw_text("UP", rise_rect.x + 24.0, rise_rect.y + 34.0, 19.0, WHITE);
            draw_text("DOWN", descend_rect.x + 12.0, descend_rect.y + 34.0, 17.0, WHITE);
            }
        }
        if !fly_mode {
            draw_rectangle(jump_rect.x, jump_rect.y, jump_rect.w, jump_rect.h, Color::from_rgba(35, 95, 145, 210));
            draw_rectangle(sprint_rect.x, sprint_rect.y, sprint_rect.w, sprint_rect.h, if sprint_held { Color::from_rgba(55, 135, 95, 235) } else { Color::from_rgba(20, 30, 40, 200) });
            draw_text("JUMP", jump_rect.x + 15.0, jump_rect.y + 33.0, 17.0, WHITE);
            draw_text("RUN", sprint_rect.x + 22.0, sprint_rect.y + 33.0, 17.0, WHITE);
        }
        // Simple creative toolbar: choose a block, then break or place the aimed surface block.
        for i in 0..BUILD_BLOCKS.len() {
            let r = block_rects[i];
            let fill = if i == selected_block { Color::from_rgba(66, 116, 160, 245) } else { Color::from_rgba(20, 30, 40, 220) };
            draw_rectangle(r.x, r.y, r.w, r.h, fill);
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, if i == selected_block { YELLOW } else { GRAY });
            draw_text(BUILD_NAMES[i], r.x + 5.0, r.y + 31.0, 14.0, WHITE);
        }
        draw_rectangle(break_rect.x, break_rect.y, break_rect.w, break_rect.h, Color::from_rgba(130, 55, 48, 235));
        draw_rectangle(place_rect.x, place_rect.y, place_rect.w, place_rect.h, Color::from_rgba(45, 115, 76, 235));
        draw_text("BREAK", break_rect.x + 8.0, break_rect.y + 31.0, 16.0, WHITE);
        draw_text("PLACE", place_rect.x + 10.0, place_rect.y + 31.0, 16.0, WHITE);

        if show_performance {
            let mesh_count: usize = chunks.iter().map(|c| c.meshes.len()).sum();
            let vertex_count: usize = chunks.iter()
                .flat_map(|c| c.meshes.iter())
                .map(|mesh| mesh.vertices.len())
                .sum();
            draw_rectangle(12.0, 12.0, 318.0, 132.0, Color::from_rgba(12, 18, 25, 190));
            draw_text("POCO VOXEL SANDBOX  |  F3: PERF", 22.0, 32.0, 17.0, WHITE);
            draw_text(&format!("FPS {fps}  |  AVG {:.1}  |  1% LOW {:.1}", average_fps, one_percent_low_fps), 22.0, 54.0, 15.0, WHITE);
            draw_text(&format!("FRAME {:.2} ms  |  CHUNKS {}  |  MESHES {}", frame_time_ms, chunks.len(), mesh_count), 22.0, 76.0, 14.0, LIGHTGRAY);
            draw_text(&format!("MESH VERTICES {}  |  FLY {}", vertex_count, if fly_mode {"ON"} else {"OFF"}), 22.0, 98.0, 14.0, LIGHTGRAY);
            draw_text(&save_status, 22.0, 118.0, 13.0, YELLOW);
        }
        if show_help {
            let panel_y = screen_height() - 138.0;
            draw_rectangle(12.0, panel_y, screen_width() - 24.0, 58.0, Color::from_rgba(12, 18, 25, 175));
            draw_text("WASD / left thumb move | mouse / right thumb look | SPACE jump", 22.0, panel_y + 23.0, 15.0, WHITE);
            draw_text("SHIFT sprint | F toggle flight | F5 save | F9 load | H help", 22.0, panel_y + 45.0, 14.0, LIGHTGRAY);
        }
        let _ = &config;
        next_frame().await;
    }
}
