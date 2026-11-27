// Golden trace vectors for the turrets (AI plan slice S14, second half): the
// attack-state trace of gen_s14_attack_states.cpp with turreted units (single,
// two and linked turrets, an immobile one, a sweeping one, one that cannot turn
// and one that starts disabled), the reference's own idle mood targets, turrets
// switched off and recentred, each turret's angle and machine every frame and
// the projectile launch transforms at the end of each scene.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s14_turrets.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/S14_Turrets.json
//
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface10doPathfindEP25PathfindServicesInterface
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder19getMoveAwayFromPathEP6ObjectS1_P4PathS1_S3_
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a PartitionManager.cpp.o __ZN16PartitionManager26getGroundOrStructureHeightEff
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface17getNextMoodTargetEbb
//
// Not covered: turreted containers, pitch toward aircraft, jets, loading a save.
#define S14_TURRETS 1
#include "gen_s14_attack_states.cpp"
