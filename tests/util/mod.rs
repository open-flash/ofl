use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::FileType;
use std::path::Path;

#[allow(unused)]
pub(crate) fn assert_same_directory_content(actual: impl AsRef<Path>, expected: impl AsRef<Path>) {
  let actual_entries = read_dir(actual).unwrap();
  let expected_entries = read_dir(expected).unwrap();
  assert_eq!(actual_entries, expected_entries);
}

fn read_dir<P: AsRef<Path>>(path: P) -> Result<BTreeMap<OsString, FileType>, std::io::Error> {
  let mut entries: BTreeMap<OsString, FileType> = BTreeMap::new();
  for entry in path.as_ref().read_dir()? {
    let entry = entry?;
    let file_name: OsString = entry.file_name();
    let file_type: FileType = entry.file_type()?;
    let old = entries.insert(file_name, file_type);
    debug_assert!(old.is_none())
  }
  Ok(entries)
}
