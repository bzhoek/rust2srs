use std::fs;
use std::fs::write;

use rmp3::{Decoder, Frame};

use crate::Result;

pub struct Mp3 {
  bytes: Vec<u8>,
}

impl Mp3 {
  fn new(file: &str) -> Result<Mp3> {
    let bytes = fs::read(file)?;
    Ok(Mp3 { bytes })
  }

  fn slice(&self, file: &str, start: u64, end: u64) -> Result<()> {
    let mut decoder = Decoder::new(&self.bytes);
    let mut duration: f64 = 0.0;
    let mut index = None;
    while let Some(frame) = decoder.peek() {
      if let Frame::Audio(audio) = frame {
        let ms = audio.sample_count() as f64 / audio.sample_rate() as f64 * 1000f64;
        duration += ms;
        match index {
          None => {
            if duration >= start as f64 {
              index = Some(decoder.position());
            }
          }
          Some(start) => {
            if duration >= end as f64 {
              let contents = &self.bytes[start..decoder.position()];
              write(file, contents)?;
              return Ok(())
            }
          }
        }
      }
      decoder.skip();
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs;

  use rmp3::{Decoder, Frame};
  use crate::assa::parse_assa_to_dialogue;

  use crate::Time;
  use crate::mp3::Mp3;

  #[test]
  fn it_uses_rmp3() {
    let file = fs::read("ichigo-01.mp3").unwrap();
    assert_eq!(23234644, file.len());
    let mut decoder = Decoder::new(&file);

    let mut frame_count = 0;
    let mut ms_duration: f64 = 0.0;

    while let Some(frame) = decoder.peek() {
      if let Frame::Audio(audio) = frame {
        assert_eq!(1152, audio.sample_count());
        assert_eq!(48000, audio.sample_rate());
        let ms = audio.sample_count() as f64 / audio.sample_rate() as f64 * 1000f64;
        ms_duration += ms;
        frame_count += 1;
      }
      decoder.skip();
    }

    assert_eq!(24.2024, ms_duration / 1000f64 / 60f64);
    assert_eq!(60506, frame_count);
  }

  #[test]
  fn it_slices_mp3() {
    let mp3 = Mp3::new("ichigo-01.mp3").unwrap();
    let start = Time { hour: 0, min: 1, sec: 39, mil: 620 };
    let end = Time { hour: 0, min: 1, sec: 41, mil: 620 };
    mp3.slice("target/output-1.mp3", start.milliseconds(), end.milliseconds()).unwrap();
  }

  #[test]
  fn it_slices_dialogue() {
    let mp3 = Mp3::new("ichigo-01.mp3").unwrap();
    let contents = fs::read_to_string("tests/ichigo-01.ass").unwrap();
    let source = parse_assa_to_dialogue(&contents).unwrap();
    let folder = "target";
    let prefix = "ichigo-01";
    for dialogue in source {
      // ichigo-1_1_0.02.38.090-0.02.40.380
      let audio_file = format!("{}/{}_{}-{}.mp3", folder, prefix, dialogue.start, dialogue.end);
      mp3.slice(&audio_file, dialogue.start.milliseconds(), dialogue.end.milliseconds()).unwrap();
    }
  }
}