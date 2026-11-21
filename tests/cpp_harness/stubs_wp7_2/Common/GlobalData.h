#pragma once
#include "Common/AsciiString.h"

class GlobalData {
public:
    AsciiString m_assetPath;
    AsciiString m_generalsAssetPath;
    AsciiString m_modBIG;
    AsciiString m_modDir;
    Bool m_looseFilesWin = FALSE;
};

extern GlobalData* TheGlobalData;
