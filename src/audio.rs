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
  pub fn init() -> Self {
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
      resolution: None,
      fft_resolution: 1024 * 3,
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

  pub fn resolution(&self) -> Option<usize> {
    self.config.resolution
  }

  pub fn set_resolution(&mut self, new_resolution: Option<usize>) {
    self.config.resolution = new_resolution;
  }

  fn process_frequencies(&mut self) -> Vec<Vec<Frequency>> {
    if self
      .channel_buffers
      .iter()
      .any(|channel_buffer| channel_buffer.len() < self.config.fft_resolution)
    {
      return Vec::new();
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

    channel_spectrum_buffers
  }

  pub fn process_data(&mut self) -> Option<Vec<f32>> {
    let Ok(data) = self.data_rx.try_recv() else {
      return None;
    };

    for channels in data.chunks_exact(self.config.channel_count) {
      for (channel_number, sample) in channels.iter().enumerate() {
        self.channel_buffers[channel_number].push_back(*sample);
      }
    }

    for buffer in &mut self.channel_buffers {
      let excess_elements = buffer.len().saturating_sub(self.config.fft_resolution);
      buffer.drain(0..excess_elements);
    }

    let frequencies = self.process_frequencies();
    if frequencies.len() == 0 {
      return None;
    }

    let resolution = frequencies[0].len();
    let mut spectrum = Vec::with_capacity(resolution);

    for index in 0..resolution {
      let mut average_amplitude = 0.0;
      for channel in &frequencies {
        average_amplitude += channel[index].volume;
      }

      average_amplitude /= frequencies.len() as f32;

      spectrum.push(average_amplitude);
    }

    Some(spectrum)
  }

  pub fn get_waveform(&self) -> Vec<f32> {
    let Some(waveform_len) = self.channel_buffers.iter().map(|buffer| buffer.len()).min() else {
      return Vec::new();
    };

    let waveform_buffer_len = self.config.resolution.unwrap_or(waveform_len);
    let mut waveform_buffer = Vec::with_capacity(waveform_buffer_len);

    let num_samples = if let Some(resolution) = self.config.resolution {
      waveform_len.min(resolution)
    } else {
      waveform_len
    };

    for index in (waveform_len - num_samples)..waveform_len {
      let mut average_sample = 0.0;
      for buffer in &self.channel_buffers {
        average_sample += buffer[index];
      }

      average_sample /= self.channel_buffers.len() as f32;
      waveform_buffer.push(average_sample);
    }

    waveform_buffer.resize(waveform_buffer_len, 0.0);
    waveform_buffer
  }

  fn handle_stream_error(error: cpal::StreamError) {
    eprintln!("an error occurred on the audio stream: {}", error);
  }
}

struct AudioProcessorConfig {
  fft_resolution: usize,
  resolution: Option<usize>,
  sampling_rate: u32,
  channel_count: usize,
}

impl AudioProcessorConfig {
  fn into_spectrum_processor_config(&self) -> SpectrumProcessorConfig {
    SpectrumProcessorConfig {
      sampling_rate: self.sampling_rate,
      resolution: self.resolution,
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
