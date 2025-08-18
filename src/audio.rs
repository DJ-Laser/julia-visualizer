use std::{collections::VecDeque, sync::mpsc};

use audioviz::spectrum::{
  Frequency, config::ProcessorConfig as SpectrumProcessorConfig,
  processor::Processor as SpectrumProcessor,
};
use cpal::traits::{DeviceTrait, HostTrait};
pub struct AudioProcessor {
  input_stream: cpal::Stream,
  data_rx: mpsc::Receiver<Vec<f32>>,
  channel_buffers: Vec<VecDeque<f32>>,
  config: AudioProcessorConfig,
}

impl AudioProcessor {
  pub fn init(fft_resolution: usize) -> Self {
    let (data_tx, data_rx) = mpsc::channel();

    let device = find_output_monitor().unwrap();

    let stream_config = device.default_input_config().unwrap().into();
    let input_stream = device
      .build_input_stream(
        &stream_config,
        move |data, _: &_| {
          let _ = data_tx.send(data.to_vec());
        },
        Self::handle_stream_error,
        None,
      )
      .unwrap();

    let channel_count = stream_config.channels as usize;
    let config = AudioProcessorConfig {
      fft_resolution,
      sampling_rate: stream_config.sample_rate.0,
      channel_count,
    };

    Self {
      input_stream,
      data_rx,
      channel_buffers: vec![VecDeque::with_capacity(config.fft_resolution); channel_count],
      config,
    }
  }

  pub fn process_data(&mut self) -> Option<Vec<Vec<Frequency>>> {
    let data = self.data_rx.try_recv().unwrap_or(Vec::new());

    for channels in data.chunks_exact(self.config.channel_count) {
      for (channel_number, sample) in channels.iter().enumerate() {
        self.channel_buffers[channel_number].push_back(*sample);
      }
    }

    for buffer in &mut self.channel_buffers {
      let excess_elements = buffer.len().saturating_sub(self.config.fft_resolution);
      buffer.drain(0..excess_elements);
    }

    self.process_frequencies()
  }

  fn process_frequencies(&mut self) -> Option<Vec<Vec<Frequency>>> {
    if self
      .channel_buffers
      .iter()
      .any(|channel_buffer| channel_buffer.len() < self.config.fft_resolution)
    {
      return None;
    }

    let mut channel_spectrum_buffers = Vec::with_capacity(self.config.channel_count);

    for channel_buffer in self.channel_buffers.iter_mut() {
      let buffer_start_offset = channel_buffer.len() - self.config.fft_resolution;
      let mut audio_data = SpectrumProcessor::from_raw_data(
        self.config.into_spectrum_processor_config(),
        channel_buffer
          .range(buffer_start_offset..)
          .copied()
          .collect(),
      );

      audio_data.compute_all();
      channel_spectrum_buffers.push(audio_data.freq_buffer);
    }

    Some(channel_spectrum_buffers)
  }

  fn handle_stream_error(error: cpal::StreamError) {
    eprintln!("an error occurred on the audio stream: {}", error);
  }
}

struct AudioProcessorConfig {
  fft_resolution: usize,
  sampling_rate: u32,
  channel_count: usize,
}

impl AudioProcessorConfig {
  fn into_spectrum_processor_config(&self) -> SpectrumProcessorConfig {
    SpectrumProcessorConfig {
      sampling_rate: self.sampling_rate,
      resolution: None,
      volume: 1.0,
      ..SpectrumProcessorConfig::default()
    }
  }
}

fn find_output_monitor() -> Option<cpal::Device> {
  let host = cpal::default_host();
  let mut devices = host
    .output_devices()
    .into_iter()
    .flatten()
    .chain(host.output_devices().into_iter().flatten());

  return devices.find(|device| device.supports_input());
}

fn is_device_supported(device: &cpal::Device) -> bool {
  if !device.supports_input() {
    return false;
  }

  let Ok(mut configs) = device.supported_input_configs() else {
    return false;
  };

  configs.any(|config| matches!(config.sample_format(), cpal::SampleFormat::F32))
}
