use avm1_parser::parse_cfg;
use serde::Serialize;
use std::io::{Read, Write};
use std::path::PathBuf;
use swf_parser::parse_swf;
use swf_types::tags::{DefineSprite, DoAction, DoInitAction};
use swf_types::Movie;
use swf_types::{Header, Tag};

pub(crate) fn dump<R: Read>(dir: &PathBuf, movie_reader: &mut R) {
  let swf_bytes: Vec<u8> = {
    let mut swf_bytes: Vec<u8> = Vec::new();
    movie_reader.read_to_end(&mut swf_bytes).expect("Failed to read SWF");
    swf_bytes
  };

  let movie = match parse_swf(&swf_bytes) {
    Ok(ok) => ok,
    Err(e) => panic!("Failed to parse SWF:\n{:?}", e),
  };

  dump_movie(dir, &movie);
}

fn dump_movie(dir: &PathBuf, movie: &Movie) {
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

  match tag {
    Tag::DefineSprite(tag) => dump_define_sprite(dir, tag),
    Tag::DoAction(tag) => dump_do_action(dir, tag),
    Tag::DoInitAction(tag) => dump_do_init_action(dir, tag),
    _ => {}
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

fn dump_define_sprite(dir: &PathBuf, tag: &DefineSprite) {
  for (i, tag) in tag.tags.iter().enumerate() {
    let tag_dir = dir.join(format!("{}", i));
    std::fs::create_dir(&tag_dir).expect("Failed to create sprite tag directory");
    dump_sprite_tag(&tag_dir, tag);
  }
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
