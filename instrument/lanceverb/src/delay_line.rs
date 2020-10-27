/// Delay-line whose maximum size is fixed
/// The advantage of using a static versus dynamic array is that its elements
/// can be laid out in a predicatable location in memeory. This can improve
/// access speeds if many delay-lines are used within another object, like a 
/// reverb
pub struct DelayLine<B> {
    pos: usize,
    buffer: B,
}

impl<B> DelayLine<B> where  B: Buffer {

    /// Default constructor for a delay line
    pub fn new() -> DelayLine<B> {
        DelayLine { 
            pos: 0, 
            buffer: B::zeroed(),
        }
    }

    /// Get size of delay-line
    pub fn size(&self) -> usize {
        self.buffer.len()
    }

    /// Get element at back
    pub fn back(&self) -> f32 {
        let idx = self.index_back();
        *self.buffer.index(idx)
    }

    /// Get index of back element.
    pub fn index_back(&self) -> usize {
        let i = self.pos + 1;
        if i < self.size() { i } else { 0 }
    }

    /// Read value at delay i
    pub fn read(&self, i: i32) -> &f32 {
        let mut idx = self.pos as i32 - i;
        if idx < 0 { idx += self.size() as i32; }
        &self.buffer.index(idx as usize)
    }

    /// Write value to delay
    pub fn write(&mut self, value: f32) {
        *self.buffer.index_mut(self.pos) = value;
        self.pos += 1;
        if self.pos >= self.size() { self.pos = 0; }
    }

    /// Write new value and return oldest value
    pub fn get_write_and_step(&mut self, value: f32) -> f32 {
        let r = *self.buffer.index(self.pos);
        self.write(value);
        r
    }

    /// Comb filter input using a delay time equal to the maximum size of the delay-line
    pub fn comb(&mut self, value: f32, feed_fwd: f32, feed_bck: f32) -> f32 {
        let d = *self.buffer.index(self.pos);
        let r = value + d*feed_bck;
        self.write(r);
        d + r*feed_fwd
    }

    /// Allpass filter input using a delay time equal to the maximum size of the delay-line
    pub fn allpass(&mut self, value: f32, feed_fwd: f32) -> f32 {
        self.comb(value, feed_fwd, -feed_fwd)
    }
}


impl<B> Clone for DelayLine<B> where B: Buffer {
    fn clone(&self) -> Self {
        DelayLine {
            pos: self.pos,
            buffer: Buffer::clone(&self.buffer),
        }
    }
}


impl<B> ::std::fmt::Debug for DelayLine<B> where B: Buffer {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "pos: {:?}, buffer: [f32; {:?}]", self.pos, self.buffer.len())
    }
}


/// Some buffer of Float values that is compatible with the delay-line
pub trait Buffer {
    fn zeroed() -> Self;
    fn clone(&self) -> Self;
    fn len(&self) -> usize;
    fn index(&self, idx: usize) -> &f32;
    fn index_mut(&mut self, idx: usize) -> &mut f32;
}
