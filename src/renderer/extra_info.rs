use wgpu::util::DeviceExt;

pub struct ExtraInfo {
    time_buffer: wgpu::Buffer,
    resolution_buffer: wgpu::Buffer,
    extra_info_bind_group: wgpu::BindGroup,
    extra_info_bind_group_layout: wgpu::BindGroupLayout,
}

impl ExtraInfo {
    pub async fn new(device: &wgpu::Device) -> Self {
        let extra_info_bind_group_layout =
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

        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time Buffer"),
            contents: bytemuck::cast_slice(&[0.0f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let resolution_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Resolution Buffer"),
            contents: bytemuck::cast_slice(&[0.0f32, 0.0f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let extra_info_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &extra_info_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(time_buffer.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(
                        resolution_buffer.as_entire_buffer_binding(),
                    ),
                },
            ],
            label: Some("fragment_bind_group"),
        });

        Self {
            time_buffer,
            resolution_buffer,
            extra_info_bind_group,
            extra_info_bind_group_layout,
        }
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.extra_info_bind_group_layout
    }

    pub fn update_time(&self, new_time_secs: f32, queue: &wgpu::Queue) {
        queue.write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&[new_time_secs]));
    }

    pub fn update_resolution(
        &self,
        new_resolution: winit::dpi::PhysicalSize<f32>,
        queue: &wgpu::Queue,
    ) {
        queue.write_buffer(
            &self.resolution_buffer,
            0,
            bytemuck::cast_slice(&[new_resolution.width, new_resolution.height]),
        );
    }
}

pub trait BindExtraInfo<'a> {
    fn bind_extra_info(&mut self, extra_info: &'a ExtraInfo);
}
impl<'a, 'b> BindExtraInfo<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn bind_extra_info(&mut self, extra_info: &'b ExtraInfo) {
        self.set_bind_group(0, &extra_info.extra_info_bind_group, &[]);
    }
}
