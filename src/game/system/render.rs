use crate::prelude::*;
use hecs;
use crate::game::component;
use crate::game::Infrastructure;
use crate::bloom;
use std::cmp;

/// A filtered render pass on a layer (created by the Itsy script via
/// `add_render_layer`), mirroring the old layer.yaml "render" section.
#[derive(Clone, Copy, Debug)]
pub struct RenderLayer {
    pub layer_id  : u32,
    pub filter    : Option<RenderFilter>,
    pub component : u32,
}

#[derive(Clone, Copy, Debug)]
pub enum RenderFilter {
    Bloom,
    Glare,
}

/// A background image draw requested by the Itsy script (`draw_background`),
/// resolved to a loaded texture. Drawn below all render layers; the image is
/// scaled to cover the display and tiled (wrapped) around the given scroll
/// offset for seamless infinite scrolling.
#[derive(Clone)]
pub struct RenderBackground {
    pub texture : Arc<Texture>,
    /// Scroll offset in screen pixels (any value; wrapped to the image size).
    pub offset_x: f32,
    pub offset_y: f32,
}

pub struct Render {
    fps_interval: Periodic,
    num_frames: u32,
    last_num_frames: u32,
    bloom: postprocessors::Bloom,
    glare: bloom::Bloom,
}

impl Render {
    pub fn new(radiant_ctx: Context, dimensions: (u32, u32)) -> Self {

        let mut bloom = postprocessors::Bloom::new(&radiant_ctx, dimensions, 2);
        bloom.clear = false;
        bloom.draw_color = Color::alpha_pm(0.15);

        Render {
            fps_interval: Periodic::new(0.0, 1.0),
            num_frames: 0,
            last_num_frames: 0,
            bloom           : bloom,
            glare           : bloom::Bloom::new(&radiant_ctx, dimensions, 2, 5, 5.0),
        }
    }

    /// Rebuild the postprocessor render targets for a new display size
    /// (display resize).
    pub fn resize(&mut self, radiant_ctx: &Context, dimensions: (u32, u32)) {
        self.bloom.rebuild(radiant_ctx, dimensions, 2);
        self.glare.rebuild(radiant_ctx, dimensions);
    }

    pub fn run(&mut self, world: &mut hecs::World, age: f32, delta: f32, inf: &Infrastructure, renderer: &Renderer) {
        let mut num_sprites = 0;

        for (_entity, (spatial, visual, fading)) in world.query_mut::<(
            &component::Spatial,
            &mut component::Visual,
            Option<&component::Fading>,
        )>() {
            if let Some(fading) = fading {
                if age >= fading.start {
                    let duration = fading.end - fading.start;
                    let progress = age - fading.start;
                    let alpha = 1.0 - (progress / duration);
                    if alpha >= 0.0 {
                        visual.color.set_a(alpha);
                        visual.effect_color.set_a(alpha);
                    }
                }
            }

            if let Some(ref layer) = visual.layer {
                visual.sprite.draw_transformed(
                    &layer, visual.frame_id as u32,
                    spatial.position, visual.color.to_pm(),
                    spatial.angle.to_radians(), (visual.scale, visual.scale)
                );
            }

            if let Some(ref effect_layer) = visual.effect_layer {
                visual.sprite.draw_transformed(
                    &effect_layer, visual.frame_id as u32,
                    spatial.position, visual.effect_color.to_pm(),
                    spatial.angle.to_radians(), (visual.effect_scale, visual.effect_scale)
                );
            }

            visual.frame_id = if visual.fps == 0 {
                cmp::min(29, cmp::max(0, (15.0 + (15.0 * spatial.lean)) as i32)) as f32
            } else {
                visual.frame_id + delta * visual.fps as f32
            };

            num_sprites += 1;
        }

        self.num_frames += 1;

        if self.fps_interval.elapsed(age) {
            self.last_num_frames = self.num_frames;
            self.num_frames = 0;
        }

        if let Some(layer) = inf.debug_layer() {
            inf.font.write(&layer, &format!("Entities: {:?}", num_sprites), (10.0, 72.0), Color::alpha_pm(0.4));
        }

        // Backgrounds requested by the Itsy script (draw_background):
        // tiled below all render layers, wrapped for infinite scrolling.
        let (display_w, display_h) = inf.display.dimensions();
        for draw in inf.background_draws.iter() {
            Self::draw_background_tiled(renderer, display_w as f32, display_h as f32, draw);
        }

        // render layers (passes created by the Itsy script)
        for info in inf.render_layers.iter() {
            if let Some(layer) = inf.layers.get(info.layer_id as usize) {
                match info.filter {
                    Some(RenderFilter::Bloom) => {
                        renderer.postprocess(&self.bloom, &(), || {
                            renderer.fill().color(Color::alpha_mask(0.3)).draw();
                            renderer.draw_layer(layer, info.component);
                        });
                    }
                    Some(RenderFilter::Glare) => {
                        renderer.postprocess(&self.glare, &blendmodes::SCREEN, || {
                            renderer.fill().color(Color::alpha_mask(0.05)).draw();
                            renderer.draw_layer(layer, info.component);
                        });
                    }
                    None => {
                        renderer.draw_layer(layer, info.component);
                    }
                }
            } else {
                eprintln!("render_layers: invalid layer id {}", info.layer_id);
            }
        }

        for layer in inf.layers.iter() {
            layer.clear();
        }
    }

    /// Draws `draw` tiled so it covers the entire display. The image is scaled to
    /// cover the display (aspect preserved), then repeated in both directions with
    /// the scroll offset wrapped, so any offset scrolls seamlessly.
    fn draw_background_tiled(renderer: &Renderer, display_w: f32, display_h: f32, draw: &RenderBackground) {
        let (tw, th) = draw.texture.dimensions();
        for (x, y, w, h) in Self::background_tiles(display_w, display_h, tw as f32, th as f32, draw.offset_x, draw.offset_y) {
            renderer.rect(((x, y), (w, h))).texture(&draw.texture).blendmode(blendmodes::COPY).draw();
        }
    }

    /// Pixel rectangles of the image tiles covering a `display_w` x `display_h`
    /// area: the image (scaled to cover the display, aspect preserved) repeated in
    /// both directions, shifted by the (wrapped) scroll offset.
    fn background_tiles(display_w: f32, display_h: f32, img_w: f32, img_h: f32, offset_x: f32, offset_y: f32) -> Vec<(f32, f32, f32, f32)> {
        let scale = (display_w / img_w).max(display_h / img_h);
        let iw = img_w * scale;
        let ih = img_h * scale;
        let ox = Self::wrap_pos(offset_x, iw);
        let oy = Self::wrap_pos(offset_y, ih);
        // The cover scale guarantees iw >= display_w and ih >= display_h, so at
        // most 2 tiles per axis are needed: a second one exists only when the
        // offset pushes the first tile's right/bottom edge past the display.
        // Tile k sits at k * size - offset.
        let kx_max = if ox + display_w > iw { 1 } else { 0 };
        let ky_max = if oy + display_h > ih { 1 } else { 0 };
        let mut tiles = Vec::new();
        for ky in 0..=ky_max {
            let y = ky as f32 * ih - oy;
            for kx in 0..=kx_max {
                let x = kx as f32 * iw - ox;
                tiles.push((x, y, iw, ih));
            }
        }
        tiles
    }

    /// Wraps `v` into [0, period) (Rust's `%` may return negative values).
    fn wrap_pos(v: f32, period: f32) -> f32 {
        let w = v % period;
        if w < 0.0 { w + period } else { w }
    }
}

#[cfg(test)]
mod tests {
    use super::Render;

    /// Checks that every tile is fully contained in a plausible region and
    /// that the union of tiles covers the whole display.
    fn assert_covers(tiles: &[(f32, f32, f32, f32)], display_w: f32, display_h: f32) {
        assert!(!tiles.is_empty());
        // Cover in x: [min_start, max_end] must contain [0, display_w].
        let min_x = tiles.iter().map(|t| t.0).fold(f32::MAX, f32::min);
        let max_x = tiles.iter().map(|t| t.0 + t.2).fold(f32::MIN, f32::max);
        let min_y = tiles.iter().map(|t| t.1).fold(f32::MAX, f32::min);
        let max_y = tiles.iter().map(|t| t.1 + t.3).fold(f32::MIN, f32::max);
        assert!(min_x <= 0.0, "gap at left edge: min_x = {}", min_x);
        assert!(max_x >= display_w, "gap at right edge: max_x = {}", max_x);
        assert!(min_y <= 0.0, "gap at top edge: min_y = {}", min_y);
        assert!(max_y >= display_h, "gap at bottom edge: max_y = {}", max_y);
    }

    #[test]
    fn wrap_pos_wraps_into_range() {
        assert!((Render::wrap_pos(0.0, 10.0) - 0.0).abs() < 1e-6);
        assert!((Render::wrap_pos(10.0, 10.0) - 0.0).abs() < 1e-6);
        assert!((Render::wrap_pos(25.5, 10.0) - 5.5).abs() < 1e-6);
        assert!((Render::wrap_pos(-5.0, 10.0) - 5.0).abs() < 1e-6);
        assert!((Render::wrap_pos(-15.25, 10.0) - 4.75).abs() < 1e-6);
    }

    #[test]
    fn tiles_cover_display_at_offset_zero() {
        // blue.jpg: 3200x1000 on a 1920x1080 display.
        let tiles = Render::background_tiles(1920.0, 1080.0, 3200.0, 1000.0, 0.0, 0.0);
        assert_covers(&tiles, 1920.0, 1080.0);
        // Cover scale = 1080/1000 = 1.08 -> 3456x1080: exactly one tile.
        assert_eq!(tiles.len(), 1);
        assert!((tiles[0].0 - 0.0).abs() < 1e-3 && (tiles[0].1 - 0.0).abs() < 1e-3);
        assert!((tiles[0].2 - 3456.0).abs() < 0.01 && (tiles[0].3 - 1080.0).abs() < 0.01);
    }

    #[test]
    fn tiles_cover_display_when_scrolled() {
        // A mid-scroll offset needs a second, shifted tile pair.
        let tiles = Render::background_tiles(1920.0, 1080.0, 3200.0, 1000.0, 1500.0, 200.0);
        assert_covers(&tiles, 1920.0, 1080.0);
        assert!(tiles.len() <= 4);
    }

    #[test]
    fn tiles_wrap_unbounded_offsets() {
        // Any offset (negative, unbounded) must give the same tiling as its
        // wrapped equivalent: seamless infinite scrolling.
        let a = Render::background_tiles(1920.0, 1080.0, 3200.0, 1000.0, -42.5, 0.0);
        let b = Render::background_tiles(1920.0, 1080.0, 3200.0, 1000.0, 3456.0 - 42.5, 0.0);
        assert_eq!(a.len(), b.len());
        for (ta, tb) in a.iter().zip(b.iter()) {
            assert!((ta.0 - tb.0).abs() < 1e-3 && (ta.1 - tb.1).abs() < 1e-3);
        }
        let c = Render::background_tiles(1920.0, 1080.0, 3200.0, 1000.0, 100_000.0, -7.0);
        assert_covers(&c, 1920.0, 1080.0);
    }

    #[test]
    fn tiles_cover_display_for_small_images() {
        // An image smaller than the display (in one axis) still covers it:
        // square 100x100 -> scaled to 1920x1920 (cover), 2 rows needed.
        let tiles = Render::background_tiles(1920.0, 1080.0, 100.0, 100.0, 300.0, 600.0);
        assert_covers(&tiles, 1920.0, 1080.0);
        assert!(tiles.len() <= 4);
    }
}
