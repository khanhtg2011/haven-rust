use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct AudioCaptureService {
    is_capturing: Arc<AtomicBool>,
}

impl AudioCaptureService {
    pub fn new() -> Self {
        Self {
            is_capturing: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_supported() -> bool {
        let host = cpal::default_host();
        host.default_output_device().is_some()
    }

    pub fn start_loopback_capture<F>(&self, mut on_data: F) -> Result<()>
    where
        F: FnMut(&[f32]) + Send + 'static,
    {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No default output audio device found"))?;

        let config = device.default_output_config()?;
        tracing::info!(
            device = %device.name().unwrap_or_default(),
            sample_rate = config.sample_rate().0,
            channels = config.channels(),
            "Starting loopback audio capture for Haven screen share"
        );

        self.is_capturing.store(true, Ordering::SeqCst);
        let capturing = self.is_capturing.clone();

        std::thread::spawn(move || {
            // Simulated capture loop or real stream callback
            while capturing.load(Ordering::SeqCst) {
                let silence = vec![0.0f32; 480];
                on_data(&silence);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        Ok(())
    }

    pub fn stop_capture(&self) {
        self.is_capturing.store(false, Ordering::SeqCst);
        tracing::info!("Stopped loopback audio capture");
    }
}

impl Default for AudioCaptureService {
    fn default() -> Self {
        Self::new()
    }
}
