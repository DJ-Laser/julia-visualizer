use std::mem;

pub struct AudioData {
  spectrum_buffer: wgpu::Buffer,
  waveform_buffer: wgpu::Buffer,
  audio_data_bind_group: wgpu::BindGroup,
  audio_data_bind_group_layout: wgpu::BindGroupLayout,
}

impl AudioData {
  fn create_buffers(device: &wgpu::Device, size: usize) -> (wgpu::Buffer, wgpu::Buffer) {
    let spectrum_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Spectrum Buffer"),
      size: (size * mem::size_of::<f32>()) as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let waveform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Waveform Buffer"),
      size: (size * mem::size_of::<f32>()) as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    (spectrum_buffer, waveform_buffer)
  }

  pub async fn new(device: &wgpu::Device, size: usize) -> Self {
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

    let (spectrum_buffer, waveform_buffer) = Self::create_buffers(device, size);

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

  pub fn resize(&mut self, device: &wgpu::Device, size: usize) {
    let (spectrum_buffer, waveform_buffer) = Self::create_buffers(device, size);
    self.spectrum_buffer = spectrum_buffer;
    self.waveform_buffer = waveform_buffer;
  }

  pub fn layout(&self) -> &wgpu::BindGroupLayout {
    &self.audio_data_bind_group_layout
  }

  pub fn update_spectrum(&self, spectrum: &[f32], queue: &wgpu::Queue) {
    queue.write_buffer(&self.spectrum_buffer, 0, bytemuck::cast_slice(&spectrum));
  }

  pub fn update_waveform(&self, waveform: &[f32], queue: &wgpu::Queue) {
    queue.write_buffer(&self.waveform_buffer, 0, bytemuck::cast_slice(&waveform));
  }
}

pub trait BindAudioData<'a> {
  fn bind_audio_data(&mut self, index: u32, audio_data: &'a AudioData);
}
impl<'a, 'b> BindAudioData<'b> for wgpu::RenderPass<'a>
where
  'b: 'a,
{
  fn bind_audio_data(&mut self, index: u32, audio_data: &'b AudioData) {
    self.set_bind_group(index, &audio_data.audio_data_bind_group, &[]);
  }
}
