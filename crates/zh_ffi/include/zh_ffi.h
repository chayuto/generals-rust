// crates/zh_ffi/include/zh_ffi.h
// C-ABI Strangler Seam Contract between GeneralsXZH (C++) and generals-rust (libzh_ffi.a)
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================================
// Pattern 1: PKT-001 (crc) & PKT-002 (RandomValue)
// ============================================================================

// --- PKT-001: Byte-wise rotate-left-add CRC (Core/GameEngine/Include/Common/crc.h) ---
uint32_t zh_crc_compute_bytes(uint32_t initial_crc, const uint8_t* buf, int32_t len);

// --- PKT-002: 6-Word ADC Cascade PRNG (Core/GameEngine/Include/Common/RandomValue.h) ---
void     zh_random_init_time(void);
void     zh_random_init_seed(uint32_t seed);
uint32_t zh_random_get_logic_base_seed(void);
uint32_t zh_random_get_logic_seed_crc(void);

int32_t  zh_random_get_logic_int(int32_t lo, int32_t hi);
float    zh_random_get_logic_real(float lo, float hi);
int32_t  zh_random_get_logic_int_unchanged(int32_t lo, int32_t hi);
float    zh_random_get_logic_real_unchanged(float lo, float hi);

int32_t  zh_random_get_client_int(int32_t lo, int32_t hi);
float    zh_random_get_client_real(float lo, float hi);
int32_t  zh_random_get_audio_int(int32_t lo, int32_t hi);
float    zh_random_get_audio_real(float lo, float hi);

// Direct state inspection for Gate V2 differential harness & Gate V3 shadow-verify
void     zh_random_get_logic_state_words(uint32_t out_words[6]);
void     zh_random_set_logic_state_words(const uint32_t in_words[6], uint32_t base_seed);
void     zh_random_get_client_state_words(uint32_t out_words[6]);
void     zh_random_set_client_state_words(const uint32_t in_words[6]);
void     zh_random_get_audio_state_words(uint32_t out_words[6]);
void     zh_random_set_audio_state_words(const uint32_t in_words[6]);

// ============================================================================
// Pattern 2: PKT-003 (ExperienceTracker) with Parent Callback VTable
// ============================================================================

typedef struct ZhExpTrackerState {
    int32_t  current_level;       // VeterancyLevel (0 = REGULAR, 1 = VETERAN, 2 = ELITE, 3 = HEROIC)
    int32_t  current_experience;  // Int
    int32_t  experience_sink;     // ObjectID (INVALID_ID = 0)
    float    experience_scalar;   // Real (default 1.0f)
    uint8_t  is_trainable;        // Bool (0 = FALSE, 1 = TRUE)
} ZhExpTrackerState;

// Callback bridge supplied by C++ Object (or by pure-Rust test mock in Gate V2-Golden)
typedef struct ZhExpTrackerParentCallbacks {
    void* parent_ctx;
    uint8_t (*is_template_trainable)(void* parent_ctx);
    int32_t (*get_experience_required)(void* parent_ctx, int32_t level);
    int32_t (*get_experience_value)(void* parent_ctx, int32_t level);
    int32_t (*get_relationship_with_killer)(void* parent_ctx, const void* killer_ctx);
    void    (*on_veterancy_level_changed)(void* parent_ctx, int32_t old_level, int32_t new_level, uint8_t provide_feedback);
    uint8_t (*forward_experience_to_sink)(void* parent_ctx, int32_t sink_obj_id, int32_t scaled_exp_gain, uint8_t can_scale_for_bonus);
    uint8_t (*forward_set_exp_and_level_to_sink)(void* parent_ctx, int32_t sink_obj_id, int32_t exp_in, uint8_t provide_feedback);
} ZhExpTrackerParentCallbacks;

void     zh_exp_tracker_init(ZhExpTrackerState* state, const ZhExpTrackerParentCallbacks* cb);
int32_t  zh_exp_tracker_get_experience_value(const ZhExpTrackerState* state, const ZhExpTrackerParentCallbacks* cb, const void* killer_ctx);
uint8_t  zh_exp_tracker_is_accepting_exp(const ZhExpTrackerState* state);
void     zh_exp_tracker_set_min_veterancy_level(ZhExpTrackerState* state, const ZhExpTrackerParentCallbacks* cb, int32_t new_level, uint8_t provide_feedback);
void     zh_exp_tracker_set_veterancy_level(ZhExpTrackerState* state, const ZhExpTrackerParentCallbacks* cb, int32_t new_level, uint8_t provide_feedback);
uint8_t  zh_exp_tracker_gain_exp_for_level(ZhExpTrackerState* state, const ZhExpTrackerParentCallbacks* cb, int32_t levels_to_gain, uint8_t can_scale_for_bonus);
uint8_t  zh_exp_tracker_can_gain_exp_for_level(const ZhExpTrackerState* state, int32_t levels_to_gain);
void     zh_exp_tracker_add_experience_points(ZhExpTrackerState* state, const ZhExpTrackerParentCallbacks* cb, int32_t experience_gain, uint8_t can_scale_for_bonus);
void     zh_exp_tracker_set_experience_and_level(ZhExpTrackerState* state, const ZhExpTrackerParentCallbacks* cb, int32_t experience_in, uint8_t provide_feedback);
uint32_t zh_exp_tracker_crc(const ZhExpTrackerState* state, uint32_t initial_crc);
int32_t  zh_exp_tracker_xfer_save(const ZhExpTrackerState* state, uint8_t* out_buf, int32_t max_len);
uint8_t  zh_exp_tracker_xfer_load(ZhExpTrackerState* state, const uint8_t* in_buf, int32_t len);

// ============================================================================
// Pattern 3: Universal BehaviorModule Bridge (Phase 5, All 222 Modules)
// ============================================================================

typedef uint32_t ZhModuleSlot;
typedef uint64_t ZhFacetMask;

#define ZH_FACET_NONE                 UINT64_C(0)
#define ZH_FACET_UPDATE               (UINT64_C(1) << 0)
#define ZH_FACET_DIE                  (UINT64_C(1) << 1)
#define ZH_FACET_DAMAGE               (UINT64_C(1) << 2)
#define ZH_FACET_CREATE               (UINT64_C(1) << 3)
#define ZH_FACET_COLLIDE              (UINT64_C(1) << 4)
#define ZH_FACET_BODY                 (UINT64_C(1) << 5)
#define ZH_FACET_CONTAIN              (UINT64_C(1) << 6)
#define ZH_FACET_UPGRADE              (UINT64_C(1) << 7)
#define ZH_FACET_SPECIAL_POWER        (UINT64_C(1) << 8)
#define ZH_FACET_DESTROY              (UINT64_C(1) << 9)
#define ZH_FACET_DRAW                 (UINT64_C(1) << 10)
#define ZH_FACET_CLIENT_UPDATE        (UINT64_C(1) << 11)
#define ZH_FACET_AI_UPDATE            (UINT64_C(1) << 12)
#define ZH_FACET_PHYSICS_UPDATE       (UINT64_C(1) << 13)
#define ZH_FACET_STEALTH_UPDATE       (UINT64_C(1) << 14)
#define ZH_FACET_DOCK_UPDATE          (UINT64_C(1) << 15)
#define ZH_FACET_EXIT_INTERFACE       (UINT64_C(1) << 16)
#define ZH_FACET_SLAVED_UPDATE        (UINT64_C(1) << 17)
#define ZH_FACET_PROJECTILE_UPDATE    (UINT64_C(1) << 18)
#define ZH_FACET_PRODUCTION_UPDATE    (UINT64_C(1) << 19)
#define ZH_FACET_SLOW_DEATH           (UINT64_C(1) << 20)
#define ZH_FACET_POWER_PLANT_UPDATE   (UINT64_C(1) << 21)
#define ZH_FACET_HORDE_UPDATE         (UINT64_C(1) << 22)
#define ZH_FACET_SPAWN_BEHAVIOR       (UINT64_C(1) << 23)
#define ZH_FACET_COUNTERMEASURES      (UINT64_C(1) << 24)
#define ZH_FACET_SPECIAL_POWER_UPDATE (UINT64_C(1) << 25)
#define ZH_FACET_PARKING_PLACE        (UINT64_C(1) << 26)
#define ZH_FACET_CAVE_INTERFACE       (UINT64_C(1) << 27)
#define ZH_FACET_LAND_MINE            (UINT64_C(1) << 28)
#define ZH_FACET_RAILED_TRANSPORT_DOCK (UINT64_C(1) << 29)
#define ZH_FACET_TRANSPORT_PASSENGER  (UINT64_C(1) << 30)

uint8_t      zh_module_is_ported(int32_t module_name_key);
ZhModuleSlot zh_module_create(
    int32_t module_name_key,
    uint32_t owner_obj_id,
    uint16_t template_id,
    int32_t tag_key,
    ZhFacetMask* out_facet_mask,
    uint32_t* out_initial_priority);
void         zh_module_on_object_created(ZhModuleSlot slot);
void         zh_module_on_delete(ZhModuleSlot slot);
void         zh_module_destroy(ZhModuleSlot slot);

uint32_t     zh_module_update(ZhModuleSlot slot);
uint64_t     zh_module_get_disabled_types_to_process(ZhModuleSlot slot);
void         zh_module_on_die(ZhModuleSlot slot, const void* damage_info_ptr);
void         zh_module_on_damage(ZhModuleSlot slot, void* damage_info_ptr);
void         zh_module_on_create(ZhModuleSlot slot);
void         zh_module_on_build_complete(ZhModuleSlot slot);
void         zh_module_on_collide(ZhModuleSlot slot, uint32_t other_obj_id, const void* loc_ptr, const void* normal_ptr);
void         zh_module_on_destroy(ZhModuleSlot slot);
void         zh_module_on_capture(ZhModuleSlot slot, int32_t old_player_idx, int32_t new_player_idx);
void         zh_module_on_disabled_edge(ZhModuleSlot slot, uint8_t now_disabled);

void         zh_module_crc(ZhModuleSlot slot, void* cxx_xfer_ptr);
void         zh_module_xfer(ZhModuleSlot slot, void* cxx_xfer_ptr);
void         zh_module_load_post_process(ZhModuleSlot slot);

#ifdef __cplusplus
}
#endif
