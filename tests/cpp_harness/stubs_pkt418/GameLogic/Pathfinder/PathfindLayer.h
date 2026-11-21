#pragma once

class PathfindLayer {
public:
    Int m_zone = 0;
    Bool m_unused = TRUE;
    Bool m_destroyed = FALSE;
    ICoord2D m_startCell{0, 0};
    ICoord2D m_endCell{0, 0};
    Int m_appliedZone = 0;

    Int getZone() const { return m_zone; }
    void setZone(Int zone) { m_zone = zone; }
    void applyZone() { m_appliedZone = m_zone; }
    Bool isUnused() const { return m_unused; }
    Bool isDestroyed() const { return m_destroyed; }
    void getStartCellIndex(ICoord2D* start) const { *start = m_startCell; }
    void getEndCellIndex(ICoord2D* end) const { *end = m_endCell; }
};
