//! `PKT-444`: Command stream, translators, and `CommandList` (`MessageStream.h` / `MessageStream.cpp`).
//!
//! Compiled in Release mode (`RTS_DEBUG`, `_ALLOW_DEBUG_CHEATS_IN_RELEASE`,
//! `ALLOW_SURRENDER`, and `DUMP_PERF_STATS` all disabled).
//!
//! Reference packet: `packets/phase7/PKT-444_MessageStream.md`
//! Golden vectors: `crates/zh_logic/tests/golden_pkt444_messagestream.rs` (`docs/golden_vectors/PKT-444.json`)

#![allow(unused_variables, dead_code)]

use zh_common::math::{Coord3D, ICoord2D, IRegion2D};
use zh_common::types::{Bool, DrawableId, Int, ObjectId, Real, UnsignedByte, UnsignedInt};

pub type TranslatorId = UnsignedInt;
pub const TRANSLATOR_ID_INVALID: Int = -1;
pub const MSG_COUNT_VALUE: Int = 2006;

/// Argument discriminant matching `GameMessageArgumentDataType`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum GameMessageArgumentDataType {
    #[default]
    Integer = 0,
    Real = 1,
    Boolean = 2,
    ObjectId = 3,
    DrawableId = 4,
    TeamId = 5,
    Location = 6,
    Pixel = 7,
    PixelRegion = 8,
    Timestamp = 9,
    WideChar = 10,
    Unknown = 11,
}

/// Safe storage representation mirroring C++ `GameMessageArgumentType` union.
///
/// Default value is all-zero, matching C++ out-of-range `getArgument()` fallback (`m_zero = { 0 }`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GameMessageArgumentValue {
    pub integer: Int,
    pub real: Real,
    pub boolean: Bool,
    pub object_id: ObjectId,
    pub drawable_id: DrawableId,
    pub team_id: UnsignedInt,
    pub location: Coord3D,
    pub pixel: ICoord2D,
    pub pixel_region: IRegion2D,
    pub timestamp: UnsignedInt,
    pub wide_char: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameMessageArgument {
    pub data: GameMessageArgumentValue,
    pub data_type: GameMessageArgumentDataType,
}

/// Release-configuration `GameMessage::Type` enum matching `PKT-444.json` ordinals.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum GameMessageType {
    #[default]
    MsgInvalid = 0,
    MsgFrameTick = 1,
    MsgRawMouseBegin = 2,
    MsgRawMousePosition = 3,
    MsgRawMouseLeftButtonDown = 4,
    MsgRawMouseLeftDoubleClick = 5,
    MsgRawMouseLeftButtonUp = 6,
    MsgRawMouseLeftClick = 7,
    MsgRawMouseLeftDrag = 8,
    MsgRawMouseMiddleButtonDown = 9,
    MsgRawMouseMiddleDoubleClick = 10,
    MsgRawMouseMiddleButtonUp = 11,
    MsgRawMouseMiddleDrag = 12,
    MsgRawMouseRightButtonDown = 13,
    MsgRawMouseRightDoubleClick = 14,
    MsgRawMouseRightButtonUp = 15,
    MsgRawMouseRightDrag = 16,
    MsgRawMouseWheel = 17,
    MsgRawMouseEnd = 18,
    MsgRawKeyDown = 19,
    MsgRawKeyUp = 20,
    MsgMouseLeftClick = 21,
    MsgMouseLeftDoubleClick = 22,
    MsgMouseMiddleClick = 23,
    MsgMouseMiddleDoubleClick = 24,
    MsgMouseRightClick = 25,
    MsgMouseRightDoubleClick = 26,
    MsgClearGameData = 27,
    MsgNewGame = 28,
    MsgLogicCrcPlayback = 29,
    MsgBeginMetaMessages = 30,
    MsgMetaSaveView1 = 31,
    MsgMetaSaveView2 = 32,
    MsgMetaSaveView3 = 33,
    MsgMetaSaveView4 = 34,
    MsgMetaSaveView5 = 35,
    MsgMetaSaveView6 = 36,
    MsgMetaSaveView7 = 37,
    MsgMetaSaveView8 = 38,
    MsgMetaViewView1 = 39,
    MsgMetaViewView2 = 40,
    MsgMetaViewView3 = 41,
    MsgMetaViewView4 = 42,
    MsgMetaViewView5 = 43,
    MsgMetaViewView6 = 44,
    MsgMetaViewView7 = 45,
    MsgMetaViewView8 = 46,
    MsgMetaCreateTeam0 = 47,
    MsgMetaCreateTeam1 = 48,
    MsgMetaCreateTeam2 = 49,
    MsgMetaCreateTeam3 = 50,
    MsgMetaCreateTeam4 = 51,
    MsgMetaCreateTeam5 = 52,
    MsgMetaCreateTeam6 = 53,
    MsgMetaCreateTeam7 = 54,
    MsgMetaCreateTeam8 = 55,
    MsgMetaCreateTeam9 = 56,
    MsgMetaSelectTeam0 = 57,
    MsgMetaSelectTeam1 = 58,
    MsgMetaSelectTeam2 = 59,
    MsgMetaSelectTeam3 = 60,
    MsgMetaSelectTeam4 = 61,
    MsgMetaSelectTeam5 = 62,
    MsgMetaSelectTeam6 = 63,
    MsgMetaSelectTeam7 = 64,
    MsgMetaSelectTeam8 = 65,
    MsgMetaSelectTeam9 = 66,
    MsgMetaAddTeam0 = 67,
    MsgMetaAddTeam1 = 68,
    MsgMetaAddTeam2 = 69,
    MsgMetaAddTeam3 = 70,
    MsgMetaAddTeam4 = 71,
    MsgMetaAddTeam5 = 72,
    MsgMetaAddTeam6 = 73,
    MsgMetaAddTeam7 = 74,
    MsgMetaAddTeam8 = 75,
    MsgMetaAddTeam9 = 76,
    MsgMetaViewTeam0 = 77,
    MsgMetaViewTeam1 = 78,
    MsgMetaViewTeam2 = 79,
    MsgMetaViewTeam3 = 80,
    MsgMetaViewTeam4 = 81,
    MsgMetaViewTeam5 = 82,
    MsgMetaViewTeam6 = 83,
    MsgMetaViewTeam7 = 84,
    MsgMetaViewTeam8 = 85,
    MsgMetaViewTeam9 = 86,
    MsgMetaSelectMatchingUnits = 87,
    MsgMetaSelectNextUnit = 88,
    MsgMetaSelectPrevUnit = 89,
    MsgMetaSelectNextWorker = 90,
    MsgMetaSelectPrevWorker = 91,
    MsgMetaSelectNextIdleWorker = 92,
    MsgMetaViewCommandCenter = 93,
    MsgMetaViewLastRadarEvent = 94,
    MsgMetaSelectHero = 95,
    MsgMetaSelectAll = 96,
    MsgMetaSelectAllAircraft = 97,
    MsgMetaScatter = 98,
    MsgMetaStop = 99,
    MsgMetaDeploy = 100,
    MsgMetaCreateFormation = 101,
    MsgMetaFollow = 102,
    MsgMetaChatPlayers = 103,
    MsgMetaChatAllies = 104,
    MsgMetaChatEveryone = 105,
    MsgMetaDiplomacy = 106,
    MsgMetaOptions = 107,
    MsgMetaIncreaseMaxRenderFps = 108,
    MsgMetaDecreaseMaxRenderFps = 109,
    MsgMetaIncreaseLogicTimeScale = 110,
    MsgMetaDecreaseLogicTimeScale = 111,
    MsgMetaToggleLowerDetails = 112,
    MsgMetaToggleControlBar = 113,
    MsgMetaTogglePlayerObserver = 114,
    MsgMetaBeginPathBuild = 115,
    MsgMetaEndPathBuild = 116,
    MsgMetaBeginForceattack = 117,
    MsgMetaEndForceattack = 118,
    MsgMetaBeginForcemove = 119,
    MsgMetaEndForcemove = 120,
    MsgMetaBeginWaypoints = 121,
    MsgMetaEndWaypoints = 122,
    MsgMetaBeginPreferSelection = 123,
    MsgMetaEndPreferSelection = 124,
    MsgMetaTakeScreenshot = 125,
    MsgMetaTakeScreenshotPng = 126,
    MsgMetaAllCheer = 127,
    MsgMetaToggleAttackmove = 128,
    MsgMetaBeginCameraRotateLeft = 129,
    MsgMetaEndCameraRotateLeft = 130,
    MsgMetaAltCameraRotateLeft = 131,
    MsgMetaBeginCameraRotateRight = 132,
    MsgMetaEndCameraRotateRight = 133,
    MsgMetaAltCameraRotateRight = 134,
    MsgMetaBeginCameraZoomIn = 135,
    MsgMetaEndCameraZoomIn = 136,
    MsgMetaBeginCameraZoomOut = 137,
    MsgMetaEndCameraZoomOut = 138,
    MsgMetaCameraReset = 139,
    MsgMetaToggleCameraTrackingDrawable = 140,
    MsgMetaToggleFastForwardReplay = 141,
    MsgMetaTogglePause = 142,
    MsgMetaTogglePauseAlt = 143,
    MsgMetaStepFrame = 144,
    MsgMetaStepFrameAlt = 145,
    MsgMetaDemoInstantQuit = 146,
    MsgMetaPlaceBeacon = 147,
    MsgMetaRemoveBeacon = 148,
    MsgEndMetaMessages = 149,
    MsgMouseoverDrawableHint = 150,
    MsgMouseoverLocationHint = 151,
    MsgValidGuicommandHint = 152,
    MsgInvalidGuicommandHint = 153,
    MsgBeginAreaSelectionHint = 154,
    MsgEndAreaSelectionHint = 155,
    MsgDoAttackObjectHint = 156,
    MsgImpossibleAttackHint = 157,
    MsgDoForceAttackObjectHint = 158,
    MsgDoForceAttackGroundHint = 159,
    MsgGetRepairedHint = 160,
    MsgGetHealedHint = 161,
    MsgDoRepairHint = 162,
    MsgResumeConstructionHint = 163,
    MsgEnterHint = 164,
    MsgDockHint = 165,
    MsgDoMovetoHint = 166,
    MsgDoAttackmovetoHint = 167,
    MsgAddWaypointHint = 168,
    MsgHijackHint = 169,
    MsgSabotageHint = 170,
    MsgFirebombHint = 171,
    MsgConvertToCarbombHint = 172,
    MsgCapturebuildingHint = 173,
    MsgSnipeVehicleHint = 174,
    MsgDefectorHint = 175,
    MsgSetRallyPointHint = 176,
    MsgDoSpecialPowerOverrideDestinationHint = 177,
    MsgDoSalvageHint = 178,
    MsgDoInvalidHint = 179,
    MsgDoAttackObjectAfterMovingHint = 180,
    MsgHackHint = 181,
    MsgBeginNetworkMessages = 1000,
    MsgCreateSelectedGroup = 1001,
    MsgCreateSelectedGroupNoSound = 1002,
    MsgDestroySelectedGroup = 1003,
    MsgRemoveFromSelectedGroup = 1004,
    MsgSelectedGroupCommand = 1005,
    MsgCreateTeam0 = 1006,
    MsgCreateTeam1 = 1007,
    MsgCreateTeam2 = 1008,
    MsgCreateTeam3 = 1009,
    MsgCreateTeam4 = 1010,
    MsgCreateTeam5 = 1011,
    MsgCreateTeam6 = 1012,
    MsgCreateTeam7 = 1013,
    MsgCreateTeam8 = 1014,
    MsgCreateTeam9 = 1015,
    MsgSelectTeam0 = 1016,
    MsgSelectTeam1 = 1017,
    MsgSelectTeam2 = 1018,
    MsgSelectTeam3 = 1019,
    MsgSelectTeam4 = 1020,
    MsgSelectTeam5 = 1021,
    MsgSelectTeam6 = 1022,
    MsgSelectTeam7 = 1023,
    MsgSelectTeam8 = 1024,
    MsgSelectTeam9 = 1025,
    MsgAddTeam0 = 1026,
    MsgAddTeam1 = 1027,
    MsgAddTeam2 = 1028,
    MsgAddTeam3 = 1029,
    MsgAddTeam4 = 1030,
    MsgAddTeam5 = 1031,
    MsgAddTeam6 = 1032,
    MsgAddTeam7 = 1033,
    MsgAddTeam8 = 1034,
    MsgAddTeam9 = 1035,
    MsgDoAttacksquad = 1036,
    MsgDoWeapon = 1037,
    MsgDoWeaponAtLocation = 1038,
    MsgDoWeaponAtObject = 1039,
    MsgDoSpecialPower = 1040,
    MsgDoSpecialPowerAtLocation = 1041,
    MsgDoSpecialPowerAtObject = 1042,
    MsgSetRallyPoint = 1043,
    MsgPurchaseScience = 1044,
    MsgQueueUpgrade = 1045,
    MsgCancelUpgrade = 1046,
    MsgQueueUnitCreate = 1047,
    MsgCancelUnitCreate = 1048,
    MsgDozerConstruct = 1049,
    MsgDozerConstructLine = 1050,
    MsgDozerCancelConstruct = 1051,
    MsgSell = 1052,
    MsgExit = 1053,
    MsgEvacuate = 1054,
    MsgExecuteRailedTransport = 1055,
    MsgCombatdropAtLocation = 1056,
    MsgCombatdropAtObject = 1057,
    MsgAreaSelectionDeprecated = 1058,
    MsgDoAttackObject = 1059,
    MsgDoForceAttackObject = 1060,
    MsgDoForceAttackGround = 1061,
    MsgGetRepaired = 1062,
    MsgGetHealed = 1063,
    MsgDoRepair = 1064,
    MsgResumeConstruction = 1065,
    MsgEnter = 1066,
    MsgDock = 1067,
    MsgDoMoveto = 1068,
    MsgDoAttackmoveto = 1069,
    MsgDoForcemoveto = 1070,
    MsgAddWaypoint = 1071,
    MsgDoGuardPosition = 1072,
    MsgDoGuardObject = 1073,
    MsgDoStop = 1074,
    MsgDoScatter = 1075,
    MsgInternetHack = 1076,
    MsgDoCheer = 1077,
    MsgToggleOvercharge = 1078,
    MsgSwitchWeapons = 1079,
    MsgConvertToCarbomb = 1080,
    MsgCapturebuilding = 1081,
    MsgDisablevehicleHack = 1082,
    MsgStealcashHack = 1083,
    MsgDisablebuildingHack = 1084,
    MsgSnipeVehicle = 1085,
    MsgDoSpecialPowerOverrideDestination = 1086,
    MsgDoSalvage = 1087,
    MsgClearIngamePopupMessage = 1088,
    MsgPlaceBeacon = 1089,
    MsgRemoveBeacon = 1090,
    MsgSetBeaconText = 1091,
    MsgSetReplayCamera = 1092,
    MsgSelfDestruct = 1093,
    MsgCreateFormation = 1094,
    MsgLogicCrc = 1095,
    MsgSetMineClearingDetail = 1096,
    MsgEnableRetaliationMode = 1097,
    MsgBeginDebugNetworkMessages = 1900,
    MsgEndNetworkMessages = 1999,
    MsgTimestamp = 2000,
    MsgObjectCreated = 2001,
    MsgObjectDestroyed = 2002,
    MsgObjectPosition = 2003,
    MsgObjectOrientation = 2004,
    MsgObjectJoinedTeam = 2005,
    MsgCount = 2006,
}

impl GameMessageType {
    pub fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::MsgInvalid),
            1 => Some(Self::MsgFrameTick),
            2 => Some(Self::MsgRawMouseBegin),
            3 => Some(Self::MsgRawMousePosition),
            4 => Some(Self::MsgRawMouseLeftButtonDown),
            5 => Some(Self::MsgRawMouseLeftDoubleClick),
            6 => Some(Self::MsgRawMouseLeftButtonUp),
            7 => Some(Self::MsgRawMouseLeftClick),
            8 => Some(Self::MsgRawMouseLeftDrag),
            9 => Some(Self::MsgRawMouseMiddleButtonDown),
            10 => Some(Self::MsgRawMouseMiddleDoubleClick),
            11 => Some(Self::MsgRawMouseMiddleButtonUp),
            12 => Some(Self::MsgRawMouseMiddleDrag),
            13 => Some(Self::MsgRawMouseRightButtonDown),
            14 => Some(Self::MsgRawMouseRightDoubleClick),
            15 => Some(Self::MsgRawMouseRightButtonUp),
            16 => Some(Self::MsgRawMouseRightDrag),
            17 => Some(Self::MsgRawMouseWheel),
            18 => Some(Self::MsgRawMouseEnd),
            19 => Some(Self::MsgRawKeyDown),
            20 => Some(Self::MsgRawKeyUp),
            21 => Some(Self::MsgMouseLeftClick),
            22 => Some(Self::MsgMouseLeftDoubleClick),
            23 => Some(Self::MsgMouseMiddleClick),
            24 => Some(Self::MsgMouseMiddleDoubleClick),
            25 => Some(Self::MsgMouseRightClick),
            26 => Some(Self::MsgMouseRightDoubleClick),
            27 => Some(Self::MsgClearGameData),
            28 => Some(Self::MsgNewGame),
            29 => Some(Self::MsgLogicCrcPlayback),
            30 => Some(Self::MsgBeginMetaMessages),
            31 => Some(Self::MsgMetaSaveView1),
            32 => Some(Self::MsgMetaSaveView2),
            33 => Some(Self::MsgMetaSaveView3),
            34 => Some(Self::MsgMetaSaveView4),
            35 => Some(Self::MsgMetaSaveView5),
            36 => Some(Self::MsgMetaSaveView6),
            37 => Some(Self::MsgMetaSaveView7),
            38 => Some(Self::MsgMetaSaveView8),
            39 => Some(Self::MsgMetaViewView1),
            40 => Some(Self::MsgMetaViewView2),
            41 => Some(Self::MsgMetaViewView3),
            42 => Some(Self::MsgMetaViewView4),
            43 => Some(Self::MsgMetaViewView5),
            44 => Some(Self::MsgMetaViewView6),
            45 => Some(Self::MsgMetaViewView7),
            46 => Some(Self::MsgMetaViewView8),
            47 => Some(Self::MsgMetaCreateTeam0),
            48 => Some(Self::MsgMetaCreateTeam1),
            49 => Some(Self::MsgMetaCreateTeam2),
            50 => Some(Self::MsgMetaCreateTeam3),
            51 => Some(Self::MsgMetaCreateTeam4),
            52 => Some(Self::MsgMetaCreateTeam5),
            53 => Some(Self::MsgMetaCreateTeam6),
            54 => Some(Self::MsgMetaCreateTeam7),
            55 => Some(Self::MsgMetaCreateTeam8),
            56 => Some(Self::MsgMetaCreateTeam9),
            57 => Some(Self::MsgMetaSelectTeam0),
            58 => Some(Self::MsgMetaSelectTeam1),
            59 => Some(Self::MsgMetaSelectTeam2),
            60 => Some(Self::MsgMetaSelectTeam3),
            61 => Some(Self::MsgMetaSelectTeam4),
            62 => Some(Self::MsgMetaSelectTeam5),
            63 => Some(Self::MsgMetaSelectTeam6),
            64 => Some(Self::MsgMetaSelectTeam7),
            65 => Some(Self::MsgMetaSelectTeam8),
            66 => Some(Self::MsgMetaSelectTeam9),
            67 => Some(Self::MsgMetaAddTeam0),
            68 => Some(Self::MsgMetaAddTeam1),
            69 => Some(Self::MsgMetaAddTeam2),
            70 => Some(Self::MsgMetaAddTeam3),
            71 => Some(Self::MsgMetaAddTeam4),
            72 => Some(Self::MsgMetaAddTeam5),
            73 => Some(Self::MsgMetaAddTeam6),
            74 => Some(Self::MsgMetaAddTeam7),
            75 => Some(Self::MsgMetaAddTeam8),
            76 => Some(Self::MsgMetaAddTeam9),
            77 => Some(Self::MsgMetaViewTeam0),
            78 => Some(Self::MsgMetaViewTeam1),
            79 => Some(Self::MsgMetaViewTeam2),
            80 => Some(Self::MsgMetaViewTeam3),
            81 => Some(Self::MsgMetaViewTeam4),
            82 => Some(Self::MsgMetaViewTeam5),
            83 => Some(Self::MsgMetaViewTeam6),
            84 => Some(Self::MsgMetaViewTeam7),
            85 => Some(Self::MsgMetaViewTeam8),
            86 => Some(Self::MsgMetaViewTeam9),
            87 => Some(Self::MsgMetaSelectMatchingUnits),
            88 => Some(Self::MsgMetaSelectNextUnit),
            89 => Some(Self::MsgMetaSelectPrevUnit),
            90 => Some(Self::MsgMetaSelectNextWorker),
            91 => Some(Self::MsgMetaSelectPrevWorker),
            92 => Some(Self::MsgMetaSelectNextIdleWorker),
            93 => Some(Self::MsgMetaViewCommandCenter),
            94 => Some(Self::MsgMetaViewLastRadarEvent),
            95 => Some(Self::MsgMetaSelectHero),
            96 => Some(Self::MsgMetaSelectAll),
            97 => Some(Self::MsgMetaSelectAllAircraft),
            98 => Some(Self::MsgMetaScatter),
            99 => Some(Self::MsgMetaStop),
            100 => Some(Self::MsgMetaDeploy),
            101 => Some(Self::MsgMetaCreateFormation),
            102 => Some(Self::MsgMetaFollow),
            103 => Some(Self::MsgMetaChatPlayers),
            104 => Some(Self::MsgMetaChatAllies),
            105 => Some(Self::MsgMetaChatEveryone),
            106 => Some(Self::MsgMetaDiplomacy),
            107 => Some(Self::MsgMetaOptions),
            108 => Some(Self::MsgMetaIncreaseMaxRenderFps),
            109 => Some(Self::MsgMetaDecreaseMaxRenderFps),
            110 => Some(Self::MsgMetaIncreaseLogicTimeScale),
            111 => Some(Self::MsgMetaDecreaseLogicTimeScale),
            112 => Some(Self::MsgMetaToggleLowerDetails),
            113 => Some(Self::MsgMetaToggleControlBar),
            114 => Some(Self::MsgMetaTogglePlayerObserver),
            115 => Some(Self::MsgMetaBeginPathBuild),
            116 => Some(Self::MsgMetaEndPathBuild),
            117 => Some(Self::MsgMetaBeginForceattack),
            118 => Some(Self::MsgMetaEndForceattack),
            119 => Some(Self::MsgMetaBeginForcemove),
            120 => Some(Self::MsgMetaEndForcemove),
            121 => Some(Self::MsgMetaBeginWaypoints),
            122 => Some(Self::MsgMetaEndWaypoints),
            123 => Some(Self::MsgMetaBeginPreferSelection),
            124 => Some(Self::MsgMetaEndPreferSelection),
            125 => Some(Self::MsgMetaTakeScreenshot),
            126 => Some(Self::MsgMetaTakeScreenshotPng),
            127 => Some(Self::MsgMetaAllCheer),
            128 => Some(Self::MsgMetaToggleAttackmove),
            129 => Some(Self::MsgMetaBeginCameraRotateLeft),
            130 => Some(Self::MsgMetaEndCameraRotateLeft),
            131 => Some(Self::MsgMetaAltCameraRotateLeft),
            132 => Some(Self::MsgMetaBeginCameraRotateRight),
            133 => Some(Self::MsgMetaEndCameraRotateRight),
            134 => Some(Self::MsgMetaAltCameraRotateRight),
            135 => Some(Self::MsgMetaBeginCameraZoomIn),
            136 => Some(Self::MsgMetaEndCameraZoomIn),
            137 => Some(Self::MsgMetaBeginCameraZoomOut),
            138 => Some(Self::MsgMetaEndCameraZoomOut),
            139 => Some(Self::MsgMetaCameraReset),
            140 => Some(Self::MsgMetaToggleCameraTrackingDrawable),
            141 => Some(Self::MsgMetaToggleFastForwardReplay),
            142 => Some(Self::MsgMetaTogglePause),
            143 => Some(Self::MsgMetaTogglePauseAlt),
            144 => Some(Self::MsgMetaStepFrame),
            145 => Some(Self::MsgMetaStepFrameAlt),
            146 => Some(Self::MsgMetaDemoInstantQuit),
            147 => Some(Self::MsgMetaPlaceBeacon),
            148 => Some(Self::MsgMetaRemoveBeacon),
            149 => Some(Self::MsgEndMetaMessages),
            150 => Some(Self::MsgMouseoverDrawableHint),
            151 => Some(Self::MsgMouseoverLocationHint),
            152 => Some(Self::MsgValidGuicommandHint),
            153 => Some(Self::MsgInvalidGuicommandHint),
            154 => Some(Self::MsgBeginAreaSelectionHint),
            155 => Some(Self::MsgEndAreaSelectionHint),
            156 => Some(Self::MsgDoAttackObjectHint),
            157 => Some(Self::MsgImpossibleAttackHint),
            158 => Some(Self::MsgDoForceAttackObjectHint),
            159 => Some(Self::MsgDoForceAttackGroundHint),
            160 => Some(Self::MsgGetRepairedHint),
            161 => Some(Self::MsgGetHealedHint),
            162 => Some(Self::MsgDoRepairHint),
            163 => Some(Self::MsgResumeConstructionHint),
            164 => Some(Self::MsgEnterHint),
            165 => Some(Self::MsgDockHint),
            166 => Some(Self::MsgDoMovetoHint),
            167 => Some(Self::MsgDoAttackmovetoHint),
            168 => Some(Self::MsgAddWaypointHint),
            169 => Some(Self::MsgHijackHint),
            170 => Some(Self::MsgSabotageHint),
            171 => Some(Self::MsgFirebombHint),
            172 => Some(Self::MsgConvertToCarbombHint),
            173 => Some(Self::MsgCapturebuildingHint),
            174 => Some(Self::MsgSnipeVehicleHint),
            175 => Some(Self::MsgDefectorHint),
            176 => Some(Self::MsgSetRallyPointHint),
            177 => Some(Self::MsgDoSpecialPowerOverrideDestinationHint),
            178 => Some(Self::MsgDoSalvageHint),
            179 => Some(Self::MsgDoInvalidHint),
            180 => Some(Self::MsgDoAttackObjectAfterMovingHint),
            181 => Some(Self::MsgHackHint),
            1000 => Some(Self::MsgBeginNetworkMessages),
            1001 => Some(Self::MsgCreateSelectedGroup),
            1002 => Some(Self::MsgCreateSelectedGroupNoSound),
            1003 => Some(Self::MsgDestroySelectedGroup),
            1004 => Some(Self::MsgRemoveFromSelectedGroup),
            1005 => Some(Self::MsgSelectedGroupCommand),
            1006 => Some(Self::MsgCreateTeam0),
            1007 => Some(Self::MsgCreateTeam1),
            1008 => Some(Self::MsgCreateTeam2),
            1009 => Some(Self::MsgCreateTeam3),
            1010 => Some(Self::MsgCreateTeam4),
            1011 => Some(Self::MsgCreateTeam5),
            1012 => Some(Self::MsgCreateTeam6),
            1013 => Some(Self::MsgCreateTeam7),
            1014 => Some(Self::MsgCreateTeam8),
            1015 => Some(Self::MsgCreateTeam9),
            1016 => Some(Self::MsgSelectTeam0),
            1017 => Some(Self::MsgSelectTeam1),
            1018 => Some(Self::MsgSelectTeam2),
            1019 => Some(Self::MsgSelectTeam3),
            1020 => Some(Self::MsgSelectTeam4),
            1021 => Some(Self::MsgSelectTeam5),
            1022 => Some(Self::MsgSelectTeam6),
            1023 => Some(Self::MsgSelectTeam7),
            1024 => Some(Self::MsgSelectTeam8),
            1025 => Some(Self::MsgSelectTeam9),
            1026 => Some(Self::MsgAddTeam0),
            1027 => Some(Self::MsgAddTeam1),
            1028 => Some(Self::MsgAddTeam2),
            1029 => Some(Self::MsgAddTeam3),
            1030 => Some(Self::MsgAddTeam4),
            1031 => Some(Self::MsgAddTeam5),
            1032 => Some(Self::MsgAddTeam6),
            1033 => Some(Self::MsgAddTeam7),
            1034 => Some(Self::MsgAddTeam8),
            1035 => Some(Self::MsgAddTeam9),
            1036 => Some(Self::MsgDoAttacksquad),
            1037 => Some(Self::MsgDoWeapon),
            1038 => Some(Self::MsgDoWeaponAtLocation),
            1039 => Some(Self::MsgDoWeaponAtObject),
            1040 => Some(Self::MsgDoSpecialPower),
            1041 => Some(Self::MsgDoSpecialPowerAtLocation),
            1042 => Some(Self::MsgDoSpecialPowerAtObject),
            1043 => Some(Self::MsgSetRallyPoint),
            1044 => Some(Self::MsgPurchaseScience),
            1045 => Some(Self::MsgQueueUpgrade),
            1046 => Some(Self::MsgCancelUpgrade),
            1047 => Some(Self::MsgQueueUnitCreate),
            1048 => Some(Self::MsgCancelUnitCreate),
            1049 => Some(Self::MsgDozerConstruct),
            1050 => Some(Self::MsgDozerConstructLine),
            1051 => Some(Self::MsgDozerCancelConstruct),
            1052 => Some(Self::MsgSell),
            1053 => Some(Self::MsgExit),
            1054 => Some(Self::MsgEvacuate),
            1055 => Some(Self::MsgExecuteRailedTransport),
            1056 => Some(Self::MsgCombatdropAtLocation),
            1057 => Some(Self::MsgCombatdropAtObject),
            1058 => Some(Self::MsgAreaSelectionDeprecated),
            1059 => Some(Self::MsgDoAttackObject),
            1060 => Some(Self::MsgDoForceAttackObject),
            1061 => Some(Self::MsgDoForceAttackGround),
            1062 => Some(Self::MsgGetRepaired),
            1063 => Some(Self::MsgGetHealed),
            1064 => Some(Self::MsgDoRepair),
            1065 => Some(Self::MsgResumeConstruction),
            1066 => Some(Self::MsgEnter),
            1067 => Some(Self::MsgDock),
            1068 => Some(Self::MsgDoMoveto),
            1069 => Some(Self::MsgDoAttackmoveto),
            1070 => Some(Self::MsgDoForcemoveto),
            1071 => Some(Self::MsgAddWaypoint),
            1072 => Some(Self::MsgDoGuardPosition),
            1073 => Some(Self::MsgDoGuardObject),
            1074 => Some(Self::MsgDoStop),
            1075 => Some(Self::MsgDoScatter),
            1076 => Some(Self::MsgInternetHack),
            1077 => Some(Self::MsgDoCheer),
            1078 => Some(Self::MsgToggleOvercharge),
            1079 => Some(Self::MsgSwitchWeapons),
            1080 => Some(Self::MsgConvertToCarbomb),
            1081 => Some(Self::MsgCapturebuilding),
            1082 => Some(Self::MsgDisablevehicleHack),
            1083 => Some(Self::MsgStealcashHack),
            1084 => Some(Self::MsgDisablebuildingHack),
            1085 => Some(Self::MsgSnipeVehicle),
            1086 => Some(Self::MsgDoSpecialPowerOverrideDestination),
            1087 => Some(Self::MsgDoSalvage),
            1088 => Some(Self::MsgClearIngamePopupMessage),
            1089 => Some(Self::MsgPlaceBeacon),
            1090 => Some(Self::MsgRemoveBeacon),
            1091 => Some(Self::MsgSetBeaconText),
            1092 => Some(Self::MsgSetReplayCamera),
            1093 => Some(Self::MsgSelfDestruct),
            1094 => Some(Self::MsgCreateFormation),
            1095 => Some(Self::MsgLogicCrc),
            1096 => Some(Self::MsgSetMineClearingDetail),
            1097 => Some(Self::MsgEnableRetaliationMode),
            1900 => Some(Self::MsgBeginDebugNetworkMessages),
            1999 => Some(Self::MsgEndNetworkMessages),
            2000 => Some(Self::MsgTimestamp),
            2001 => Some(Self::MsgObjectCreated),
            2002 => Some(Self::MsgObjectDestroyed),
            2003 => Some(Self::MsgObjectPosition),
            2004 => Some(Self::MsgObjectOrientation),
            2005 => Some(Self::MsgObjectJoinedTeam),
            2006 => Some(Self::MsgCount),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Stable handle to a message inside a `GameMessageList`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MessageId(pub u64);

/// Single queued input/command message (`GameMessage`).
#[derive(Debug, Clone, PartialEq)]
pub struct GameMessage {
    id: MessageId,
    msg_type: GameMessageType,
    player_index: Int,
    args: Vec<GameMessageArgument>,
}

impl GameMessage {
    /// Creates a new `GameMessage` originating from `player_index`.
    pub fn new(msg_type: GameMessageType, player_index: Int) -> Self {
        Self {
            id: MessageId(0),
            msg_type,
            player_index,
            args: Vec::new(),
        }
    }

    #[inline]
    pub const fn id(&self) -> MessageId {
        self.id
    }

    #[inline]
    pub const fn message_type(&self) -> GameMessageType {
        self.msg_type
    }

    #[inline]
    pub const fn player_index(&self) -> Int {
        self.player_index
    }

    pub fn get_command_as_string(&self) -> &'static str {
        Self::get_command_type_as_string(self.msg_type)
    }

    pub fn get_command_type_as_string(t: GameMessageType) -> &'static str {
        Self::get_command_type_as_string_raw(t.as_i32())
    }

    pub fn get_command_type_as_string_raw(raw: Int) -> &'static str {
        todo!("PKT-444: implement GameMessage::getCommandTypeAsString")
    }

    pub fn append_integer_argument(&mut self, arg: Int) {
        todo!("PKT-444: implement GameMessage::appendIntegerArgument")
    }

    pub fn append_real_argument(&mut self, arg: Real) {
        todo!("PKT-444: implement GameMessage::appendRealArgument")
    }

    pub fn append_boolean_argument(&mut self, arg: Bool) {
        todo!("PKT-444: implement GameMessage::appendBooleanArgument")
    }

    pub fn append_drawable_id_argument(&mut self, arg: DrawableId) {
        todo!("PKT-444: implement GameMessage::appendDrawableIDArgument")
    }

    pub fn append_object_id_argument(&mut self, arg: ObjectId) {
        todo!("PKT-444: implement GameMessage::appendObjectIDArgument")
    }

    pub fn append_team_id_argument(&mut self, arg: UnsignedInt) {
        todo!("PKT-444: implement GameMessage::appendTeamIDArgument")
    }

    pub fn append_location_argument(&mut self, arg: Coord3D) {
        todo!("PKT-444: implement GameMessage::appendLocationArgument")
    }

    pub fn append_pixel_argument(&mut self, arg: ICoord2D) {
        todo!("PKT-444: implement GameMessage::appendPixelArgument")
    }

    pub fn append_pixel_region_argument(&mut self, arg: IRegion2D) {
        todo!("PKT-444: implement GameMessage::appendPixelRegionArgument")
    }

    pub fn append_timestamp_argument(&mut self, arg: UnsignedInt) {
        todo!("PKT-444: implement GameMessage::appendTimestampArgument")
    }

    pub fn append_wide_char_argument(&mut self, arg: u16) {
        todo!("PKT-444: implement GameMessage::appendWideCharArgument")
    }

    pub fn get_argument_count(&self) -> UnsignedByte {
        todo!("PKT-444: implement GameMessage::getArgumentCount")
    }

    pub fn get_argument(&self, arg_index: Int) -> GameMessageArgumentValue {
        todo!("PKT-444: implement GameMessage::getArgument")
    }

    pub fn get_argument_data_type(&self, arg_index: Int) -> GameMessageArgumentDataType {
        todo!("PKT-444: implement GameMessage::getArgumentDataType")
    }
}

/// Doubly-ordered message container shared by `MessageStream` and `CommandList`.
#[derive(Debug, Clone, Default)]
pub struct GameMessageList {
    messages: Vec<GameMessage>,
    next_id: u64,
}

impl GameMessageList {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            next_id: 1,
        }
    }

    pub fn init(&mut self) {}
    pub fn reset(&mut self) {
        todo!("PKT-444: implement GameMessageList::reset")
    }
    pub fn update(&mut self) {}

    pub fn append_message(&mut self, msg: GameMessage) -> MessageId {
        todo!("PKT-444: implement GameMessageList::appendMessage")
    }

    pub fn insert_message_after(
        &mut self,
        msg: GameMessage,
        after_id: MessageId,
    ) -> Option<MessageId> {
        todo!("PKT-444: implement GameMessageList::insertMessage")
    }

    pub fn remove_message(&mut self, id: MessageId) -> Option<GameMessage> {
        todo!("PKT-444: implement GameMessageList::removeMessage")
    }

    pub fn contains_message_of_type(&self, msg_type: GameMessageType) -> bool {
        todo!("PKT-444: implement GameMessageList::containsMessageOfType")
    }

    pub fn messages(&self) -> &[GameMessage] {
        &self.messages
    }

    pub fn get_message_mut(&mut self, id: MessageId) -> Option<&mut GameMessage> {
        self.messages.iter_mut().find(|m| m.id == id)
    }
}

/// Surviving commands handed from `MessageStream::propagate_messages` to `GameLogic`.
#[derive(Debug, Clone, Default)]
pub struct CommandList {
    list: GameMessageList,
}

impl CommandList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(&mut self) {}
    pub fn reset(&mut self) {
        todo!("PKT-444: implement CommandList::reset")
    }
    pub fn update(&mut self) {}

    pub fn append_message_list(&mut self, msgs: Vec<GameMessage>) {
        todo!("PKT-444: implement CommandList::appendMessageList")
    }

    pub fn messages(&self) -> &[GameMessage] {
        self.list.messages()
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMessageDisposition {
    KeepMessage = 0,
    DestroyMessage = 1,
}

/// Translator callback attached to a `MessageStream`.
pub trait GameMessageTranslator {
    fn translate_game_message(&mut self, msg: &GameMessage) -> GameMessageDisposition;

    /// Optional type-erased inspection hook (defaults to `None` so callers need not implement it).
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
}

struct TranslatorEntry {
    id: TranslatorId,
    translator: Box<dyn GameMessageTranslator>,
    priority: UnsignedInt,
}

/// Prioritized pipeline of translators filtering messages before `CommandList`.
pub struct MessageStream {
    list: GameMessageList,
    translators: Vec<TranslatorEntry>,
    next_translator_id: TranslatorId,
}

impl Default for MessageStream {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageStream {
    pub fn new() -> Self {
        Self {
            list: GameMessageList::new(),
            translators: Vec::new(),
            next_translator_id: 1,
        }
    }

    pub fn init(&mut self) {}

    pub fn reset(&mut self) {
        todo!("PKT-444: implement MessageStream::reset")
    }

    pub fn update(&mut self) {}

    pub fn is_ready_for_messages(&self, has_local_player: bool) -> bool {
        has_local_player
    }

    pub fn append_message(&mut self, msg_type: GameMessageType, player_index: Int) -> MessageId {
        todo!("PKT-444: implement MessageStream::appendMessage")
    }

    pub fn insert_message_after(
        &mut self,
        msg_type: GameMessageType,
        player_index: Int,
        after_id: MessageId,
    ) -> Option<MessageId> {
        todo!("PKT-444: implement MessageStream::insertMessage")
    }

    pub fn get_message_mut(&mut self, id: MessageId) -> Option<&mut GameMessage> {
        self.list.get_message_mut(id)
    }

    pub fn contains_message_of_type(&self, msg_type: GameMessageType) -> bool {
        todo!("PKT-444: implement MessageStream::containsMessageOfType")
    }

    pub fn attach_translator(
        &mut self,
        translator: Box<dyn GameMessageTranslator>,
        priority: UnsignedInt,
    ) -> TranslatorId {
        todo!("PKT-444: implement MessageStream::attachTranslator (strict > priority)")
    }

    pub fn find_translator(&self, id: TranslatorId) -> Option<&dyn GameMessageTranslator> {
        todo!("PKT-444: implement MessageStream::findTranslator")
    }

    pub fn find_translator_mut(
        &mut self,
        id: TranslatorId,
    ) -> Option<&mut dyn GameMessageTranslator> {
        todo!("PKT-444: implement MessageStream::findTranslator mut")
    }

    pub fn remove_translator(&mut self, id: TranslatorId) {
        todo!("PKT-444: implement MessageStream::removeTranslator")
    }

    /// Returns attached translator IDs in current dispatch priority order.
    pub fn translator_ids(&self) -> Vec<TranslatorId> {
        self.translators.iter().map(|t| t.id).collect()
    }

    pub fn propagate_messages(&mut self, command_list: &mut CommandList) {
        todo!("PKT-444: implement MessageStream::propagateMessages + redundancy pruning")
    }

    pub fn is_redundant_network_message(slice: &[GameMessage], index: usize) -> bool {
        todo!("PKT-444: implement MessageStream::isRedundantNetworkMessage")
    }
}

/// Normalizes two 2D drag endpoints into an ordered bounding rectangle (`buildRegion`).
pub fn build_region(anchor: ICoord2D, dest: ICoord2D) -> IRegion2D {
    todo!("PKT-444: implement buildRegion")
}
