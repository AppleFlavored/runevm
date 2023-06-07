use crate::ParsingError;

pub trait FromData: Sized {
    const SIZE: usize;

    fn parse(data: &[u8]) -> Option<Self>;
}

pub trait FromSeries<'a>: Sized {
    fn parse(stream: &'a mut Stream, count: u16) -> Result<Self, ParsingError>;
}

pub trait FromSlice<'a>: Sized {
    fn parse(stream: &'a mut Stream) -> Result<Self, ParsingError>;
}

impl FromData for u8 {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        data.get(0).copied()
    }
}

impl FromData for u16 {
    const SIZE: usize = 2;

    fn parse(data: &[u8]) -> Option<Self> {
        data.try_into().ok().map(u16::from_be_bytes)
    }
}

impl FromData for u32 {
    const SIZE: usize = 4;

    fn parse(data: &[u8]) -> Option<Self> {
        data.try_into().ok().map(u32::from_be_bytes)
    }
}

pub struct Stream<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Stream<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Stream { data, offset: 0 }
    }

    pub fn skip<T: FromData>(&mut self) {
        self.advance(T::SIZE);
    }

    pub fn advance(&mut self, len: usize) {
        self.offset += len;
    }

    pub fn read<T: FromData>(&mut self) -> Option<T> {
        self.read_bytes(T::SIZE).and_then(T::parse)
    }

    pub fn read_element<'b, T: FromSlice<'b>>(&'b mut self) -> Result<T, ParsingError> {
        T::parse(self)
    }

    pub fn read_array<'b, T: FromSeries<'b>>(&'b mut self, count: u16) -> Result<T, ParsingError> {
        T::parse(self, count)
    }

    pub fn read_bytes(&mut self, len: usize) -> Option<&'a [u8]> {
        let slice = self.data.get(self.offset..self.offset + len)?;
        self.advance(len);
        Some(slice)
    }
}
