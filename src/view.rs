use super::*;
use rustbox::Key;
use rustbox::{Color, OutputMode, RustBox};
use std::char;
use std::default::Default;

pub struct View<'a> {
  state: &'a mut state::State<'a>,
  width: usize,
  skip: usize,
  reverse: bool,
  unique: bool,
  contrast: bool,
  position: &'a str,
  select_foreground_color: Color,
  foreground_color: Color,
  background_color: Color,
  hint_background_color: Color,
  hint_foreground_color: Color,
  copy_mode_cursor_up_key: Option<char>,
  copy_mode_cursor_down_key: Option<char>,
}

impl<'a> View<'a> {
  pub fn new(
    state: &'a mut state::State<'a>,
    width: usize,
    reverse: bool,
    unique: bool,
    contrast: bool,
    position: &'a str,
    select_foreground_color: Color,
    foreground_color: Color,
    background_color: Color,
    hint_foreground_color: Color,
    hint_background_color: Color,
    copy_mode_cursor_up_key: Option<char>,
    copy_mode_cursor_down_key: Option<char>,
  ) -> View<'a> {
    View {
      state: state,
      width: width,
      skip: 0,
      reverse: reverse,
      unique: unique,
      contrast: contrast,
      position: position,
      select_foreground_color: select_foreground_color,
      foreground_color: foreground_color,
      background_color: background_color,
      hint_foreground_color: hint_foreground_color,
      hint_background_color: hint_background_color,
      copy_mode_cursor_up_key: copy_mode_cursor_up_key,
      copy_mode_cursor_down_key: copy_mode_cursor_down_key,
    }
  }

  pub fn prev(&mut self) {
    if self.skip > 0 {
      self.skip = self.skip - 1;
    }
  }

  pub fn next(&mut self, maximum: usize) {
    if self.skip < maximum {
      self.skip = self.skip + 1;
    }
  }

  fn make_hint_text(&self, hint: &str) -> String {
    let text = if self.contrast {
      format!("[{}]", hint).to_string()
    } else {
      hint.to_string()
    };

    text
  }

  pub fn present(&mut self) -> Option<(String, bool, Option<String>)> {
    let mut rustbox = match RustBox::init(Default::default()) {
      Result::Ok(v) => v,
      Result::Err(e) => panic!("{}", e),
    };

    rustbox.set_output_mode(OutputMode::EightBit);

    let mut typed_hint: String = "".to_owned();
    let matches = self.state.matches(self.reverse, self.unique);
    let longest_hint = matches
      .iter()
      .filter_map(|m| m.hint.clone())
      .max_by(|x, y| x.len().cmp(&y.len()))
      .unwrap()
      .clone();
    let mut selected;

    self.skip = if self.reverse { matches.len() - 1 } else { 0 };

    loop {
      rustbox.clear();
      rustbox.present();

      // start_ys[i] = j means that the i-th logical row should start on the
      // j-th physical row considering line-wrapping.
      let mut start_ys: Vec<usize> = Vec::with_capacity(self.state.lines.len());
      start_ys.push(0);
      for (index, line) in self.state.lines.iter().enumerate() {
        let clean = line.trim_end_matches(|c: char| c.is_whitespace());
        let start_y = start_ys[index];
        let line_chars = clean.chars().count();
        let mut char_indices: Vec<usize> = clean.char_indices().map(|(i, _)| i).collect();
        char_indices.push(clean.len());

        for start_x in (0..line_chars).step_by(self.width) {
          rustbox.print(
            0,
            start_y + (start_x / self.width),
            rustbox::RB_NORMAL,
            Color::White,
            Color::Black,
            &clean[char_indices[start_x]..char_indices[line_chars.min(start_x + self.width)]],
          );
        }

        let line_count = (line_chars.max(1) - 1) / self.width + 1;
        start_ys.push(start_y + line_count);
      }

      // This is a closure that translates a logical (col, row) coordinate into
      // a physical (x, y) coord that we should pass to rustbox::print.
      let translate_coord =
        |col: usize, row: usize| return (col % self.width, start_ys[row] + (col / self.width));

      selected = matches.get(self.skip);

      for mat in matches.iter() {
        let selected_color = if selected == Some(mat) {
          self.select_foreground_color
        } else {
          self.foreground_color
        };

        // Find long utf sequences and extract it from mat.x
        let line = &self.state.lines[mat.y as usize];
        let prefix = &line[0..mat.x as usize];
        let extra = prefix.len() - prefix.chars().count();
        let offset = (mat.x as usize) - extra;
        let text = self.make_hint_text(mat.text);

        for ch_index in 0..text.len() {
          let (x, y) = translate_coord(offset + ch_index, mat.y as usize);

          rustbox.print(
            x,
            y,
            rustbox::RB_NORMAL,
            selected_color,
            self.background_color,
            &text[ch_index..ch_index + 1],
          );
        }

        if let Some(ref hint) = mat.hint {
          let extra_position = if self.position == "left" {
            0
          } else {
            text.len() - mat.hint.clone().unwrap().len()
          };

          let text = self.make_hint_text(hint.as_str());

          for ch_index in 0..text.len() {
            let (x, y) = translate_coord(offset + extra_position + ch_index, mat.y as usize);

            rustbox.print(
              x,
              y,
              rustbox::RB_BOLD,
              self.hint_foreground_color,
              self.hint_background_color,
              &text[ch_index..ch_index + 1],
            );
          }
        }
      }

      rustbox.present();

      match rustbox.poll_event(false) {
        Ok(rustbox::Event::KeyEvent(key)) => match key {
          Key::Esc => {
            break;
          }
          Key::Enter => match matches.iter().enumerate().find(|&h| h.0 == self.skip) {
            Some(hm) => return Some((hm.1.text.to_string(), false, None)),
            _ => panic!("Match not found?"),
          },
          Key::Up => {
            self.prev();
          }
          Key::Down => {
            self.next(matches.len() - 1);
          }
          Key::Left => {
            self.prev();
          }
          Key::Right => {
            self.next(matches.len() - 1);
          }
          Key::Char(ch) => {
            let key = ch.to_string();
            let lower_key = key.to_lowercase();

            if let Some(up_key) = self.copy_mode_cursor_up_key {
              if up_key == ch {
                return Some((String::new(), false, Some(String::from("cursor-up"))));
              }
            }

            if let Some(down_key) = self.copy_mode_cursor_down_key {
              if down_key == ch {
                return Some((String::new(), false, Some(String::from("cursor-down"))));
              }
            }

            typed_hint.push_str(lower_key.as_str());

            match matches
              .iter()
              .find(|mat| mat.hint == Some(typed_hint.clone()))
            {
              Some(mat) => return Some((mat.text.to_string(), key != lower_key, None)),
              None => {
                if typed_hint.len() >= longest_hint.len() {
                  break;
                }
              }
            }
          }
          _ => {}
        },
        Err(e) => panic!("{}", e),
        _ => {}
      }
    }

    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn split(output: &str) -> Vec<&str> {
    output.split("\n").collect::<Vec<&str>>()
  }

  #[test]
  fn hint_text() {
    let lines = split("lorem 127.0.0.1 lorem");
    let custom = [].to_vec();
    let mut state = state::State::new(&lines, "abcd", &custom);
    let mut view = View {
      state: &mut state,
      skip: 0,
      reverse: false,
      unique: false,
      contrast: false,
      position: &"",
      select_foreground_color: rustbox::Color::Default,
      foreground_color: rustbox::Color::Default,
      background_color: rustbox::Color::Default,
      hint_background_color: rustbox::Color::Default,
      hint_foreground_color: rustbox::Color::Default,
    };

    let result = view.make_hint_text("a");
    assert_eq!(result, "a".to_string());

    view.contrast = true;
    let result = view.make_hint_text("a");
    assert_eq!(result, "[a]".to_string());
  }
}
