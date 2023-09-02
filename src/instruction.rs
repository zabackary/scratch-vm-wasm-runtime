use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u16)]
#[derive(Copy, Clone, Debug)]
pub enum InstructionType {
    Noop = 0x0000,
    ExtraArg = 0x0001,
    LoadConst = 0x0002,
    Load = 0x0003,
    Store = 0x0004,
    Jump = 0x0005,
    JumpIf = 0x0006,
    AllocList = 0x0007,
    OpAdd = 0x0008,
    OpSubtract = 0x0009,
    OpMultiply = 0x000a,
    OpDivide = 0x000b,
    OpAnd = 0x000c,
    OpOr = 0x000d,
    UnaryNot = 0x000e,
    UnaryAbs = 0x000f,
    UnaryFloor = 0x0010,
    UnaryCeil = 0x0011,
    UnarySqrt = 0x0012,
    UnarySin = 0x0013,
    UnaryCos = 0x0014,
    UnaryTan = 0x0015,
    UnaryAsin = 0x0016,
    UnaryAcos = 0x0017,
    UnaryAtan = 0x0018,
    UnaryLn = 0x0019,
    UnaryLog = 0x001a,
    UnaryEPow = 0x001b,
    Unary10Pow = 0x001c,
    OpLt = 0x001d,
    Reserved = 0x001e,
    OpEq = 0x001f,
    ListDel = 0x0020,
    ListIns = 0x0021,
    ListDelAll = 0x0022,
    ListReplace = 0x0023,
    ListPush = 0x0024,
    ListLoad = 0x0025,
    ListLen = 0x0026,
    ListIFind = 0x0027,
    ListIIncludes = 0x0028,
    MonitorShowVar = 0x0029,
    MonitorHideVar = 0x002a,
    MonitorShowList = 0x002b,
    MonitorHideList = 0x002c,
    Return = 0x002d,
}

#[wasm_bindgen]
#[repr(C)]
pub struct Instruction {
    pub name: InstructionType,
    _padding: [u8; 2],
    pub argument: u32,
}