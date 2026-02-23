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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_version() {
        assert_eq!(RPC_VERSION, 2);
    }

    // --- RpcMsgType ---

    #[test]
    fn test_rpc_msg_type_values() {
        assert_eq!(RpcMsgType::Call as u32, 0);
        assert_eq!(RpcMsgType::Reply as u32, 1);
    }

    #[test]
    fn test_rpc_msg_type_clone_eq() {
        let a = RpcMsgType::Call;
        let b = a;
        assert_eq!(a, b);
    }

    // --- RpcReplyStat ---

    #[test]
    fn test_rpc_reply_stat_values() {
        assert_eq!(RpcReplyStat::MsgAccepted as u32, 0);
        assert_eq!(RpcReplyStat::MsgDenied as u32, 1);
    }

    // --- RpcAcceptStat ---

    #[test]
    fn test_rpc_accept_stat_try_from_valid() {
        assert_eq!(RpcAcceptStat::try_from(0), Ok(RpcAcceptStat::Success));
        assert_eq!(RpcAcceptStat::try_from(1), Ok(RpcAcceptStat::ProgUnavail));
        assert_eq!(RpcAcceptStat::try_from(2), Ok(RpcAcceptStat::ProgMismatch));
        assert_eq!(RpcAcceptStat::try_from(3), Ok(RpcAcceptStat::ProcUnavail));
        assert_eq!(RpcAcceptStat::try_from(4), Ok(RpcAcceptStat::GarbageArgs));
        assert_eq!(RpcAcceptStat::try_from(5), Ok(RpcAcceptStat::SystemErr));
    }

    #[test]
    fn test_rpc_accept_stat_try_from_invalid() {
        assert_eq!(RpcAcceptStat::try_from(6), Err(6));
        assert_eq!(RpcAcceptStat::try_from(100), Err(100));
    }

    // --- RpcRejectStat ---

    #[test]
    fn test_rpc_reject_stat_try_from_valid() {
        assert_eq!(RpcRejectStat::try_from(0), Ok(RpcRejectStat::RpcMismatch));
        assert_eq!(RpcRejectStat::try_from(1), Ok(RpcRejectStat::AuthError));
    }

    #[test]
    fn test_rpc_reject_stat_try_from_invalid() {
        assert_eq!(RpcRejectStat::try_from(2), Err(2));
        assert_eq!(RpcRejectStat::try_from(u32::MAX), Err(u32::MAX));
    }

    // --- RpcAuthStat ---

    #[test]
    fn test_rpc_auth_stat_try_from_all_valid() {
        let expected = [
            (0, RpcAuthStat::AuthOk),
            (1, RpcAuthStat::AuthBadcred),
            (2, RpcAuthStat::AuthRejectedcred),
            (3, RpcAuthStat::AuthBadverf),
            (4, RpcAuthStat::AuthRejectedverf),
            (5, RpcAuthStat::AuthTooweak),
            (6, RpcAuthStat::AuthInvalidresp),
            (7, RpcAuthStat::AuthFailed),
            (8, RpcAuthStat::AuthKerbGeneric),
            (9, RpcAuthStat::AuthTimeexpire),
            (10, RpcAuthStat::AuthTktFile),
            (11, RpcAuthStat::AuthDecode),
            (12, RpcAuthStat::AuthNetAddr),
            (13, RpcAuthStat::RpcsecGssCredproblem),
            (14, RpcAuthStat::RpcsecGssCtxproblem),
        ];
        for (val, variant) in expected {
            assert_eq!(RpcAuthStat::try_from(val), Ok(variant), "failed for {val}");
        }
    }

    #[test]
    fn test_rpc_auth_stat_try_from_invalid() {
        assert_eq!(RpcAuthStat::try_from(15), Err(15));
        assert_eq!(RpcAuthStat::try_from(255), Err(255));
    }

    // --- RpcAuthFlavor ---

    #[test]
    fn test_rpc_auth_flavor_try_from_all_valid() {
        let expected = [
            (0, RpcAuthFlavor::AuthNone),
            (1, RpcAuthFlavor::AuthSys),
            (2, RpcAuthFlavor::AuthShort),
            (3, RpcAuthFlavor::AuthDh),
            (4, RpcAuthFlavor::AuthKerb),
            (5, RpcAuthFlavor::AuthRsa),
            (6, RpcAuthFlavor::RpcsecGss),
        ];
        for (val, variant) in expected {
            assert_eq!(
                RpcAuthFlavor::try_from(val),
                Ok(variant),
                "failed for {val}"
            );
        }
    }

    #[test]
    fn test_rpc_auth_flavor_try_from_invalid() {
        assert_eq!(RpcAuthFlavor::try_from(7), Err(7));
        assert_eq!(RpcAuthFlavor::try_from(1000), Err(1000));
    }

    // --- repr values ---

    #[test]
    fn test_accept_stat_repr_values() {
        assert_eq!(RpcAcceptStat::Success as u32, 0);
        assert_eq!(RpcAcceptStat::SystemErr as u32, 5);
    }

    #[test]
    fn test_auth_stat_repr_values() {
        assert_eq!(RpcAuthStat::AuthOk as u32, 0);
        assert_eq!(RpcAuthStat::RpcsecGssCtxproblem as u32, 14);
    }

    #[test]
    fn test_auth_flavor_repr_values() {
        assert_eq!(RpcAuthFlavor::AuthNone as u32, 0);
        assert_eq!(RpcAuthFlavor::RpcsecGss as u32, 6);
    }

    // --- Debug ---

    #[test]
    fn test_debug_formatting() {
        assert_eq!(format!("{:?}", RpcMsgType::Call), "Call");
        assert_eq!(format!("{:?}", RpcAcceptStat::GarbageArgs), "GarbageArgs");
    }
}
