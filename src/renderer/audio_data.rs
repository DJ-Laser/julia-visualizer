use std::time::Duration;

use wgpu::util::DeviceExt;

pub struct AudioData {
  spectrum_buffer: wgpu::Buffer,
  waveform_buffer: wgpu::Buffer,
  audio_data_bind_group: wgpu::BindGroup,
  audio_data_bind_group_layout: wgpu::BindGroupLayout,
}

impl AudioData {
  pub async fn new(device: &wgpu::Device) -> Self {
    let audio_data_bind_group_layout =
      device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
          wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
              ty: wgpu::BufferBindingType::Uniform,
              has_dynamic_offset: false,
              min_binding_size: None,
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
        label: Some("fragment_bind_group_layout"),
      });

    let spectrum_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Spectrum Buffer"),
      contents: bytemuck::cast_slice(&[0.0f32]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let waveform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Waveform Buffer"),
      contents: bytemuck::cast_slice(&[0.0f32, 0.0f32]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let audio_data_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &audio_data_bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::Buffer(spectrum_buffer.as_entire_buffer_binding()),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Buffer(waveform_buffer.as_entire_buffer_binding()),
        },
      ],
      label: Some("fragment_bind_group"),
    });

    Self {
      spectrum_buffer,
      waveform_buffer,
      audio_data_bind_group,
      audio_data_bind_group_layout,
    }
  }

  pub fn layout(&self) -> &wgpu::BindGroupLayout {
    &self.audio_data_bind_group_layout
  }

  pub fn update_time(&self, new_time: Duration, queue: &wgpu::Queue) {
    queue.write_buffer(
      &self.spectrum_buffer,
      0,
      bytemuck::cast_slice(&[new_time.as_secs_f32()]),
    );
  }

  pub fn update_resolution(
    &self,
    new_resolution: winit::dpi::PhysicalSize<f32>,
    queue: &wgpu::Queue,
  ) {
    queue.write_buffer(
      &self.waveform_buffer,
      0,
      bytemuck::cast_slice(&[new_resolution.width, new_resolution.height]),
    );
  }
}

pub trait BindExtraInfo<'a> {
  fn bind_extra_info(&mut self, extra_info: &'a AudioData);
}
impl<'a, 'b> BindExtraInfo<'b> for wgpu::RenderPass<'a>
where
  'b: 'a,
{
  fn bind_extra_info(&mut self, extra_info: &'b AudioData) {
    self.set_bind_group(0, &extra_info.audio_data_bind_group, &[]);
  }
}
