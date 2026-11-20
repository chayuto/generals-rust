/*
 * WP-2.2a GameMath Golden Vector Generator.
 *
 * Links directly against the exact libgm.a static library that GeneralsXZH links
 * (vendor/GeneralsX/build/<preset>/_deps/gamemath-build/libgm.a, built with
 * -ffp-contract=off -fno-builtin), so the oracle is the shipped reference artifact,
 * not a re-compilation.
 *
 * Build & run (from repo root):
 *   cc -O0 -ffp-contract=off -std=c99 \
 *      -Ivendor/GeneralsX/build/macos-vulkan/_deps/gamemath-src/include \
 *      tests/cpp_harness/gen_gamemath_vectors.c \
 *      vendor/GeneralsX/build/macos-vulkan/_deps/gamemath-build/libgm.a \
 *      -o /tmp/gen_gamemath_vectors && /tmp/gen_gamemath_vectors
 *
 * Output: docs/golden_vectors/WP2_2a_GameMath.json (2,000 vectors: 1,000 range-structured
 * seeds + 1,000 raw IEEE-754 bit-pattern seeds).
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "gmath.h"

#define WWMATH_PI 3.141592654f
#define WWMATH_TWO_PI 6.283185308f

static uint32_t splitmix32(uint32_t *state) {
    uint32_t z = (*state += 0x9e3779b9u);
    z = (z ^ (z >> 16)) * 0x85ebca6bu;
    z = (z ^ (z >> 13)) * 0xc2b2ae35u;
    return z ^ (z >> 16);
}

static uint32_t fb(float v) { uint32_t b; memcpy(&b, &v, 4); return b; }
static float bf(uint32_t b) { float v; memcpy(&v, &b, 4); return v; }
static uint64_t db(double v) { uint64_t b; memcpy(&b, &v, 8); return b; }

/* WWMath::Normalize_Angle (wwmath.h:1072) under USE_DETERMINISTIC_MATH. */
static float normalize_angle(float angle) {
    float q = (angle + WWMATH_PI) / WWMATH_TWO_PI;
    return angle - (WWMATH_TWO_PI * gm_floorf(q));
}

static const uint32_t kSpecial[] = {
    0x00000000u, 0x80000000u, /* +-0 */
    0x3f800000u, 0xbf800000u, /* +-1 */
    0x3f000000u, 0xbf000000u, /* +-0.5 */
    0x3f490fdbu, 0xbf490fdbu, /* +-pi/4 */
    0x3fc90fdbu, 0xbfc90fdbu, /* +-pi/2 */
    0x40490fdbu, 0xc0490fdbu, /* +-pi */
    0x40c90fdbu, 0xc0c90fdbu, /* +-2pi */
    0x4016cbe4u, 0x43490f80u, 0x43490f81u, /* rem_pio2 thresholds */
    0x3f490fd8u, 0x3f490fdau, 0x39800000u, 0x397fffffu, /* sin/cos/tan small thresholds */
    0x3f2ca140u, 0x3f79999au, 0x3ee00000u, 0x3f300000u, 0x3f980000u, 0x401c0000u,
    0x50800000u, 0xd0800000u, 0x31000000u, 0x32000000u, 0x23000000u,
    0x00000001u, 0x80000001u, 0x007fffffu, 0x00800000u, /* subnormals / min normal */
    0x7f7fffffu, 0xff7fffffu, /* +-FLT_MAX */
    0x7f800000u, 0xff800000u, /* +-inf */
    0x7fc00000u, /* qNaN */
    0x4b000000u, 0xcb000000u, 0x4afffffeu, 0x3fc00000u, 0xbfc00000u, /* rint/round edges: 2^23, 1.5 */
    0x40200000u, 0xc0200000u, /* +-2.5 */
    0x5d5e0b6bu, 0x6c4ecb8fu, /* 1e18, ~1e27 (deep k_rem_pio2f) */
};
#define NUM_SPECIAL (sizeof(kSpecial) / sizeof(kSpecial[0]))

static void gen_inputs(uint32_t seed, float *x, float *y, float *u, int *n) {
    uint32_t rng = seed * 0x517cc1b7u + 0x33445566u;
    *n = (int)(splitmix32(&rng) % 61u) - 30;
    if (seed >= 1000u) {
        /* Raw IEEE-754 bit patterns across the full float space. */
        *x = bf(splitmix32(&rng));
        *y = bf(splitmix32(&rng));
        *u = bf(splitmix32(&rng));
        return;
    }
    if (seed < NUM_SPECIAL) {
        *x = bf(kSpecial[seed]);
        *y = bf(kSpecial[(seed * 7u + 3u) % NUM_SPECIAL]);
        *u = bf(kSpecial[(seed * 11u + 5u) % NUM_SPECIAL]);
        return;
    }
    {
        uint32_t mode = seed % 6u;
        int32_t iv = (int32_t)(splitmix32(&rng) % 200001u) - 100000;
        int32_t jv = (int32_t)(splitmix32(&rng) % 200001u) - 100000;
        int32_t uv = (int32_t)(splitmix32(&rng) % 2001u) - 1000;
        switch (mode) {
        case 0: *x = (float)iv * 1.0e-9f; break;              /* tiny */
        case 1: *x = (float)iv * (0.78539816f / 100000.0f); break; /* |x| <= pi/4 */
        case 2: *x = (float)iv * (2.3561945f / 100000.0f); break;  /* |x| < 3pi/4 */
        case 3: *x = (float)iv * 0.002f; break;                /* medium, <= 200 */
        case 4: *x = (float)iv * 1.37f; break;                 /* large > 2^7*pi/2 */
        default: *x = (float)iv * 7919.0f; break;              /* very large */
        }
        *y = (float)jv * 0.01f;
        *u = (float)uv / 1000.0f;
    }
}

int main(void) {
    const uint32_t count = 2000u;
    FILE *fp = fopen("docs/golden_vectors/WP2_2a_GameMath.json", "wb");
    if (!fp) { perror("docs/golden_vectors/WP2_2a_GameMath.json"); return 1; }
    fprintf(fp, "{\n  \"packet_id\": \"WP-2.2a\",\n  \"module\": \"GameMath\",\n"
                "  \"oracle\": \"vendor/GeneralsX libgm.a (TheSuperHackers/GameMath, -ffp-contract=off)\",\n"
                "  \"vector_count\": %u,\n  \"vectors\": [\n", count);

    for (uint32_t seed = 0; seed < count; ++seed) {
        float x, y, u;
        int n;
        gen_inputs(seed, &x, &y, &u, &n);
        float ax = gm_fabsf(x);
        double xd = (double)x * 1.000000119;

        fprintf(fp,
            "    {\"seed\": %u, \"x\": %u, \"y\": %u, \"u\": %u, \"n\": %d, \"xd\": %llu, "
            "\"sin\": %u, \"cos\": %u, \"tan\": %u, \"asin\": %u, \"acos\": %u, "
            "\"atan\": %u, \"atan2\": %u, \"sqrt\": %u, \"sqrtd\": %llu, "
            "\"floor\": %u, \"ceil\": %u, \"fabs\": %u, \"rint\": %u, \"round\": %u, "
            "\"scalbn\": %u, \"copysign\": %u, \"norm_angle\": %u}%s\n",
            seed, fb(x), fb(y), fb(u), n, (unsigned long long)db(xd),
            fb(gm_sinf(x)), fb(gm_cosf(x)), fb(gm_tanf(x)), fb(gm_asinf(u)), fb(gm_acosf(u)),
            fb(gm_atanf(x)), fb(gm_atan2f(y, x)), fb(gm_sqrtf(ax)), (unsigned long long)db(gm_sqrt(xd < 0 ? -xd : xd)),
            fb(gm_floorf(x)), fb(gm_ceilf(x)), fb(ax), fb(gm_rintf(x)), fb(gm_roundf(x)),
            fb(gm_scalbnf(x, n)), fb(gm_copysignf(x, y)), fb(normalize_angle(x)),
            (seed + 1 < count) ? "," : "");
    }
    fprintf(fp, "  ]\n}\n");
    fclose(fp);
    printf("Generated %u WP-2.2a GameMath golden vectors from libgm.a\n", count);
    return 0;
}
