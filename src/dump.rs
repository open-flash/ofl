use avm1_parser::parse_cfg;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use swf_types::tags::{DefineSprite, DoAction, DoInitAction};
use swf_types::Movie;
use swf_types::{Header, Tag};

pub(crate) fn dump_movie(dir: &PathBuf, movie: &Movie) {
  {
    let path = dir.join("movie.json");
    let file = std::fs::File::create(path).expect("Failed to create movie file");
    let writer = std::io::BufWriter::new(file);

    let mut ser = serde_json_v8::Serializer::pretty(writer);
    movie.serialize(&mut ser).expect("Failed to serialize movie");
    ser.into_inner().write_all(b"\n").expect("Failed to write movie");
  }

  dump_header(dir, &movie.header);

  for (i, tag) in movie.tags.iter().enumerate() {
    let tag_dir = dir.join(format!("{}", i));
    std::fs::create_dir(&tag_dir).expect("Failed to create tag directory");
    dump_tag(&tag_dir, tag);
  }
}

fn dump_header(dir: &PathBuf, header: &Header) {
  let path = dir.join("header.json");
  let file = std::fs::File::create(path).expect("Failed to create header file");
  let writer = std::io::BufWriter::new(file);

  let mut ser = serde_json_v8::Serializer::pretty(writer);
  header.serialize(&mut ser).expect("Failed to serialize header");
  ser.into_inner().write_all(b"\n").expect("Failed to write header");
}

fn dump_tag(dir: &PathBuf, tag: &Tag) {
  let path = dir.join("tag.json");
  let file = std::fs::File::create(path).expect("Failed to create tag file");
  let writer = std::io::BufWriter::new(file);

  let mut ser = serde_json_v8::Serializer::pretty(writer);
  tag.serialize(&mut ser).expect("Failed to serialize tag");
  ser.into_inner().write_all(b"\n").expect("Failed to write tag");

  if let Tag::DefineSprite(tag) = tag {
    dump_define_sprite(dir, tag)
  }
}

fn dump_define_sprite(dir: &PathBuf, tag: &DefineSprite) {
  for (i, tag) in tag.tags.iter().enumerate() {
    let tag_dir = dir.join(format!("{}", i));
    std::fs::create_dir(&tag_dir).expect("Failed to create sprite tag directory");
    dump_sprite_tag(&tag_dir, tag);
  }
}

fn dump_sprite_tag(dir: &PathBuf, tag: &Tag) {
  let path = dir.join("tag.json");
  let file = std::fs::File::create(path).expect("Failed to create sprite tag file");
  let writer = std::io::BufWriter::new(file);

  let mut ser = serde_json_v8::Serializer::pretty(writer);
  tag.serialize(&mut ser).expect("Failed to serialize sprite tag");
  ser.into_inner().write_all(b"\n").expect("Failed to write sprite tag");

  match tag {
    Tag::DoAction(tag) => dump_do_action(dir, tag),
    Tag::DoInitAction(tag) => dump_do_init_action(dir, tag),
    _ => {}
  }
}

#[derive(Copy, Clone, Hash, Ord, PartialOrd, PartialEq, Eq, Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Avm1Location {
  RootDoAction { tag_index: usize },
  RootDoInitAction { tag_index: usize },
  SpriteDoAction { tag_index: usize, sprite_tag_index: usize },
  SpriteDoInitAction { tag_index: usize, sprite_tag_index: usize },
}

pub(crate) fn find_avm1(movie: &Movie) -> HashMap<Avm1Location, &Vec<u8>> {
  let mut avm1_buffers = HashMap::new();
  for (tag_index, tag) in movie.tags.iter().enumerate() {
    match tag {
      Tag::DefineSprite(tag) => {
        for (sprite_tag_index, sprite_tag) in tag.tags.iter().enumerate() {
          match sprite_tag {
            Tag::DoAction(tag) => {
              avm1_buffers.insert(
                Avm1Location::SpriteDoAction {
                  tag_index,
                  sprite_tag_index,
                },
                &tag.actions,
              );
            }
            Tag::DoInitAction(tag) => {
              avm1_buffers.insert(
                Avm1Location::SpriteDoInitAction {
                  tag_index,
                  sprite_tag_index,
                },
                &tag.actions,
              );
            }
            _ => {}
          }
        }
      }
      Tag::DoAction(tag) => {
        avm1_buffers.insert(Avm1Location::RootDoAction { tag_index }, &tag.actions);
      }
      Tag::DoInitAction(tag) => {
        avm1_buffers.insert(Avm1Location::RootDoInitAction { tag_index }, &tag.actions);
      }
      _ => {}
    }
  }
  avm1_buffers
}

fn dump_do_action(dir: &PathBuf, tag: &DoAction) {
  {
    let path = dir.join("main.avm1");
    let file = std::fs::File::create(path).expect("Failed to create AVM1 file");
    let mut writer = std::io::BufWriter::new(file);
    writer.write_all(&tag.actions).expect("Failed to write AVM1");
  }
  {
    let cfg = parse_cfg(&tag.actions);

    let path = dir.join("main.cfg.json");
    let file = std::fs::File::create(path).expect("Failed to create CFG file");
    let writer = std::io::BufWriter::new(file);

    let mut ser = serde_json_v8::Serializer::pretty(writer);
    cfg.serialize(&mut ser).expect("Failed to serialize CFG");
    ser.into_inner().write_all(b"\n").expect("Failed to write CFG");
  }
}

fn dump_do_init_action(dir: &PathBuf, tag: &DoInitAction) {
  {
    let path = dir.join("main.avm1");
    let file = std::fs::File::create(path).expect("Failed to create AVM1 file");
    let mut writer = std::io::BufWriter::new(file);
    writer.write_all(&tag.actions).expect("Failed to write AVM1");
  }
  {
    let cfg = parse_cfg(&tag.actions);

    let path = dir.join("main.cfg.json");
    let file = std::fs::File::create(path).expect("Failed to create CFG file");
    let writer = std::io::BufWriter::new(file);

    let mut ser = serde_json_v8::Serializer::pretty(writer);
    cfg.serialize(&mut ser).expect("Failed to serialize CFG");
    ser.into_inner().write_all(b"\n").expect("Failed to write CFG");
  }
}
