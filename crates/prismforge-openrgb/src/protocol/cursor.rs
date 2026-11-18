use prismforge_core::Rgb;

use super::controller::{ControllerDataError, StringProblem};

const COLOR_LEN: usize = 4;

pub struct Cursor<'a> {
    bytes: &'a [u8],
}

impl<'a> Cursor<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn remaining(&self) -> usize {
        self.bytes.len()
    }

    pub fn take(
        &mut self,
        len: usize,
        field: &'static str,
    ) -> Result<&'a [u8], ControllerDataError> {
        if len > self.bytes.len() {
            return Err(ControllerDataError::Truncated { field });
        }
        let (head, tail) = self.bytes.split_at(len);
        self.bytes = tail;
        Ok(head)
    }

    pub fn u16(&mut self, field: &'static str) -> Result<u16, ControllerDataError> {
        let bytes = self.take(2, field)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub fn u32(&mut self, field: &'static str) -> Result<u32, ControllerDataError> {
        let bytes = self.take(4, field)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn i32(&mut self, field: &'static str) -> Result<i32, ControllerDataError> {
        Ok(self.u32(field)? as i32)
    }

    pub fn string(&mut self, field: &'static str) -> Result<String, ControllerDataError> {
        let len = self.u16(field)?;
        self.terminated_string(len as usize, field)
    }

    pub fn long_string(&mut self, field: &'static str) -> Result<String, ControllerDataError> {
        let len = self.u32(field)?;
        self.terminated_string(len as usize, field)
    }

    pub fn colors(&mut self, field: &'static str) -> Result<Vec<Rgb>, ControllerDataError> {
        let count = self.u16(field)? as usize;
        let bytes = self.take(count * COLOR_LEN, field)?;
        Ok(bytes
            .as_chunks::<COLOR_LEN>()
            .0
            .iter()
            .map(|[r, g, b, _]| Rgb::new(*r, *g, *b))
            .collect())
    }

    fn terminated_string(
        &mut self,
        len: usize,
        field: &'static str,
    ) -> Result<String, ControllerDataError> {
        let bytes = self.take(len, field)?;
        let invalid = |problem| ControllerDataError::InvalidString { field, problem };
        let Some((&terminator, text)) = bytes.split_last() else {
            return Err(invalid(StringProblem::Empty));
        };
        if terminator != 0 {
            return Err(invalid(StringProblem::Unterminated));
        }
        if text.contains(&0) {
            return Err(invalid(StringProblem::InteriorNul));
        }
        String::from_utf8(text.to_vec()).map_err(|_| invalid(StringProblem::InvalidUtf8))
    }
}
