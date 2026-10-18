//! The DomeInfo frame filter (card BFLT, `analysis/re-glow.md` §3 `[O]`):
//! both games lift everything brighter than a threshold over the finished
//! 3D frame, before the HUD — the whole-frame brightening every BF1 map
//! (and BF2 TAT2) shows on the PS2.
//!
//! What the PS2 does (B1 `FUN_0049fe60`, B2 `FUN_00585890`): nothing while
//! `Intensity` < 1; else the frame is boxed 2:1 to 256×224, `Threshold` is
//! subtracted (GS ALPHA 0x62, clamped at 0), two sub-pixel blur steps, and
//! `Intensity`/128 of the result is added back over the frame (ALPHA 0x68)
//! — per channel, in the GS's display bytes. Measured on the PS2 (§3.3):
//! `out = min(255, v + I/128 · up2(max(0, box2(v) − T)))` predicts the
//! filtered frame within mean |d| 1.6–3.9.
//!
//! The port runs it at native resolution: the low-pass is a box of
//! `height/224` pixels centred on each pixel ([`box_side`]). Like the
//! terrain light pass (`terrain_light.rs`) it reads a COPY of the frame,
//! encodes to display units by hand, and writes the colour over the target
//! (`FRAME_WRITES`: the alpha plane stays as the scene left it). A window whose surface cannot be copied draws no filter
//! (said once on stderr).

use crate::Renderer;

/// A map's filter as authored (`manifest::ManifestFilter`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameFilter {
    /// `Threshold`, a display byte 0..255.
    pub threshold: f32,
    /// `Intensity`; the gain is `intensity / 128`.
    pub intensity: f32,
}

/// The widest box the pass averages (a 2688-line target).
const MAX_BOX: u32 = 12;

/// The low-pass box's side in output pixels for a `height`-line target:
/// one PS2 half-scale pixel (448 display lines / 224) at this resolution.
pub fn box_side(height: u32) -> u32 {
    ((height as f32 / 224.0).round() as u32).clamp(1, MAX_BOX)
}

/// The shader's `FilterParams`.
fn params(f: &FrameFilter, height: u32) -> [f32; 4] {
    [
        f.threshold / 255.0,
        f.intensity / 128.0,
        box_side(height) as f32,
        0.0,
    ]
}

/// The pass's per-renderer state.
pub(crate) struct FilterState {
    pub filter: Option<FrameFilter>,
    pipeline: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
    params_buf: wgpu::Buffer,
    /// The frame's copy the shader reads, its size, and the bind group.
    dest: Option<(wgpu::Texture, wgpu::BindGroup, u32, u32)>,
    copyable: bool,
    warned: bool,
}

impl FilterState {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat, copyable: bool) -> Self {
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("frame filter bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("frame filter layout"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("frame filter shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("filter.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("frame filter pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_filter"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_filter"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: crate::pipelines::FRAME_WRITES,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let params_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("frame filter params"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            filter: None,
            pipeline,
            bgl,
            params_buf,
            dest: None,
            copyable,
            warned: false,
        }
    }
}

impl Renderer {
    /// The map's frame filter, or `None` for no pass (card BFLT).
    pub fn set_frame_filter(&mut self, filter: Option<FrameFilter>) {
        self.filter.filter = filter;
    }

    /// Whether this frame runs the filter: one set with a gain, and a
    /// target it can copy.
    pub(crate) fn filter_wanted(&mut self) -> bool {
        let f = &mut self.filter;
        let want = f.filter.is_some_and(|p| p.intensity >= 1.0);
        if want && !f.copyable {
            if !f.warned {
                eprintln!("frame filter: the window surface cannot be copied; no filter pass");
                f.warned = true;
            }
            return false;
        }
        want
    }

    /// Upload the parameters and (re)build the copy for a `w`×`h` target.
    pub(crate) fn prepare_filter(&mut self, w: u32, h: u32) {
        let f = &mut self.filter;
        let Some(p) = f.filter else { return };
        self.queue
            .write_buffer(&f.params_buf, 0, bytemuck::cast_slice(&params(&p, h)));
        if matches!(&f.dest, Some((_, _, dw, dh)) if (*dw, *dh) == (w, h)) {
            return;
        }
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("frame filter source"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("frame filter bg"),
            layout: &f.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: f.params_buf.as_entire_binding(),
                },
            ],
        });
        f.dest = Some((tex, bg, w, h));
    }

    /// Copy `src` (the finished 3D frame) and draw the filter over `view`.
    pub(crate) fn filter_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        src: &wgpu::Texture,
        view: &wgpu::TextureView,
    ) -> u32 {
        let f = &self.filter;
        let Some((dest, bg, w, h)) = &f.dest else {
            return 0;
        };
        encoder.copy_texture_to_texture(
            src.as_image_copy(),
            dest.as_image_copy(),
            wgpu::Extent3d {
                width: *w,
                height: *h,
                depth_or_array_layers: 1,
            },
        );
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("frame filter pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&f.pipeline);
        pass.set_bind_group(0, bg, &[]);
        pass.draw(0..3, 0..1);
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One PS2 half-scale pixel: 2 at the PS2's 448 lines, 3 at 720,
    /// 5 at 1080, capped.
    #[test]
    fn the_box_is_one_ps2_half_pixel() {
        assert_eq!(box_side(448), 2);
        assert_eq!(box_side(720), 3);
        assert_eq!(box_side(1080), 5);
        assert_eq!(box_side(100), 1);
        assert_eq!(box_side(8000), MAX_BOX);
    }

    /// TAT1's 50/50: threshold 50/255, gain 50/128.
    #[test]
    fn params_are_display_units() {
        let p = params(
            &FrameFilter {
                threshold: 50.0,
                intensity: 50.0,
            },
            720,
        );
        assert_eq!(p, [50.0 / 255.0, 50.0 / 128.0, 3.0, 0.0]);
    }
}
