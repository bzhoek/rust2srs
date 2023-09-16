use std::fs;
use std::fs::write;

use rmp3::{Decoder, Frame};

use crate::{Dialogue, Result};

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
              return Ok(());
            }
          }
        }
      }
      decoder.skip();
    }
    Ok(())
  }
}

pub fn extract_sound_clips(audio_file: &str, folder: &str, prefix: &str, subtitles: Vec<Dialogue>)
                           -> Result<()> {
  let mp3 = Mp3::new(audio_file)?;
  for dialogue in subtitles {
    let audio_file = format!("{}/{}_{}-{}.mp3", folder, prefix, dialogue.start, dialogue.end);
    mp3.slice(&audio_file, dialogue.start.milliseconds(), dialogue.end.milliseconds()).unwrap();
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use std::fs;

  use rmp3::{Decoder, Frame};

  use crate::{parse_subtitle_file, Time};
  use crate::mp3::{extract_sound_clips, Mp3};

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
  fn it_slices_ichigo() {
    let subtitles = parse_subtitle_file("tests/ichigo-01.ass").unwrap();
    extract_sound_clips("ichigo-01.mp3", "target", "ichigo-01", subtitles).unwrap();
  }
  fn it_extracts_totoro() {
    let subtitles = parse_subtitle_file("tests/totoro.ja.vtt").unwrap();
    extract_sound_clips("totoro.mp3", "target", "totoro", subtitles).unwrap();
  }

  #[test]
  fn it_slices_totoro() {
    let mp3 = Mp3::new("totoro.mp3").unwrap();
    // 00:04:52,470 --> 00:04:54,490
    let start = Time { hour: 0, min: 4, sec: 52, mil: 470 };
    let end = Time { hour: 0, min: 4, sec: 54, mil: 490 };
    mp3.slice("target/output-1.mp3", start.milliseconds(), end.milliseconds()).unwrap();
  }

  #[test]
  fn it_extracts_one_totoro_dialogue() {
    let mut dialogue = parse_subtitle_file("tests/totoro.ja.vtt").unwrap();
    assert_eq!(839, dialogue.len());
    let one = dialogue.remove(26);
    let dialogue = vec![one];
    extract_sound_clips("totoro.mp3", "target", "totoro", dialogue).unwrap();
  }


}