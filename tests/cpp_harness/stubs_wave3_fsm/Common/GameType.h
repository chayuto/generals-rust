#pragma once
enum ObjectID : Int { INVALID_ID = 0 };
enum DrawableID : Int { INVALID_DRAWABLE_ID = 0 };
enum KindOfType : Int { KINDOF_INVALID = -1 };
enum ScienceType : Int { SCIENCE_INVALID = -1 };

struct Coord2D { Real x, y; };
struct ICoord2D { Int x, y; };
struct Region2D { Coord2D lo, hi; };
struct IRegion2D { ICoord2D lo, hi; };
struct Coord3D { Real x, y, z; };
struct ICoord3D { Int x, y, z; };
struct Region3D { Coord3D lo, hi; };
struct IRegion3D { ICoord3D lo, hi; };
struct RealRange { Real lo, hi; };
struct RGBColor { Real red, green, blue; };
struct RGBAColorReal { Real red, green, blue, alpha; };
struct RGBAColorInt { UnsignedInt red, green, blue, alpha; };
struct Vector4 { Real X, Y, Z, W; };
class Matrix3D {
public:
    Vector4 Row[3];
    Vector4& operator[](int i) { return Row[i]; }
    const Vector4& operator[](int i) const { return Row[i]; }
};
