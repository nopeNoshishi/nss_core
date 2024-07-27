// Std
use std::fs::File;

// External
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

pub(crate) fn read_decoder(reader: File) -> ZlibDecoder<File> {
    ZlibDecoder::new(reader)
}

pub(crate) fn write_encoder(writer: File) -> ZlibEncoder<File> {
    ZlibEncoder::new(writer, Compression::default())
}
