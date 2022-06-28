use std::ffi::CString;

use bitflags::bitflags;
use deku::prelude::*;

use helper::*;

mod helper {
    pub use bstr::BString;
    use deku::{
        ctx::{Limit, Size},
        prelude::*,
    };

    /// Take a uleb128 from stream, then takes a subslice of that size,
    /// and try to parse it to T if the uleb128 is not zero.
    #[derive(Debug)]
    pub enum Lengthed<T> {
        None,
        Some(T),
    }

    impl<'a, T, Ctx> DekuRead<'a, Ctx> for Lengthed<T>
    where
        T: DekuRead<'a, Ctx>,
    {
        fn read(
            input: &'a deku::bitvec::BitSlice<deku::bitvec::Msb0, u8>,
            ctx: Ctx,
        ) -> Result<(&'a deku::bitvec::BitSlice<deku::bitvec::Msb0, u8>, Self), DekuError>
        where
            Self: Sized,
        {
            let (rest, len) = Uleb128::read(input, ())?;
            let len = len.0 as usize;

            if len == 0 {
                return Ok((rest, Lengthed::None));
            }

            let bits_len = Size::Bytes(len).bit_size();
            if bits_len > rest.len() {
                return Err(DekuError::Parse(format!(
                    "no enough data (`{}` vs {})",
                    len,
                    rest.len()
                )));
            }

            let (bytes, rest) = rest.split_at(bits_len);
            let (_, t) = T::read(bytes, ctx)?;

            Ok((rest, Lengthed::Some(t)))
        }
    }

    impl<T, Ctx> DekuWrite<Ctx> for Lengthed<T>
    where
        T: DekuWrite<Ctx>,
        Ctx: Copy,
    {
        fn write(
            &self,
            output: &mut deku::bitvec::BitVec<deku::bitvec::Msb0, u8>,
            ctx: Ctx,
        ) -> Result<(), DekuError> {
            match self {
                Lengthed::None => Uleb128(0).write(output, ()),
                Lengthed::Some(val) => {
                    let old_len = output.len();

                    T::write(val, output, ctx)?;

                    let new_len = output.len();
                    output.truncate(old_len);

                    Uleb128((new_len - old_len) as _).write(output, ())?;
                    T::write(val, output, ctx)
                }
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Uleb128(pub u32);

    impl<'a> DekuRead<'a> for Uleb128 {
        fn read(
            mut input: &'a deku::bitvec::BitSlice<deku::bitvec::Msb0, u8>,
            _: (),
        ) -> Result<(&'a deku::bitvec::BitSlice<deku::bitvec::Msb0, u8>, Self), DekuError>
        where
            Self: Sized,
        {
            const CONTINUATION_BIT: u8 = 1 << 7;

            let mut val = 0;
            let mut shift = 0;

            let effective_bits_of = |byte| byte & !CONTINUATION_BIT;
            let not_continue = |byte| byte & CONTINUATION_BIT == 0;

            loop {
                let (i2, mut b) = u8::read(input, ())?;
                input = i2;

                if shift == 28 && b > 0b1111 {
                    // Consume all
                    while b & CONTINUATION_BIT != 0 {
                        let (i2, b2) = u8::read(input, ())?;
                        input = i2;
                        b = b2;
                    }
                    return Err(DekuError::Parse(
                        "integer overflow(bigger than u32)".to_owned(),
                    ));
                }

                let effective_bits = effective_bits_of(b) as u32;
                val |= effective_bits << shift;

                if not_continue(b) {
                    return Ok((input, Uleb128(val)));
                }

                shift += 7;
            }
        }
    }

    impl DekuWrite for Uleb128 {
        fn write(
            &self,
            output: &mut deku::bitvec::BitVec<deku::bitvec::Msb0, u8>,
            _: (),
        ) -> Result<(), DekuError> {
            todo!()
        }
    }

    #[derive(Debug)]
    pub struct Uleb128_33(pub u32);
    impl<'a> DekuRead<'a> for Uleb128_33 {
        fn read(
            i: &'a deku::bitvec::BitSlice<deku::bitvec::Msb0, u8>,
            _: (),
        ) -> Result<(&'a deku::bitvec::BitSlice<deku::bitvec::Msb0, u8>, Self), DekuError>
        where
            Self: Sized,
        {
            const CONTINUATION_BIT: u8 = 1 << 7;
            const FIRST_CONTINUATION_BIT: u8 = 1 << 6;

            let effective_bits_of = |byte| byte & !CONTINUATION_BIT;
            let not_continue = |byte| byte & CONTINUATION_BIT == 0;

            let (mut i, b) = u8::read(i, ())?;
            let b = b >> 1;

            let mut val = b as u32 & 0x3f;
            if b & FIRST_CONTINUATION_BIT == 0 {
                return Ok((i, Uleb128_33(val)));
            }

            let mut shift = 6;

            loop {
                let (i2, mut b) = u8::read(i, ())?;
                i = i2;

                if shift == 27 && b > 0b11111 {
                    // Consume all
                    while b & CONTINUATION_BIT != 0 {
                        let (i2, b2) = u8::read(i, ())?;
                        i = i2;
                        b = b2;
                    }
                    return Err(DekuError::Parse(
                        "integer overflow(bigger than u32)".to_owned(),
                    ));
                }
                let effective_bits = effective_bits_of(b) as u32;
                val |= effective_bits << shift;

                if not_continue(b) {
                    return Ok((i, Uleb128_33(val)));
                }

                shift += 7;
            }
        }
    }

    impl DekuWrite for Uleb128_33 {
        fn write(
            &self,
            output: &mut deku::bitvec::BitVec<deku::bitvec::Msb0, u8>,
            ctx: (),
        ) -> Result<(), DekuError> {
            todo!()
        }
    }

    /// A `BString` that starts with it's length plus `N`
    #[derive(Debug)]
    pub struct LenString<const N: usize = 0>(pub BString);

    impl<'a, const N: usize> DekuRead<'a> for LenString<N> {
        fn read(
            input: &'a deku::bitvec::BitSlice<deku::bitvec::Msb0, u8>,
            ctx: (),
        ) -> Result<(&'a deku::bitvec::BitSlice<deku::bitvec::Msb0, u8>, Self), DekuError>
        where
            Self: Sized,
        {
            let (rest, len) = Uleb128::read(input, ctx)?;
            let (rest, bytes) = <&[u8]>::read(rest, (Limit::new_count((len.0 as usize) - N), ()))?;
            Ok((rest, LenString(BString::from(bytes))))
        }
    }

    impl<const N: usize> DekuWrite for LenString<N> {
        fn write(
            &self,
            output: &mut deku::bitvec::BitVec<deku::bitvec::Msb0, u8>,
            ctx: (),
        ) -> Result<(), DekuError> {
            let len = self.0.len() + N;
            Uleb128(len as _).write(output, ctx)?;
            <&[u8]>::write(&self.0.as_ref(), output, ())
        }
    }

    #[derive(Debug, DekuRead, DekuWrite)]
    #[deku(id = "line_num", ctx = "line_num: u32")]
    pub enum VariantInteger {
        #[deku(id_pat = "0..=255")]
        U8(u8),
        #[deku(id_pat = "256..=65535")]
        U16(u16),
        #[deku(id_pat = "_")]
        U32(u32),
    }
}

bitflags! {
    pub struct DumpFlags: u32 {
        const IS_BIG_ENDIAN = 0b00000001;
        const IS_STRIPPED = 0b00000010;
        const HAS_FFI = 0b00000100;
    }
}

impl<'a> DekuRead<'a> for DumpFlags {
    fn read(
        input: &'a deku::bitvec::BitSlice<deku::bitvec::Msb0, u8>,
        ctx: (),
    ) -> Result<(&'a deku::bitvec::BitSlice<deku::bitvec::Msb0, u8>, Self), DekuError>
    where
        Self: Sized,
    {
        let (rest, n) = Uleb128::read(input, ctx)?;
        let flags = DumpFlags::from_bits(n.0).ok_or_else(|| {
            DekuError::Parse(format!("the value(`{}`) does not correspond a flag", n.0))
        })?;
        Ok((rest, flags))
    }
}

impl DekuWrite for DumpFlags {
    fn write(
        &self,
        output: &mut deku::bitvec::BitVec<deku::bitvec::Msb0, u8>,
        ctx: (),
    ) -> Result<(), DekuError> {
        Uleb128(self.bits).write(output, ctx)
    }
}

bitflags! {
    pub struct PtFlags: u8 {
        const CHILD = 0b00000001;
        const VARIADIC = 0b00000010;
        const FFI = 0b00000100;
        const NO_JIT = 0b00001000;
        const HAS_ILOOP = 0b00010000;
    }
}

impl<'a> DekuRead<'a> for PtFlags {
    fn read(
        input: &'a deku::bitvec::BitSlice<deku::bitvec::Msb0, u8>,
        ctx: (),
    ) -> Result<(&'a deku::bitvec::BitSlice<deku::bitvec::Msb0, u8>, Self), DekuError>
    where
        Self: Sized,
    {
        let (rest, n) = u8::read(input, ctx)?;
        let flags = PtFlags::from_bits(n).ok_or_else(|| {
            DekuError::Parse(format!("the value(`{}`) does not correspond a flag", n))
        })?;
        Ok((rest, flags))
    }
}

impl DekuWrite for PtFlags {
    fn write(
        &self,
        output: &mut deku::bitvec::BitVec<deku::bitvec::Msb0, u8>,
        ctx: (),
    ) -> Result<(), DekuError> {
        self.bits.write(output, ctx)
    }
}

/// `[0x1b, 0x4c, 0x4a]`
#[derive(Debug, DekuRead, DekuWrite)]
pub struct Magic(#[deku(assert_eq = "[0x1b, 0x4c, 0x4a]")] [u8; 3]);

#[derive(Debug, DekuRead, DekuWrite)]
pub struct Dump {
    pub magic: Magic,
    pub version: u8,
    pub flags: DumpFlags,
    #[deku(cond = "!flags.contains(DumpFlags::IS_STRIPPED)")]
    pub filename: Option<LenString>,
    #[deku(until = "|p: &Lengthed<Prototype>| matches!(p, Lengthed::None)", ctx = "*flags")]
    pub prototypes: Vec<Lengthed<Prototype>>,
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(ctx = "dump_flags: DumpFlags")]
pub struct Prototype {
    pub flags: PtFlags,
    pub params_num: u8,
    pub frame_size: u8,
    pub upvalues_num: u8,
    pub kgc_num: Uleb128,
    pub kn_num: Uleb128,
    pub inst_num: Uleb128,
    #[deku(cond = "!dump_flags.contains(DumpFlags::IS_STRIPPED)")]
    pub debug_header: Option<DebugHeader>,
    #[deku(count = "inst_num.0 as usize")]
    pub instructions: Vec<Instruction>,
    #[deku(count = "usize::from(*upvalues_num)")]
    pub upvalues: Vec<Upvalue>,
    #[deku(count = "kgc_num.0 as usize")]
    pub constant_gc: Vec<ConstantGc>,
    #[deku(count = "kn_num.0 as usize")]
    pub constant_numbers: Vec<ConstantNumber>,
    #[deku(
        cond = "!dump_flags.contains(DumpFlags::IS_STRIPPED)",
        ctx = "inst_num.0 as usize, debug_header.unwrap(), *upvalues_num"
    )]
    pub debug_info: Option<DebugInfo>,
}

#[derive(Debug, DekuRead, DekuWrite)]
pub struct Instruction(u32);

#[derive(Debug, DekuRead, DekuWrite)]
pub struct Upvalue(u16);

/// Correspounds to the represent of the `number` type in LuaJIT.
///
/// See LuaJIT's source code for more information
#[derive(Debug, Clone, Copy, DekuRead, DekuWrite)]
pub struct Number(Uleb128, Uleb128);

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(type = "Uleb128")]
pub enum ConstantGc {
    #[deku(id = "Uleb128(0)")]
    Fn,
    #[deku(id = "Uleb128(1)")]
    Table(Table),
    #[deku(id = "Uleb128(2)")]
    Signed(Number),
    #[deku(id = "Uleb128(3)")]
    Unsigned(Number),
    #[deku(id = "Uleb128(4)")]
    Complex { re: Number, im: Number },
    #[deku(id_pat = "_")]
    Str(LenString<5>),
}

#[derive(Debug, DekuRead, DekuWrite)]
pub struct Table {
    array_num: Uleb128,
    hash_num: Uleb128,
    #[deku(count = "array_num.0 as usize")]
    pub array: Vec<TableItem>,
    // Keep the insert order
    #[deku(count = "hash_num.0 as usize")]
    pub hash: Vec<(TableItem, TableItem)>,
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(type = "Uleb128")]
pub enum TableItem {
    #[deku(id = "Uleb128(0)")]
    Nil,
    #[deku(id = "Uleb128(1)")]
    False,
    #[deku(id = "Uleb128(2)")]
    True,
    #[deku(id = "Uleb128(3)")]
    Int(Uleb128),
    #[deku(id = "Uleb128(4)")]
    Num(Number),
    #[deku(id_pat = "_")]
    Str(LenString<5>),
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum ConstantNumber {
    #[deku(id_pat = "n if n & 0b0000_0001 == 0")]
    Integer(Uleb128_33),
    #[deku(id_pat = "_")]
    Number(Uleb128_33, Uleb128),
}

#[derive(Debug, Clone, Copy, DekuRead, DekuWrite)]
pub struct DebugHeader {
    size: Uleb128,
    first_line: Uleb128,
    line_num: Uleb128,
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(ctx = "inst_num: usize, header: DebugHeader, upvalues_num: u8")]
pub struct DebugInfo {
    #[deku(ctx = "(header.line_num.0, inst_num)")]
    pub line_info: LineInfo,
    #[deku(count = "upvalues_num")]
    pub upvalue_names: Vec<CString>,
    #[deku(until = "|var: &VarInfoWrapper| matches!(var, VarInfoWrapper::None)")]
    pub var_info: Vec<VarInfoWrapper>,
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum VarInfoWrapper {
    #[deku(id = "0x00")]
    None,
    #[deku(id_pat = "_")]
    Some(VarInfo),
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum VarKind {
    #[deku(id = "0x01")]
    ForIndex,
    #[deku(id = "0x02")]
    ForStop,
    #[deku(id = "0x03")]
    ForStep,
    #[deku(id = "0x04")]
    ForGeneraor,
    #[deku(id = "0x05")]
    ForState,
    #[deku(id = "0x06")]
    ForControl,
    #[deku(id_pat = "_")]
    Name(CString),
}

#[derive(Debug, DekuRead, DekuWrite)]
pub struct VarInfo {
    pub kind: VarKind,
    pub gap: Uleb128,
    pub range: Uleb128,
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(ctx = "line_num: u32, inst_num: usize")]
pub struct LineInfo {
    #[deku(count = "inst_num", ctx = "line_num")]
    pub map: Vec<VariantInteger>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use deku::DekuContainerRead;

    #[test]
    fn it_should_read_properly() {
        let bytecode = std::fs::read("D:\\dm\\res-decoder\\temp\\src\\bytecodes\\Sys.lua").unwrap();
        let (_, dump) = Dump::from_bytes((&bytecode, 0)).unwrap();
        let bytes = dump.to_bytes().unwrap();
        Dump::from_bytes((&bytes, 0)).unwrap();
    }
}
