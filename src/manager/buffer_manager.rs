// use weresocool_instrument::StereoWaveform;

// #[derive(Clone, Debug)]
// pub struct Buffer {
// pub stereo_waveform: StereoWaveform,
// pub write_idx: usize,
// pub read_idx: usize,
// }

// /// This assumes that all buffers are the same size
// impl Buffer {
// pub fn init() -> Self {
// Self {
// stereo_waveform: StereoWaveform::new(0),
// write_idx: 0,
// read_idx: 0,
// }
// }

// pub const fn init_with_buffer(stereo_waveform: StereoWaveform) -> Self {
// Self {
// stereo_waveform,
// write_idx: 0,
// read_idx: 0,
// }
// }

// pub fn write(&mut self, stereo_waveform: StereoWaveform) {
// self.stereo_waveform.append(stereo_waveform);
// self.write_idx += 1;
// }

// pub fn read(&mut self, buffer_size: usize) -> Option<StereoWaveform> {
// let sw = self.stereo_waveform.get_buffer(self.read_idx, buffer_size);
// if sw.is_some() {
// self.read_idx += 1;
// };
// sw
// }
// }

// #[derive(Clone, Debug)]
// pub struct BufferManager {
// pub buffers: [Option<Buffer>; 2],
// renderer_write_idx: usize,
// buffer_idx: usize,
// }

// impl BufferManager {
// pub const fn init_silent() -> Self {
// Self {
// buffers: [None, None],
// renderer_write_idx: 0,
// buffer_idx: 0,
// }
// }

// pub const fn init_wth_buffer(buffer: Buffer) -> Self {
// Self {
// buffers: [Some(buffer), None],
// renderer_write_idx: 0,
// buffer_idx: 0,
// }
// }

// pub fn inc_buffer(&mut self) {
// self.buffer_idx = (self.buffer_idx + 1) % 2;
// }

// pub fn inc_render_write_buffer(&mut self) {
// self.renderer_write_idx = (self.renderer_write_idx + 1) % 2;
// }

// pub fn current_buffer(&mut self) -> &mut Option<Buffer> {
// &mut self.buffers[self.buffer_idx]
// }

// pub fn current_render_write_buffer(&mut self) -> &mut Option<Buffer> {
// &mut self.buffers[self.renderer_write_idx]
// }

// pub fn next_buffer(&mut self) -> &mut Option<Buffer> {
// &mut self.buffers[(self.buffer_idx + 1) % 2]
// }

// pub fn exists_current_buffer(&mut self) -> bool {
// self.current_buffer().is_some()
// }

// pub fn exists_next_buffer(&mut self) -> bool {
// self.next_buffer().is_some()
// }

// pub fn read(&mut self, buffer_size: usize) -> Option<StereoWaveform> {
// let next = self.exists_next_buffer();
// let current = self.current_buffer();

// match current {
// Some(buffer) => {
// let mut sw = buffer.read(buffer_size);

// if next {
// if let Some(s) = sw.as_mut() {
// s.fade_out()
// }

// *current = None;
// self.inc_buffer();
// }
// sw
// }
// None => {
// if next {
// self.inc_buffer();
// self.read(buffer_size)
// } else {
// None
// }
// }
// }
// }

// pub fn write(&mut self, stereo_waveform: StereoWaveform) {
// let current = self.current_render_write_buffer();
// match current {
// Some(buffer) => buffer.write(stereo_waveform),
// None => {
// let mut new_buffer = Buffer::init();
// new_buffer.write(stereo_waveform);
// *current = Some(new_buffer);
// }
// }
// }
// }

// #[cfg(test)]
// mod buffer_manager_tests {
// use super::*;
// #[test]
// fn test_inc_buffer() {
// let mut r = BufferManager::init_silent();
// r.inc_buffer();
// assert_eq!(r.buffer_idx, 1);
// r.inc_buffer();
// assert_eq!(r.buffer_idx, 0);
// }

// #[test]
// fn test_inc_render_write_buffer() {
// let mut r = BufferManager::init_silent();
// r.inc_render_write_buffer();
// assert_eq!(r.renderer_write_idx, 1);
// r.inc_render_write_buffer();
// assert_eq!(r.renderer_write_idx, 0);
// }

// fn buffer_manager_mock() -> BufferManager {
// BufferManager::init_wth_buffer(Buffer::init_with_buffer(StereoWaveform::new_with_buffer(
// vec![1.0, 1.0, 1.0, 1.0],
// )))
// }

// #[test]
// fn test_read_normal() {
// let mut b = buffer_manager_mock();
// let read = b.read(2);

// let expected = StereoWaveform::new_with_buffer(vec![1.0, 1.0]);
// assert_eq!(read.unwrap(), expected);
// }

// #[test]
// fn test_read_with_next_fade() {
// let mut b = buffer_manager_mock();
// *b.next_buffer() = Some(Buffer::init_with_buffer(StereoWaveform::new_with_buffer(
// vec![1.0, 1.0],
// )));
// let read = b.read(2);

// let expected = StereoWaveform::new_with_buffer(vec![0.5, 0.0]);
// assert_eq!(read.unwrap(), expected);
// }

// #[test]
// fn test_read_with_empty_current() {
// let mut b = buffer_manager_mock();
// b.inc_buffer();
// assert_eq!(b.exists_current_buffer(), false);
// assert_eq!(b.exists_next_buffer(), true);

// let read = b.read(2);

// let expected = StereoWaveform::new_with_buffer(vec![1.0, 1.0]);
// assert_eq!(read.unwrap(), expected);
// }

// #[test]
// fn test_read_empty_buffer_manager() {
// let mut b = BufferManager::init_silent();
// assert_eq!(b.exists_current_buffer(), false);
// assert_eq!(b.exists_next_buffer(), false);

// let read = b.read(2);

// assert_eq!(read, None);
// }
// }
