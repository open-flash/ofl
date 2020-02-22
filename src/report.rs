//use sha2::Sha256;
//
//pub struct Report {
//  pub file_size: u32,
//  pub hashes: Hashes,
//}
//
//pub struct Hashes {
//  pub sha256: [u8; 32],
//}
//
////fn hash() {
////  Sha256::digest()
////}
//
//impl Report {
//  fn print<W: std::io::Write>(&self, writer: &mut W) -> () {
//    writeln!(writer, "File size: {} bytes", self.file_size);
//    writeln!(writer, "SHA-256: {:?}", self.hashes.sha256);
//  }
//}
