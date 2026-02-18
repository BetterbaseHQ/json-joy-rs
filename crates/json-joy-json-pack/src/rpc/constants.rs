//! ONC RPC protocol constants.
//!
//! Upstream reference: `json-pack/src/rpc/constants.ts`
//! References: RFC 1057, RFC 1831, RFC 5531

pub const RPC_VERSION: u32 = 2;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpcMsgType {
    Call = 0,
    Reply = 1,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpcReplyStat {
    MsgAccepted = 0,
    MsgDenied = 1,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpcAcceptStat {
    Success = 0,
    ProgUnavail = 1,
    ProgMismatch = 2,
    ProcUnavail = 3,
    GarbageArgs = 4,
    SystemErr = 5,
}

impl TryFrom<u32> for RpcAcceptStat {
    type Error = u32;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Success),
            1 => Ok(Self::ProgUnavail),
            2 => Ok(Self::ProgMismatch),
            3 => Ok(Self::ProcUnavail),
            4 => Ok(Self::GarbageArgs),
            5 => Ok(Self::SystemErr),
            other => Err(other),
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpcRejectStat {
    RpcMismatch = 0,
    AuthError = 1,
}

impl TryFrom<u32> for RpcRejectStat {
    type Error = u32;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::RpcMismatch),
            1 => Ok(Self::AuthError),
            other => Err(other),
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpcAuthStat {
    AuthOk = 0,
    AuthBadcred = 1,
    AuthRejectedcred = 2,
    AuthBadverf = 3,
    AuthRejectedverf = 4,
    AuthTooweak = 5,
    AuthInvalidresp = 6,
    AuthFailed = 7,
    AuthKerbGeneric = 8,
    AuthTimeexpire = 9,
    AuthTktFile = 10,
    AuthDecode = 11,
    AuthNetAddr = 12,
    RpcsecGssCredproblem = 13,
    RpcsecGssCtxproblem = 14,
}

impl TryFrom<u32> for RpcAuthStat {
    type Error = u32;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::AuthOk),
            1 => Ok(Self::AuthBadcred),
            2 => Ok(Self::AuthRejectedcred),
            3 => Ok(Self::AuthBadverf),
            4 => Ok(Self::AuthRejectedverf),
            5 => Ok(Self::AuthTooweak),
            6 => Ok(Self::AuthInvalidresp),
            7 => Ok(Self::AuthFailed),
            8 => Ok(Self::AuthKerbGeneric),
            9 => Ok(Self::AuthTimeexpire),
            10 => Ok(Self::AuthTktFile),
            11 => Ok(Self::AuthDecode),
            12 => Ok(Self::AuthNetAddr),
            13 => Ok(Self::RpcsecGssCredproblem),
            14 => Ok(Self::RpcsecGssCtxproblem),
            other => Err(other),
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpcAuthFlavor {
    AuthNone = 0,
    AuthSys = 1,
    AuthShort = 2,
    AuthDh = 3,
    AuthKerb = 4,
    AuthRsa = 5,
    RpcsecGss = 6,
}

impl TryFrom<u32> for RpcAuthFlavor {
    type Error = u32;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::AuthNone),
            1 => Ok(Self::AuthSys),
            2 => Ok(Self::AuthShort),
            3 => Ok(Self::AuthDh),
            4 => Ok(Self::AuthKerb),
            5 => Ok(Self::AuthRsa),
            6 => Ok(Self::RpcsecGss),
            other => Err(other),
        }
    }
}
