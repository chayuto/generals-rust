# Translation Packet `PKT-016`: `BezierSegment`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-016`
- **Work Package:** `WP-2.2` (Phase 2)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 1: C-ABI Leaf
- **Allowed Rust Output File:** `crates/zh_common/src/math/bezier_segment.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_common/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (5):**
  - `Common/BezFwdIterator.h`
  - `Common/BezierSegment.h`
  - `Common/STLTypedefs.h`
  - `Lib/BaseDefines.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.

---

## Compact Worker Cheat Sheet (Embedded in All Translation Packets)

### 1. Primitive Type Mapping
| C++ Type | Rust Type | Size | Notes |
|---|---|---:|---|
| `Int` | `i32` | 4 | Two's-complement 32-bit signed |
| `UnsignedInt` | `u32` | 4 | 32-bit unsigned |
| `Short` | `i16` | 2 | 16-bit signed |
| `UnsignedShort` | `u16` | 2 | 16-bit unsigned |
| `Byte` / `char` | `i8` | 1 | Signed 8-bit (`char` is signed on x86/macOS) |
| `UnsignedByte` | `u8` | 1 | Unsigned 8-bit |
| `Bool` | `u8` (`Bool`) | 1 | `FALSE = 0`, `TRUE = 1` (1 byte in `Xfer`!) |
| `Real` | `f32` | 4 | IEEE-754 binary32, round-to-nearest-even, no FMA |
| `ObjectID` / `TeamID` / `DrawableID` | `ObjectId(pub i32)` etc. | 4 | `INVALID_ID = 0` |
| `NameKeyType` | `NameKeyType(pub i32)` | 4 | `NAMEKEY_INVALID = 0` |
| `UpdateSleepTime` | `UpdateSleepTime(pub u32)` | 4 | `NONE = 1`, `FOREVER = 0x3fff_ffff` |

### 2. Ten Non-Negotiable Determinism Rules
1. **Wrapping Integer Arithmetic:** Never use bare `+`, `-`, `*`, `+=`, `-=`, `*=` on `i32`/`u32` where C++ can overflow or wrap. Use `.wrapping_add()`, `.wrapping_sub()`, `.wrapping_mul()`, `.wrapping_neg()`.
2. **Unfused IEEE-754 `f32` Arithmetic:** Never call `f32::mul_add` (FMA). Keep exact C++ parenthesization and left-to-right operator grouping: `(a * b) * c + d` must not be rewritten as `a * (b * c) + d`.
3. **Deterministic Transcendentals via `GameMath`:** Never call standard `f32::sin`, `cos`, `tan`, `asin`, `acos`, `atan2`, `sqrt`, `floor`, `ceil`, `powf`, `exp`, or `ln`. Always call `zh_common::gamemath::{gm_sinf, gm_cosf, gm_tanf, gm_asinf, gm_acosf, gm_atan2f, gm_sqrtf, gm_floorf, gm_ceilf, gm_powf, gm_expf, gm_logf}`.
4. **Mixed `Int * Real` Promotion & Float-to-Int Truncation:**
   - In C++, `Int a; Real s; a *= s;` evaluates `a = static_cast<Int>(static_cast<Real>(a) * s)`.
   - In Rust, always write: `a = ((a as f32) * s) as i32;` (or `zh_common::math::cpp_mul_i32_f32(a, s)`).
   - For `REAL_TO_INT_FLOOR(x)` and `REAL_TO_INT_CEIL(x)`, call `zh_common::math::{real_to_int_floor, real_to_int_ceil}` (which route to `gm_floorf(x) as i32` / `gm_ceilf(x) as i32` in default deterministic mode, and `fast_float_floor`/`ceil` under `#[cfg(feature = "retail-crc")]`).
5. **No `HashMap` or `HashSet`:** Never import `std::collections::{HashMap, HashSet}`. Always use `std::collections::{BTreeMap, BTreeSet}` keyed by a deterministic integer ID (`ObjectId`, `NameKeyType`, `TeamId`, `u32`), never by raw pointer address.
6. **No `BinaryHeap` for Sleepy Updates:** Use `zh_logic::SleepyUpdateHeap` (strict `>` comparator, left-child tie-breaking, in-place root sifting, reverse-heap-index removal).
7. **Evaluation Order of Side-Effecting Arguments:** When a C++ function call has 2+ arguments that call `GameLogicRandomValue*` or mutate state, evaluate them into explicit `let arg0 = ...; let arg1 = ...;` bindings in left-to-right order before the call.
8. **Exact PRNG Call Parity:** `GameLogicRandomValue(lo, hi)` and `GameLogicRandomValueReal(lo, hi)` return `hi` **without advancing the seed** when `lo >= hi` (under default `RETAIL_COMPATIBLE_CRC == 0`). Never reorder, hoist, or short-circuit a branch that calls `GameLogicRandomValue*`.
9. **Exact `Xfer` & `XferCRC` Field Order and Widths:**
   - Serialize fields in the **exact** C++ `crc(Xfer*)` and `xfer(Xfer*)` order.
   - Never serialize `usize` or `isize`; cast lengths to `u16`, `u32`, or `i32` matching the C++ local variable type (`UnsignedShort`, `UnsignedInt`, `Int`).
   - Remember `Bool` is `1 byte` (`u8`), and enum values serialized via `xferUser(&e, sizeof(Enum))` are `4 bytes` (`i32`).
10. **Preserve `#if !RETAIL_COMPATIBLE_CRC` Guards:** Translate `#if RETAIL_COMPATIBLE_CRC` / `#if !RETAIL_COMPATIBLE_CRC` blocks using `#[cfg(feature = "retail-crc")]` and `#[cfg(not(feature = "retail-crc"))]`.

---

## 3. Reference C++ Source (`fbraz3/GeneralsX` at `84a7a20`)

### `Core/GameEngine/Include/Common/BezierSegment.h`
```cpp
// BezierSegment.h ////////////////////////////////////////////////////////////////////////////////
// John K McDonald, Jr.
// September 2002
// DO NOT DISTRIBUTE

#pragma once

#ifdef SAGE_USE_GLM
#include <glm/glm.hpp>
#elif defined(_WIN32)
#include <d3dx8math.h>
#else
#error "Missing a math library"
#endif
#include "Common/STLTypedefs.h"
#include "Lib/BaseDefines.h"

#define USUAL_TOLERANCE 1.0f

#ifdef SAGE_USE_GLM
// GeneralsX @bugfix Copilot 15/08/2026 Match D3DX operation ordering for deterministic Bezier trajectories.
class BezierMath
{
public:
	static glm::vec4 GLMVec4Transform(const glm::vec4& v, const glm::mat4& m)
	{
#if USE_DETERMINISTIC_MATH
		return glm::vec4(
			((v.x * m[0][0] + v.y * m[1][0]) + v.z * m[2][0]) + v.w * m[3][0],
			((v.x * m[0][1] + v.y * m[1][1]) + v.z * m[2][1]) + v.w * m[3][1],
			((v.x * m[0][2] + v.y * m[1][2]) + v.z * m[2][2]) + v.w * m[3][2],
			((v.x * m[0][3] + v.y * m[1][3]) + v.z * m[2][3]) + v.w * m[3][3]);
#else
		return m * v;
#endif
	}

	static float GLMVec4Dot(const glm::vec4& a, const glm::vec4& b)
	{
#if USE_DETERMINISTIC_MATH
		return ((a.x * b.x + a.y * b.y) + a.z * b.z) + a.w * b.w;
#else
		return glm::dot(a, b);
#endif
	}
};
#else
class BezierMath
{
public:
	static void D3DXVec4Transform(D3DXVECTOR4* out, const D3DXVECTOR4* v, const D3DXMATRIX* m)
	{
#if USE_DETERMINISTIC_MATH
		const float x = v->x, y = v->y, z = v->z, w = v->w;
		out->x = ((x * m->m[0][0] + y * m->m[1][0]) + z * m->m[2][0]) + w * m->m[3][0];
		out->y = ((x * m->m[0][1] + y * m->m[1][1]) + z * m->m[2][1]) + w * m->m[3][1];
		out->z = ((x * m->m[0][2] + y * m->m[1][2]) + z * m->m[2][2]) + w * m->m[3][2];
		out->w = ((x * m->m[0][3] + y * m->m[1][3]) + z * m->m[2][3]) + w * m->m[3][3];
#else
		::D3DXVec4Transform(out, v, m);
#endif
	}

	static float D3DXVec4Dot(const D3DXVECTOR4* a, const D3DXVECTOR4* b)
	{
#if USE_DETERMINISTIC_MATH
		return ((a->x * b->x + a->y * b->y) + a->z * b->z) + a->w * b->w;
#else
		return ::D3DXVec4Dot(a, b);
#endif
	}
};
#endif

class BezierSegment
{
	protected:
#ifndef SAGE_USE_GLM
		static const D3DXMATRIX s_bezBasisMatrix;
#else // SAGE_USE_GLM
		static const glm::mat4 s_bezBasisMatrix;
#endif
		Coord3D m_controlPoints[4];

	public:	// Constructors
		BezierSegment();
		BezierSegment(Real x0, Real y0, Real z0,
									Real x1, Real y1, Real z1,
									Real x2, Real y2, Real z2,
									Real x3, Real y3, Real z3);

		BezierSegment(Real cp[12]);


		BezierSegment(const Coord3D& cp0,
									const Coord3D& cp1,
									const Coord3D& cp2,
									const Coord3D& cp3);

		BezierSegment(Coord3D cp[4]);

	public:
		void evaluateBezSegmentAtT(Real tValue, Coord3D *outResult) const;
		void getSegmentPoints(Int numSegments, VecCoord3D *outResult) const;
		Real getApproximateLength(Real withinTolerance = USUAL_TOLERANCE) const;

		void splitSegmentAtT(Real tValue, BezierSegment &outSeg1, BezierSegment &outSeg2) const;

	public:	// He get's friendly access.
		friend class BezFwdIterator;
};
```

### `Core/GameEngine/Source/Common/Bezier/BezierSegment.cpp`
```cpp
#include "PreRTS.h"
#include "Common/BezierSegment.h"
#include "Common/BezFwdIterator.h"

#ifdef _WIN32
#include <d3dx8math.h>
#elif defined(SAGE_USE_GLM)
#include <glm/glm.hpp>
#else
#error "Missing a math library"
#endif

//-------------------------------------------------------------------------------------------------
BezierSegment::BezierSegment()
{
	for(int i=0; i < 4; i++)
		m_controlPoints[i].zero();
}

//-------------------------------------------------------------------------------------------------
BezierSegment::BezierSegment(Real x0, Real y0, Real z0,
														 Real x1, Real y1, Real z1,
														 Real x2, Real y2, Real z2,
														 Real x3, Real y3, Real z3)
{
	m_controlPoints[0].x = x0;
	m_controlPoints[0].y = y0;
	m_controlPoints[0].z = z0;

	m_controlPoints[1].x = x1;
	m_controlPoints[1].y = y1;
	m_controlPoints[1].z = z1;

	m_controlPoints[2].x = x2;
	m_controlPoints[2].y = y2;
	m_controlPoints[2].z = z2;

	m_controlPoints[3].x = x3;
	m_controlPoints[3].y = y3;
	m_controlPoints[3].z = z3;
}

BezierSegment::BezierSegment(Real cp[12])
{
	m_controlPoints[0].x = cp[0];
	m_controlPoints[0].y = cp[1];
	m_controlPoints[0].z = cp[2];

	m_controlPoints[1].x = cp[3];
	m_controlPoints[1].y = cp[4];
	m_controlPoints[1].z = cp[5];

	m_controlPoints[2].x = cp[6];
	m_controlPoints[2].y = cp[7];
	m_controlPoints[2].z = cp[8];

	m_controlPoints[3].x = cp[9];
	m_controlPoints[3].y = cp[10];
	m_controlPoints[3].z = cp[11];
}

BezierSegment::BezierSegment(const Coord3D& cp0, const Coord3D& cp1, const Coord3D& cp2, const Coord3D& cp3)
{
	m_controlPoints[0] = cp0;
	m_controlPoints[1] = cp1;
	m_controlPoints[2] = cp2;
	m_controlPoints[3] = cp3;
}

BezierSegment::BezierSegment(Coord3D cp[4])
{
	m_controlPoints[0] = cp[0];
	m_controlPoints[1] = cp[1];
	m_controlPoints[2] = cp[2];
	m_controlPoints[3] = cp[3];
}


//-------------------------------------------------------------------------------------------------
void BezierSegment::evaluateBezSegmentAtT(Real tValue, Coord3D *outResult) const

{
	if (!outResult)
		return;

#ifndef SAGE_USE_GLM
	D3DXVECTOR4	tVec(tValue * tValue * tValue, tValue * tValue, tValue, 1);

	D3DXVECTOR4 xCoords(m_controlPoints[0].x, m_controlPoints[1].x, m_controlPoints[2].x, m_controlPoints[3].x);
	D3DXVECTOR4 yCoords(m_controlPoints[0].y, m_controlPoints[1].y, m_controlPoints[2].y, m_controlPoints[3].y);
	D3DXVECTOR4 zCoords(m_controlPoints[0].z, m_controlPoints[1].z, m_controlPoints[2].z, m_controlPoints[3].z);

	D3DXVECTOR4 tResult;
	BezierMath::D3DXVec4Transform(&tResult, &tVec, &BezierSegment::s_bezBasisMatrix);

	outResult->x = BezierMath::D3DXVec4Dot(&xCoords, &tResult);
	outResult->y = BezierMath::D3DXVec4Dot(&yCoords, &tResult);
	outResult->z = BezierMath::D3DXVec4Dot(&zCoords, &tResult);
#else // SAGE_USE_GLM
	glm::vec4 tVec(tValue * tValue * tValue, tValue * tValue, tValue, 1);

	glm::vec4 xCoords(m_controlPoints[0].x, m_controlPoints[1].x, m_controlPoints[2].x, m_controlPoints[3].x);
	glm::vec4 yCoords(m_controlPoints[0].y, m_controlPoints[1].y, m_controlPoints[2].y, m_controlPoints[3].y);
	glm::vec4 zCoords(m_controlPoints[0].z, m_controlPoints[1].z, m_controlPoints[2].z, m_controlPoints[3].z);

	// GeneralsX @bugfix Copilot 26/08/2026 Route GLM Bezier evaluation through deterministic transform and dot helpers.
	glm::vec4 tResult = BezierMath::GLMVec4Transform(tVec, BezierSegment::s_bezBasisMatrix);
	
	outResult->x = BezierMath::GLMVec4Dot(xCoords, tResult);
	outResult->y = BezierMath::GLMVec4Dot(yCoords, tResult);
	outResult->z = BezierMath::GLMVec4Dot(zCoords, tResult);
#endif
}

//-------------------------------------------------------------------------------------------------
void BezierSegment::getSegmentPoints(Int numSegments, VecCoord3D *outResult) const
{
	if (!outResult) {
		return;
	}

	outResult->clear();
	outResult->resize(numSegments);

	BezFwdIterator iter(numSegments, this);
	iter.start();
	Int i = 0;
	while (!iter.done()) {
		(*outResult)[i] = iter.getCurrent();
		++i;
		iter.next();
	}
}

//-------------------------------------------------------------------------------------------------
// This function isn't terribly fast. There are alternatives, and if this is too slow, we can
// take a look at the other approximations.
// There is no known close-form solution to this problem.
Real BezierSegment::getApproximateLength(Real withinTolerance) const
{
	/*
		How this works:
		We can determine the approximate length of a bezier segment by
		L0 = |(P0,P1)| + |(P1,P2)| + |(P2,P3)|
		L1 = |(P0,P3)|

		The length of the segment is approximately 1/2 L0 + 1/2 L1

		P1__P2
		/		 \
	 P0----P3

		The error in this is L1 - L0. If the error is too much, then we subdivide the curve and
		try again.
	*/

	Coord3D p0p1 = { m_controlPoints[1].x - m_controlPoints[0].x,
									 m_controlPoints[1].y - m_controlPoints[0].y,
									 m_controlPoints[1].z - m_controlPoints[0].z };

	Coord3D p1p2 = { m_controlPoints[2].x - m_controlPoints[1].x,
									 m_controlPoints[2].y - m_controlPoints[1].y,
									 m_controlPoints[2].z - m_controlPoints[1].z };

	Coord3D p2p3 = { m_controlPoints[3].x - m_controlPoints[2].x,
									 m_controlPoints[3].y - m_controlPoints[2].y,
									 m_controlPoints[3].z - m_controlPoints[2].z };

	Coord3D p0p3 = { m_controlPoints[3].x - m_controlPoints[0].x,
									 m_controlPoints[3].y - m_controlPoints[0].y,
									 m_controlPoints[3].z - m_controlPoints[0].z };

	Real length0 = p0p3.length();
	Real length1 = p0p1.length() + p1p2.length() + p2p3.length();

	if ((length1 - length0) > withinTolerance) {
		BezierSegment seg1, seg2;
		splitSegmentAtT(0.5f, seg1, seg2);
		return (seg1.getApproximateLength(withinTolerance) + seg2.getApproximateLength(withinTolerance));
	}

	return (INT_TO_REAL(length0 + length1) / 2.0f);
}

//-------------------------------------------------------------------------------------------------
void BezierSegment::splitSegmentAtT(Real tValue, BezierSegment &outSeg1, BezierSegment &outSeg2) const
{
	// I think there are faster ways to do this. Could someone clue me in?

	Coord3D p0p1 = { m_controlPoints[1].x - m_controlPoints[0].x,
									 m_controlPoints[1].y - m_controlPoints[0].y,
									 m_controlPoints[1].z - m_controlPoints[0].z };

	Coord3D p1p2 = { m_controlPoints[2].x - m_controlPoints[1].x,
									 m_controlPoints[2].y - m_controlPoints[1].y,
									 m_controlPoints[2].z - m_controlPoints[1].z };

	Coord3D p2p3 = { m_controlPoints[3].x - m_controlPoints[2].x,
									 m_controlPoints[3].y - m_controlPoints[2].y,
									 m_controlPoints[3].z - m_controlPoints[2].z };

	p0p1.scale(tValue);
	p1p2.scale(tValue);
	p2p3.scale(tValue);

	p0p1.add(m_controlPoints[0]);
	p1p2.add(m_controlPoints[1]);
	p2p3.add(m_controlPoints[2]);

	Coord3D triLeft = { p1p2.x - p0p1.x,
											p1p2.y - p0p1.y,
											p1p2.z - p0p1.z };

	Coord3D triRight = { p2p3.x - p1p2.x,
											 p2p3.y - p1p2.y,
											 p2p3.z - p1p2.z };

	triLeft.scale(tValue);
	triRight.scale(tValue);

	triLeft.add(p0p1);
	triRight.add(p1p2);

	outSeg1.m_controlPoints[0] = m_controlPoints[0];
	outSeg1.m_controlPoints[1] = p0p1;
	outSeg1.m_controlPoints[2] = triLeft;
	evaluateBezSegmentAtT(tValue, &outSeg1.m_controlPoints[3]);

	outSeg2.m_controlPoints[0] = outSeg1.m_controlPoints[3];
	outSeg2.m_controlPoints[1] = triRight;
	outSeg2.m_controlPoints[2] = p2p3;
	outSeg2.m_controlPoints[3] = m_controlPoints[3];
}

//-------------------------------------------------------------------------------------------------
// The Basis Matrix for a bezier segment
#ifndef SAGE_USE_GLM
const D3DXMATRIX BezierSegment::s_bezBasisMatrix(
	-1.0f,  3.0f, -3.0f,  1.0f,
	 3.0f, -6.0f,  3.0f,  0.0f,
	-3.0f,  3.0f,  0.0f,  0.0f,
	 1.0f,  0.0f,  0.0f,  0.0f
);
#else
const glm::mat4 BezierSegment::s_bezBasisMatrix(
	-1.0f,  3.0f, -3.0f,  1.0f,
	 3.0f, -6.0f,  3.0f,  0.0f,
	-3.0f,  3.0f,  0.0f,  0.0f,
	 1.0f,  0.0f,  0.0f,  0.0f
);
#endif
```
