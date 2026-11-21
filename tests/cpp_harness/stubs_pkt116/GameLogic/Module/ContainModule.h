#pragma once

class ContainModuleInterface {
public:
    Bool m_isTunnel = FALSE;
    Bool isTunnelContain() const { return m_isTunnel; }
};
